use crate::storage::get_pool;
use crate::Wallpaper;
use super::core::get_wallpaper_by_id;

pub async fn get_user_favorites(
    user_id: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let pool = get_pool()?;
    let offset = page * limit;
    let rows = sqlx::query!(
        r#"
        SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, w.thumbnail_url, w.tags as "tags: sqlx::types::Json<Vec<String>>", w.primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live FROM wallpapers w
        INNER JOIN user_favorites f ON w.id = f.wallpaper_id
        JOIN users u ON w.author_id = u.id
        WHERE f.user_id = $1
        ORDER BY w.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    let results: Vec<Wallpaper> = rows
        .into_iter()
        .map(|r| Wallpaper {
            id: r.id,
            title: r.title,
            author_id: r.author_id,
            author_name: r.author_name,
            image_url: r.image_url,
            thumbnail_url: r.thumbnail_url,
            tags: r.tags.0,
            primary_colors: r.primary_colors.0,
            dimensions: (r.width as u32, r.height as u32),
            size_bytes: r.size_bytes as u64,
            likes: r.likes.unwrap_or(0) as u32,
            downloads: r.downloads.unwrap_or(0) as u32,
            created_at: r.created_at,
            is_private: r.is_private,
            is_live: r.is_live,
            embedding: None,
            phash: None,
        })
        .collect();
    let arc_results = std::sync::Arc::new(results);
    Ok(arc_results)
}

pub async fn check_favorites_db(user_id: &str, wallpaper_ids: &[String]) -> anyhow::Result<Vec<String>> {
    if wallpaper_ids.is_empty() {
        return Ok(Vec::new());
    }
    let pool = get_pool()?;
    
    // Convert Vec<String> to a postgres array
    let rows = sqlx::query!(
        r#"
        SELECT wallpaper_id FROM user_favorites
        WHERE user_id = $1 AND wallpaper_id = ANY($2)
        "#,
        user_id,
        wallpaper_ids as &[String]
    )
    .fetch_all(pool)
    .await?;

    let ids = rows.into_iter().map(|r| r.wallpaper_id).collect();
    Ok(ids)
}

pub async fn toggle_favorite(user_id: &str, wallpaper_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;

    let mut tx = pool.begin().await?;

    let exists: Option<i32> = sqlx::query_scalar!(
        "SELECT 1 as result FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2",
        user_id,
        wallpaper_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten();

    if exists.is_some() {
        sqlx::query!(
            "DELETE FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2",
            user_id,
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE wallpapers SET likes = likes - 1 WHERE id = $1",
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        crate::storage::cache::get_wallpaper_cache()
            .remove(wallpaper_id)
            .await;
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

        Ok(false)
    } else {
        sqlx::query!(
            "INSERT INTO user_favorites (user_id, wallpaper_id) VALUES ($1, $2)",
            user_id,
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE wallpapers SET likes = likes + 1 WHERE id = $1",
            wallpaper_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        crate::storage::cache::get_wallpaper_cache()
            .remove(wallpaper_id)
            .await;
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

        if let Ok(Some(wp)) = get_wallpaper_by_id(wallpaper_id).await {
            if let Ok(Some(author_record)) = crate::storage::users::get_user_by_id(&wp.author_id).await {
                if let Ok(Some(liker)) = crate::storage::users::get_user_by_id(user_id).await {
                    if author_record.user.id != user_id {
                        let _ = crate::storage::notifications::create_notification_db(
                            &author_record.user.id,
                            "New Like",
                            &format!("{} liked your wallpaper '{}'", liker.user.name, wp.title),
                        )
                        .await;
                    }
                }
            }
        }

        Ok(true)
    }
}

pub async fn is_favorited(user_id: &str, wallpaper_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;
    let exists: Option<i32> = sqlx::query_scalar!(
        "SELECT 1 as result FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2",
        user_id,
        wallpaper_id
    )
    .fetch_optional(pool)
    .await?
    .flatten();
    Ok(exists.is_some())
}

pub async fn increment_download(id: &str, ip: &str) -> anyhow::Result<()> {
    let cache = crate::storage::cache::get_download_rate_limit_cache();
    let key = format!("{}:{}", ip, id);
    if cache.get(&key).await.is_some() {
        return Ok(());
    }
    cache.insert(key, true).await;

    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE wallpapers SET downloads = downloads + 1 WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;

    crate::storage::cache::get_wallpaper_cache().remove(id).await;
    crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

    Ok(())
}

pub async fn record_user_download_db(user_id: &str, wallpaper_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "INSERT INTO user_downloads (user_id, wallpaper_id) VALUES ($1, $2) ON CONFLICT (user_id, wallpaper_id) DO UPDATE SET downloaded_at = NOW()",
        user_id, wallpaper_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_download_history_db(
    user_id: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let pool = get_pool()?;
    let offset = page * limit;
    let rows = sqlx::query!(
        r#"
        SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, w.thumbnail_url, w.tags as "tags: sqlx::types::Json<Vec<String>>", w.primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live 
        FROM wallpapers w
        INNER JOIN user_downloads d ON w.id = d.wallpaper_id
        JOIN users u ON w.author_id = u.id
        WHERE d.user_id = $1
        ORDER BY d.downloaded_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    let results: Vec<Wallpaper> = rows
        .into_iter()
        .map(|r| Wallpaper {
            id: r.id,
            title: r.title,
            author_id: r.author_id,
            author_name: r.author_name,
            image_url: r.image_url,
            thumbnail_url: r.thumbnail_url,
            tags: r.tags.0,
            primary_colors: r.primary_colors.0,
            dimensions: (r.width as u32, r.height as u32),
            size_bytes: r.size_bytes as u64,
            likes: r.likes.unwrap_or(0) as u32,
            downloads: r.downloads.unwrap_or(0) as u32,
            created_at: r.created_at,
            is_private: r.is_private,
            is_live: r.is_live,
            embedding: None,
            phash: None,
        })
        .collect();
    let arc_results = std::sync::Arc::new(results);
    Ok(arc_results)
}

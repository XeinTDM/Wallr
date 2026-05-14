use crate::storage::get_pool;
use crate::Wallpaper;

pub async fn get_trending_tags(limit: u32) -> anyhow::Result<Vec<String>> {
    let cache = crate::storage::cache::get_trending_tags_cache();
    if let Some(cached) = cache.get(&limit.to_string()).await {
        return Ok(cached);
    }

    let pool = get_pool()?;

    let rows = sqlx::query!(
        r#"
        SELECT tag, count
        FROM trending_tags
        ORDER BY count DESC
        LIMIT $1
        "#,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    let mut tags = Vec::new();
    for row in rows {
        let tag: String = row.tag.unwrap_or_default();
        tags.push(tag);
    }

    cache.insert(limit.to_string(), tags.clone()).await;

    Ok(tags)
}

pub async fn get_user_uploads(
    author_name: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache_key = format!("uploads:{}:{}:{}", author_name, page, limit);
    let author_name_cloned = author_name.to_string();

    crate::storage::wallpapers::core::fetch_and_cache_wallpaper_list(cache_key, || async move {
        let pool = get_pool()?;
        let offset = page * limit;
        let rows = sqlx::query!(
            r#"
            SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, w.created_at, is_private, is_live FROM wallpapers w JOIN users u ON w.author_id = u.id
            WHERE u.name = $1
            ORDER BY w.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            author_name_cloned,
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
                description: None,
                source_url: None,
            })
            .collect();
        Ok(results)
    })
    .await
}

pub async fn get_public_uploads(
    author_name: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache_key = format!("public_uploads:{}:{}:{}", author_name, page, limit);
    let author_name_cloned = author_name.to_string();

    crate::storage::wallpapers::core::fetch_and_cache_wallpaper_list(cache_key, || async move {
        let pool = get_pool()?;
        let offset = page * limit;
        let rows = sqlx::query!(
            r#"
            SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, w.created_at, is_private, is_live FROM wallpapers w JOIN users u ON w.author_id = u.id
            WHERE u.name = $1 AND is_private = false
            ORDER BY w.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            author_name_cloned,
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
                description: None,
                source_url: None,
            })
            .collect();
        Ok(results)
    })
    .await
}

pub async fn get_creator_analytics_db(
    author_name: &str,
) -> anyhow::Result<crate::CreatorAnalytics> {
    let pool = get_pool()?;

    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total_uploads!",
            COALESCE(SUM(w.likes), 0) as "total_likes!",
            COALESCE(SUM(w.downloads), 0) as "total_downloads!"
        FROM wallpapers w JOIN users u ON w.author_id = u.id
        WHERE u.name = $1
        "#,
        author_name
    )
    .fetch_one(pool)
    .await?;

    Ok(crate::CreatorAnalytics {
        total_uploads: row.total_uploads as u32,
        total_likes: row.total_likes as u32,
        total_downloads: row.total_downloads as u32,
    })
}

pub async fn get_admin_stats_db() -> anyhow::Result<crate::AdminStats> {
    let pool = get_pool()?;

    let w_stats = sqlx::query!(
        "SELECT COUNT(*) as wp_count, COALESCE(SUM(downloads), 0) as total_downloads, COALESCE(SUM(likes), 0) as total_likes FROM wallpapers"
    ).fetch_one(pool).await?;

    let u_stats = sqlx::query!("SELECT COUNT(*) as user_count FROM users")
        .fetch_one(pool)
        .await?;

    Ok(crate::AdminStats {
        total_users: u_stats.user_count.unwrap_or(0) as u32,
        total_wallpapers: w_stats.wp_count.unwrap_or(0) as u32,
        total_downloads: w_stats.total_downloads.unwrap_or(0) as u32,
        total_likes: w_stats.total_likes.unwrap_or(0) as u32,
    })
}

pub async fn refresh_trending_tags_view() -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("REFRESH MATERIALIZED VIEW CONCURRENTLY trending_tags")
        .execute(pool)
        .await?;
    Ok(())
}

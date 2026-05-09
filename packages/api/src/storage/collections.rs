use super::cache::get_collection_cache;
use super::get_pool;

pub async fn load_all_collections() -> anyhow::Result<Vec<crate::Collection>> {
    let cache = get_collection_cache();
    if let Some(cached) = cache.get("all").await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let rows = sqlx::query!("SELECT * FROM collections ORDER BY name ASC")
        .fetch_all(pool)
        .await?;

    let results: Vec<crate::Collection> = rows
        .into_iter()
        .map(|row| {
            crate::Collection {
                id: row.id,
                name: row.name,
                item_count: row.item_count as u32,
                cover_url: row.cover_url,
            }
        })
        .collect();

    cache.insert("all".to_string(), results.clone()).await;
    Ok(results)
}

pub async fn create_user_collection(
    user_id: &str,
    name: &str,
    description: Option<&str>,
    is_private: bool,
) -> anyhow::Result<String> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO user_collections (id, user_id, name, description, is_private) VALUES ($1, $2, $3, $4, $5)",
        id, user_id, name, description, is_private
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_user_collections(user_id: &str) -> anyhow::Result<Vec<crate::UserCollection>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id, c.user_id, c.name, c.description, c.is_private, c.created_at,
            (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.id) as item_count,
            (SELECT w.thumbnail_url FROM collection_items ci JOIN wallpapers w ON w.id = ci.wallpaper_id WHERE ci.collection_id = c.id ORDER BY ci.added_at DESC LIMIT 1) as cover_url
        FROM user_collections c
        WHERE c.user_id = $1
        ORDER BY c.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| crate::UserCollection {
        id: r.id,
        user_id: r.user_id,
        name: r.name,
        description: r.description,
        is_private: r.is_private,
        item_count: r.item_count.unwrap_or(0) as u32,
        cover_url: r.cover_url,
        created_at: r.created_at,
    }).collect())
}

pub async fn get_public_user_collections_db(user_id: &str) -> anyhow::Result<Vec<crate::UserCollection>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id, c.user_id, c.name, c.description, c.is_private, c.created_at,
            (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.id) as item_count,
            (SELECT w.thumbnail_url FROM collection_items ci JOIN wallpapers w ON w.id = ci.wallpaper_id WHERE ci.collection_id = c.id ORDER BY ci.added_at DESC LIMIT 1) as cover_url
        FROM user_collections c
        WHERE c.user_id = $1 AND c.is_private = false
        ORDER BY c.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| crate::UserCollection {
        id: r.id,
        user_id: r.user_id,
        name: r.name,
        description: r.description,
        is_private: r.is_private,
        item_count: r.item_count.unwrap_or(0) as u32,
        cover_url: r.cover_url,
        created_at: r.created_at,
    }).collect())
}

pub async fn add_wallpaper_to_collection_db(collection_id: &str, wallpaper_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "INSERT INTO collection_items (collection_id, wallpaper_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        collection_id, wallpaper_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_wallpaper_from_collection_db(collection_id: &str, wallpaper_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "DELETE FROM collection_items WHERE collection_id = $1 AND wallpaper_id = $2",
        collection_id, wallpaper_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_collection_wallpapers_db(collection_id: &str, page: u32, limit: u32) -> anyhow::Result<std::sync::Arc<Vec<crate::Wallpaper>>> {
    let pool = get_pool()?;
    let offset = (page * limit) as i64;
    let limit = limit as i64;

    let rows = sqlx::query!(
        r#"
        SELECT 
            w.id, w.title, w.author, w.image_url, w.thumbnail_url, 
            w.tags, w.primary_colors, w.width, w.height, w.size_bytes, 
            w.likes, w.downloads, w.created_at, w.is_private
        FROM wallpapers w
        JOIN collection_items ci ON ci.wallpaper_id = w.id
        WHERE ci.collection_id = $1
        ORDER BY ci.added_at DESC
        LIMIT $2 OFFSET $3
        "#,
        collection_id, limit, offset
    )
    .fetch_all(pool)
    .await?;

    let mut wallpapers = Vec::new();
    for row in rows {
        let tags: Vec<String> = serde_json::from_value(row.tags).unwrap_or_default();
        let primary_colors: Vec<String> = serde_json::from_value(row.primary_colors).unwrap_or_default();
        wallpapers.push(crate::Wallpaper {
            id: row.id,
            title: row.title,
            author: row.author,
            image_url: row.image_url,
            thumbnail_url: row.thumbnail_url,
            tags,
            primary_colors,
            dimensions: (row.width as u32, row.height as u32),
            size_bytes: row.size_bytes as u64,
            likes: row.likes.unwrap_or(0) as u32,
            downloads: row.downloads.unwrap_or(0) as u32,
            created_at: row.created_at,
            is_private: row.is_private,
        });
    }

    Ok(std::sync::Arc::new(wallpapers))
}

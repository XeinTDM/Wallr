use super::get_pool;
use crate::models::{EditorialCollection, Wallpaper};
use uuid::Uuid;

pub async fn get_published_editorial_collections() -> anyhow::Result<Vec<EditorialCollection>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id, c.title, c.description, c.cover_url, c.is_published, c.published_at, c.created_at,
            COALESCE(ic.count, 0) as item_count
        FROM editorial_collections c
        LEFT JOIN (
            SELECT collection_id, COUNT(*) as count
            FROM editorial_collection_items
            GROUP BY collection_id
        ) ic ON ic.collection_id = c.id
        WHERE c.is_published = true
        ORDER BY c.published_at DESC NULLS LAST
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| EditorialCollection {
            id: r.id,
            title: r.title,
            description: r.description,
            cover_url: r.cover_url,
            is_published: r.is_published,
            published_at: r.published_at,
            created_at: r.created_at,
            item_count: r.item_count.unwrap_or(0) as u32,
        })
        .collect())
}

pub async fn get_all_editorial_collections() -> anyhow::Result<Vec<EditorialCollection>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id, c.title, c.description, c.cover_url, c.is_published, c.published_at, c.created_at,
            COALESCE(ic.count, 0) as item_count
        FROM editorial_collections c
        LEFT JOIN (
            SELECT collection_id, COUNT(*) as count
            FROM editorial_collection_items
            GROUP BY collection_id
        ) ic ON ic.collection_id = c.id
        ORDER BY c.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| EditorialCollection {
            id: r.id,
            title: r.title,
            description: r.description,
            cover_url: r.cover_url,
            is_published: r.is_published,
            published_at: r.published_at,
            created_at: r.created_at,
            item_count: r.item_count.unwrap_or(0) as u32,
        })
        .collect())
}

pub async fn get_editorial_collection_wallpapers(
    collection_id: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<Wallpaper>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT w.id, w.title, w.author as "author_name", w.image_url, w.thumbnail_url, 
               w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, 
               w.downloads, w.search_vector, w.created_at, w.is_private, w.is_live
        FROM editorial_collection_items ci
        JOIN wallpapers w ON w.id = ci.wallpaper_id
        WHERE ci.collection_id = $1
        ORDER BY ci.sort_order ASC, ci.added_at DESC
        LIMIT $2 OFFSET $3
        "#,
        collection_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Wallpaper {
            id: r.id,
            title: r.title,
            author_id: "".to_string(), // Default since it was not stored previously or author is just string
            author_name: r.author_name,
            image_url: r.image_url,
            thumbnail_url: r.thumbnail_url,
            tags: serde_json::from_value(r.tags).unwrap_or_default(),
            primary_colors: serde_json::from_value(r.primary_colors).unwrap_or_default(),
            dimensions: (r.width as u32, r.height as u32),
            size_bytes: r.size_bytes as u64,
            likes: r.likes.unwrap_or(0) as u32,
            downloads: r.downloads.unwrap_or(0) as u32,
            created_at: r.created_at.unwrap_or_else(|| chrono::Utc::now()),
            is_private: r.is_private.unwrap_or(false),
            is_live: r.is_live.unwrap_or(false),
            embedding: None,
            phash: None,
            description: None,
            source_url: None,
        })
        .collect())
}

pub async fn create_editorial_collection(
    title: &str,
    description: &str,
    cover_url: Option<&str>,
    is_published: bool,
) -> anyhow::Result<String> {
    let pool = get_pool()?;
    let id = Uuid::new_v4().to_string();
    let published_at = if is_published { Some(chrono::Utc::now()) } else { None };

    sqlx::query!(
        "INSERT INTO editorial_collections (id, title, description, cover_url, is_published, published_at) VALUES ($1, $2, $3, $4, $5, $6)",
        id, title, description, cover_url, is_published, published_at
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn update_editorial_collection(
    id: &str,
    title: &str,
    description: &str,
    cover_url: Option<&str>,
    is_published: bool,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    
    // Check current state to see if published_at needs updating
    let current = sqlx::query!("SELECT is_published FROM editorial_collections WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;

    if let Some(c) = current {
        let published_at = if is_published && !c.is_published {
            Some(chrono::Utc::now())
        } else if !is_published {
            None
        } else {
            // Unchanged or already published, just leave as is via COALESCE or update logic
            // simpler to just update only is_published if we don't care to keep published_at when turning off,
            // but let's just do a simple approach.
            None
        };
        
        if is_published && !c.is_published {
            sqlx::query!(
                "UPDATE editorial_collections SET title = $1, description = $2, cover_url = $3, is_published = $4, published_at = $5 WHERE id = $6",
                title, description, cover_url, is_published, chrono::Utc::now(), id
            ).execute(pool).await?;
        } else {
            sqlx::query!(
                "UPDATE editorial_collections SET title = $1, description = $2, cover_url = $3, is_published = $4 WHERE id = $5",
                title, description, cover_url, is_published, id
            ).execute(pool).await?;
        }
    }
    
    Ok(())
}

pub async fn add_wallpaper_to_editorial_collection(
    collection_id: &str,
    wallpaper_id: &str,
    sort_order: i32,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "INSERT INTO editorial_collection_items (collection_id, wallpaper_id, sort_order) VALUES ($1, $2, $3) ON CONFLICT (collection_id, wallpaper_id) DO UPDATE SET sort_order = $3",
        collection_id, wallpaper_id, sort_order
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_wallpaper_from_editorial_collection(
    collection_id: &str,
    wallpaper_id: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "DELETE FROM editorial_collection_items WHERE collection_id = $1 AND wallpaper_id = $2",
        collection_id, wallpaper_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

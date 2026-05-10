use crate::storage::get_pool;
use crate::Wallpaper;

pub(crate) fn map_wallpaper_row(row: sqlx::postgres::PgRow) -> Wallpaper {
    use sqlx::Row;
    let tags_val: sqlx::types::Json<Vec<String>> = row.get("tags");
    let colors_val: sqlx::types::Json<Vec<String>> = row.get("primary_colors");
    let width: i32 = row.get("width");
    let height: i32 = row.get("height");
    let size_bytes: i64 = row.get("size_bytes");
    let likes: i32 = row.get("likes");
    let downloads: i32 = row.get("downloads");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let is_private: bool = row.get("is_private");
    let is_live: bool = row.try_get("is_live").unwrap_or(false);
    let phash: Option<Vec<u8>> = row.try_get("phash").unwrap_or(None);

    Wallpaper {
        id: row.get("id"),
        title: row.get("title"),
        author: row.get("author"),
        image_url: row.get("image_url"),
        thumbnail_url: row.get("thumbnail_url"),
        tags: tags_val.0,
        primary_colors: colors_val.0,
        dimensions: (width as u32, height as u32),
        size_bytes: size_bytes as u64,
        likes: likes as u32,
        downloads: downloads as u32,
        created_at,
        is_private,
        is_live,
        embedding: None,
        phash,
    }
}

pub async fn save_wallpaper_data(wallpaper: &Wallpaper) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let embed = wallpaper.embedding.clone().map(pgvector::Vector::from);
    let phash_ref = wallpaper.phash.as_deref();

    sqlx::query!(
        r#"
        INSERT INTO wallpapers (id, title, author, image_url, thumbnail_url, tags, primary_colors, width, height, size_bytes, likes, downloads, created_at, is_private, is_live, embedding, phash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            image_url = EXCLUDED.image_url,
            thumbnail_url = EXCLUDED.thumbnail_url,
            tags = EXCLUDED.tags,
            primary_colors = EXCLUDED.primary_colors,
            width = EXCLUDED.width,
            height = EXCLUDED.height,
            size_bytes = EXCLUDED.size_bytes,
            likes = EXCLUDED.likes,
            downloads = EXCLUDED.downloads,
            is_private = EXCLUDED.is_private,
            is_live = EXCLUDED.is_live,
            embedding = EXCLUDED.embedding,
            phash = EXCLUDED.phash
        "#,
        wallpaper.id,
        wallpaper.title,
        wallpaper.author,
        wallpaper.image_url,
        wallpaper.thumbnail_url,
        sqlx::types::Json(&wallpaper.tags) as _,
        sqlx::types::Json(&wallpaper.primary_colors) as _,
        wallpaper.dimensions.0 as i32,
        wallpaper.dimensions.1 as i32,
        wallpaper.size_bytes as i64,
        wallpaper.likes as i32,
        wallpaper.downloads as i32,
        wallpaper.created_at,
        wallpaper.is_private,
        wallpaper.is_live,
        embed as _,
        phash_ref
    )
    .execute(pool)
    .await?;

    get_wallpaper_cache().remove(&wallpaper.id).await;

    Ok(())
}

pub async fn get_wallpaper_by_id(id: &str) -> anyhow::Result<Option<Wallpaper>> {
    let cache = get_wallpaper_cache();
    if let Some(cached) = cache.get(id).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let row = sqlx::query!(r#"SELECT id, title, author, image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, created_at, is_private, is_live FROM wallpapers WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await?;

    let result = row.map(|r| Wallpaper {
        id: r.id,
        title: r.title,
        author: r.author,
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
        phash: None, // We don't need to load phash into memory for general queries unless necessary
    });
    cache.insert(id.to_string(), result.clone()).await;
    Ok(result)
}

pub async fn update_wallpaper_db(
    id: &str,
    title: &str,
    tags: &Vec<String>,
    is_private: bool,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE wallpapers SET title = $1, tags = $2, is_private = $3 WHERE id = $4",
        title,
        sqlx::types::Json(tags) as _,
        is_private,
        id
    )
    .execute(pool)
    .await?;

    crate::storage::cache::get_wallpaper_cache().remove(id).await;
    crate::storage::cache::get_wallpaper_list_cache().invalidate_all();
    Ok(())
}

pub async fn delete_wallpaper(id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let mut tx = pool.begin().await?;

    sqlx::query!("DELETE FROM user_favorites WHERE wallpaper_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM wallpapers WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    crate::storage::cache::get_wallpaper_cache().remove(id).await;
    crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

    let storage_path = crate::storage::files::get_storage_path();
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_master.jpg", id))).await;
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_master.avif", id))).await;
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_thumb.jpg", id))).await;

    Ok(())
}

pub async fn add_tag(wallpaper_id: &str, tag: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let tag_array_json = serde_json::json!([tag]);

    let exists: Option<i32> = sqlx::query_scalar!(
        r#"SELECT 1 as result FROM wallpapers WHERE id = $1 AND tags @> $2"#,
        wallpaper_id,
        tag_array_json
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    if exists.is_none() {
        sqlx::query!(
            r#"UPDATE wallpapers SET tags = tags || $2 WHERE id = $1"#,
            wallpaper_id,
            tag_array_json
        )
        .execute(pool)
        .await?;

        crate::storage::cache::get_wallpaper_cache()
            .remove(wallpaper_id)
            .await;
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();
    }

    Ok(())
}

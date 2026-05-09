use crate::Wallpaper;
use super::cache::{get_wallpaper_cache, get_wallpaper_list_cache};
use super::get_pool;

pub async fn save_wallpaper_data(wallpaper: &Wallpaper) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        r#"
        INSERT INTO wallpapers (id, title, author, image_url, thumbnail_url, tags, primary_colors, width, height, size_bytes, likes, downloads, created_at, is_private)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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
            is_private = EXCLUDED.is_private
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
        wallpaper.is_private
    )
    .execute(pool)
    .await?;

    get_wallpaper_cache().remove(&wallpaper.id).await;

    Ok(())
}

fn map_wallpaper_row(row: sqlx::postgres::PgRow) -> Wallpaper {
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
    }
}

fn apply_filters(q: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>, filters: &crate::FilterOptions) {
    q.push(" AND is_private = false");

    if !filters.resolution.is_empty() {
        match filters.resolution.as_str() {
            "4k" => { q.push(" AND width >= 3840 AND height >= 2160"); },
            "8k" => { q.push(" AND width >= 7680 AND height >= 4320"); },
            "hd" => { q.push(" AND width >= 1920 AND height >= 1080"); },
            _ => {}
        }
    }
    if !filters.aspect_ratio.is_empty() {
        match filters.aspect_ratio.as_str() {
            "ultrawide" => { q.push(" AND (width::float / height::float) >= 2.3"); },
            "desktop" => { q.push(" AND (width::float / height::float) >= 1.3 AND (width::float / height::float) < 2.3"); },
            "mobile" => { q.push(" AND (width::float / height::float) < 1.0"); },
            _ => {}
        }
    }
    if !filters.color.is_empty() {
        q.push(" AND primary_colors ? ");
        q.push_bind(filters.color.clone());
    }
    if !filters.ai_filter.is_empty() {
        match filters.ai_filter.as_str() {
            "hide" => { q.push(" AND NOT (tags @> '[\"ai\"]')"); },
            "only" => { q.push(" AND tags @> '[\"ai\"]'"); },
            _ => {}
        }
    }
    if !filters.timeframe.is_empty() {
        match filters.timeframe.as_str() {
            "daily" => { q.push(" AND created_at >= NOW() - INTERVAL '1 day'"); },
            "weekly" => { q.push(" AND created_at >= NOW() - INTERVAL '7 days'"); },
            "monthly" => { q.push(" AND created_at >= NOW() - INTERVAL '30 days'"); },
            "yearly" => { q.push(" AND created_at >= NOW() - INTERVAL '1 year'"); },
            _ => {}
        }
    }
}

pub async fn load_all_wallpapers(
    page: u32,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache = get_wallpaper_list_cache();
    let cache_key = format!("all:{}:{}:{:?}", page, limit, filters);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let offset = page * limit;

    let mut q = sqlx::QueryBuilder::new("SELECT * FROM wallpapers WHERE 1=1");
    apply_filters(&mut q, &filters);

    match filters.sort.as_str() {
        "downloads" => { q.push(" ORDER BY downloads DESC"); },
        "rating" => { q.push(" ORDER BY likes DESC"); },
        "date" | _ => { q.push(" ORDER BY created_at DESC"); },
    }

    q.push(" LIMIT ");
    q.push_bind(limit as i64);
    q.push(" OFFSET ");
    q.push_bind(offset as i64);

    let rows = q.build().fetch_all(pool).await?;

    let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;
    Ok(arc_results)
}

pub async fn get_wallpaper_by_id(id: &str) -> anyhow::Result<Option<Wallpaper>> {
    let cache = get_wallpaper_cache();
    if let Some(cached) = cache.get(id).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let row = sqlx::query!(r#"SELECT id, title, author, image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, created_at, is_private FROM wallpapers WHERE id = $1"#, id)
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
    });
    cache.insert(id.to_string(), result.clone()).await;
    Ok(result)
}

pub async fn get_wallpapers_by_tag(
    tag: &str,
    page: u32,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache = get_wallpaper_list_cache();
    let cache_key = format!("tag:{}:{}:{}:{:?}", tag, page, limit, filters);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let tag_json = serde_json::json!(tag);
    let offset = page * limit;

    let mut q = sqlx::QueryBuilder::new("SELECT * FROM wallpapers WHERE tags @> ");
    q.push_bind(tag_json);
    apply_filters(&mut q, &filters);

    match filters.sort.as_str() {
        "downloads" => { q.push(" ORDER BY downloads DESC"); },
        "rating" => { q.push(" ORDER BY likes DESC"); },
        "date" | _ => { q.push(" ORDER BY created_at DESC"); },
    }

    q.push(" LIMIT ");
    q.push_bind(limit as i64);
    q.push(" OFFSET ");
    q.push_bind(offset as i64);

    let rows = q.build().fetch_all(pool).await?;

    let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;
    Ok(arc_results)
}

pub async fn get_trending_tags(limit: u32) -> anyhow::Result<Vec<String>> {
    let cache = super::cache::get_trending_tags_cache();
    if let Some(cached) = cache.get(&limit).await {
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

    cache.insert(limit, tags.clone()).await;

    Ok(tags)
}

pub async fn get_user_favorites(
    user_id: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let pool = get_pool()?;
    let offset = page * limit;
    let rows = sqlx::query!(
        r#"
        SELECT w.id, w.title, w.author, w.image_url, w.thumbnail_url, w.tags as "tags: sqlx::types::Json<Vec<String>>", w.primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private FROM wallpapers w
        INNER JOIN user_favorites f ON w.id = f.wallpaper_id
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

    let results: Vec<Wallpaper> = rows.into_iter().map(|r| Wallpaper {
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
    }).collect();
    let arc_results = std::sync::Arc::new(results);
    Ok(arc_results)
}

pub async fn get_all_user_favorite_ids(user_id: &str) -> anyhow::Result<Vec<String>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        r#"
        SELECT wallpaper_id FROM user_favorites
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let ids = rows.into_iter().map(|r| r.wallpaper_id).collect();
    Ok(ids)
}

pub async fn get_user_uploads(
    author_name: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache = get_wallpaper_list_cache();
    let cache_key = format!("uploads:{}:{}:{}", author_name, page, limit);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let offset = page * limit;
    let rows = sqlx::query!(
        r#"
        SELECT id, title, author, image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, created_at, is_private FROM wallpapers
        WHERE author = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        author_name,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    let results: Vec<Wallpaper> = rows.into_iter().map(|r| Wallpaper {
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
    }).collect();
    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;
    Ok(arc_results)
}

pub async fn toggle_favorite(user_id: &str, wallpaper_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;

    let exists: Option<i32> =
        sqlx::query_scalar!("SELECT 1 as result FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2", user_id, wallpaper_id)
            .fetch_optional(pool)
            .await?.flatten();

    if exists.is_some() {
        sqlx::query!("DELETE FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2", user_id, wallpaper_id)
            .execute(pool)
            .await?;

        sqlx::query!("UPDATE wallpapers SET likes = likes - 1 WHERE id = $1", wallpaper_id)
            .execute(pool)
            .await?;

        super::cache::get_wallpaper_cache().remove(wallpaper_id).await;

        Ok(false)
    } else {
        sqlx::query!("INSERT INTO user_favorites (user_id, wallpaper_id) VALUES ($1, $2)", user_id, wallpaper_id)
            .execute(pool)
            .await?;

        sqlx::query!("UPDATE wallpapers SET likes = likes + 1 WHERE id = $1", wallpaper_id)
            .execute(pool)
            .await?;

        super::cache::get_wallpaper_cache().remove(wallpaper_id).await;

        Ok(true)
    }
}

pub async fn is_favorited(user_id: &str, wallpaper_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;
    let exists: Option<i32> =
        sqlx::query_scalar!("SELECT 1 as result FROM user_favorites WHERE user_id = $1 AND wallpaper_id = $2", user_id, wallpaper_id)
            .fetch_optional(pool)
            .await?.flatten();
    Ok(exists.is_some())
}

pub async fn increment_download(id: &str, ip: &str) -> anyhow::Result<()> {
    let cache = super::cache::get_download_rate_limit_cache();
    let key = format!("{}:{}", ip, id);
    if cache.get(&key).await.is_some() {
        return Ok(());
    }
    cache.insert(key, true).await;

    let pool = get_pool()?;
    sqlx::query!("UPDATE wallpapers SET downloads = downloads + 1 WHERE id = $1", id)
        .execute(pool)
        .await?;

    super::cache::get_wallpaper_cache().remove(id).await;

    Ok(())
}

pub async fn search_wallpapers_db(
    query: &str,
    page: u32,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache = get_wallpaper_list_cache();
    let cache_key = format!("search:{}:{}:{}:{:?}", query, page, limit, filters);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(cached);
    }

    let pool = get_pool()?;

    let offset = page * limit;

    let mut q = sqlx::QueryBuilder::new("SELECT * FROM wallpapers WHERE search_vector @@ websearch_to_tsquery('english', ");
    q.push_bind(query);
    q.push(")");
    
    apply_filters(&mut q, &filters);

    match filters.sort.as_str() {
        "downloads" => {
            q.push(" ORDER BY downloads DESC, ts_rank(search_vector, websearch_to_tsquery('english', ");
            q.push_bind(query);
            q.push(")) DESC");
        },
        "rating" => {
            q.push(" ORDER BY likes DESC, ts_rank(search_vector, websearch_to_tsquery('english', ");
            q.push_bind(query);
            q.push(")) DESC");
        },
        "date" => { q.push(" ORDER BY created_at DESC"); },
        _ => {
            q.push(" ORDER BY ts_rank(search_vector, websearch_to_tsquery('english', ");
            q.push_bind(query);
            q.push(")) DESC");
        },
    }

    q.push(" LIMIT ");
    q.push_bind(limit as i64);
    q.push(" OFFSET ");
    q.push_bind(offset as i64);

    let rows = q.build().fetch_all(pool).await?;

    let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;
    Ok(arc_results)
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

        super::cache::get_wallpaper_cache().remove(wallpaper_id).await;
    }

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
    
    super::cache::get_wallpaper_cache().remove(id).await;
    super::cache::get_wallpaper_list_cache().invalidate_all();
    
    let storage_path = super::files::get_storage_path();
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_master.jpg", id))).await;
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_master.avif", id))).await;
    let _ = tokio::fs::remove_file(storage_path.join(format!("{}_thumb.jpg", id))).await;
    
    Ok(())
}

pub async fn get_public_uploads(
    author_name: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache = get_wallpaper_list_cache();
    let cache_key = format!("public_uploads:{}:{}:{}", author_name, page, limit);
    if let Some(cached) = cache.get(&cache_key).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let offset = page * limit;
    let rows = sqlx::query!(
        r#"
        SELECT id, title, author, image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, created_at, is_private FROM wallpapers
        WHERE author = $1 AND is_private = false
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        author_name,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    let results: Vec<Wallpaper> = rows.into_iter().map(|r| Wallpaper {
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
    }).collect();
    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;
    Ok(arc_results)
}

pub async fn get_creator_analytics_db(author_name: &str) -> anyhow::Result<crate::CreatorAnalytics> {
    let pool = get_pool()?;

    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total_uploads!",
            COALESCE(SUM(likes), 0) as "total_likes!",
            COALESCE(SUM(downloads), 0) as "total_downloads!"
        FROM wallpapers
        WHERE author = $1
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

    let u_stats = sqlx::query!("SELECT COUNT(*) as user_count FROM users").fetch_one(pool).await?;

    Ok(crate::AdminStats {
        total_users: u_stats.user_count.unwrap_or(0) as u32,
        total_wallpapers: w_stats.wp_count.unwrap_or(0) as u32,
        total_downloads: w_stats.total_downloads.unwrap_or(0) as u32,
        total_likes: w_stats.total_likes.unwrap_or(0) as u32,
    })
}

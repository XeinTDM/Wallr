use crate::storage::get_pool;
use crate::Wallpaper;
use super::core::map_wallpaper_row;
use crate::storage::cache::get_wallpaper_list_cache;

fn apply_filters(q: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>, filters: &crate::FilterOptions) {
    q.push(" AND w.is_private = false");

    if !filters.resolution.is_empty() {
        match filters.resolution.as_str() {
            "4k" => {
                q.push(" AND w.width >= 3840 AND w.height >= 2160");
            }
            "8k" => {
                q.push(" AND w.width >= 7680 AND w.height >= 4320");
            }
            "hd" => {
                q.push(" AND w.width >= 1920 AND w.height >= 1080");
            }
            _ => {}
        }
    }
    if !filters.aspect_ratio.is_empty() {
        match filters.aspect_ratio.as_str() {
            "ultrawide" => {
                q.push(" AND (w.width::float / w.height::float) >= 2.3");
            }
            "desktop" => {
                q.push(" AND (w.width::float / w.height::float) >= 1.3 AND (w.width::float / w.height::float) < 2.3");
            }
            "mobile" => {
                q.push(" AND (w.width::float / w.height::float) < 1.0");
            }
            _ => {}
        }
    }
    if !filters.color.is_empty() {
        q.push(" AND w.primary_colors ? ");
        q.push_bind(filters.color.clone());
    }
    if !filters.ai_filter.is_empty() {
        match filters.ai_filter.as_str() {
            "hide" => {
                q.push(" AND NOT (w.tags @> '[\"ai\"]')");
            }
            "only" => {
                q.push(" AND w.tags @> '[\"ai\"]'");
            }
            _ => {}
        }
    }
    if !filters.timeframe.is_empty() {
        match filters.timeframe.as_str() {
            "daily" => {
                q.push(" AND w.created_at >= NOW() - INTERVAL '1 day'");
            }
            "weekly" => {
                q.push(" AND w.created_at >= NOW() - INTERVAL '7 days'");
            }
            "monthly" => {
                q.push(" AND w.created_at >= NOW() - INTERVAL '30 days'");
            }
            "yearly" => {
                q.push(" AND w.created_at >= NOW() - INTERVAL '1 year'");
            }
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

    let cursor_cache = crate::storage::cache::get_cursor_cache();
    let prev_cursor_key = if page > 0 {
        Some(format!("cursor:all:{}:{}:{:?}", page - 1, limit, filters))
    } else {
        None
    };

    let mut cursor = None;
    if let Some(key) = &prev_cursor_key {
        cursor = cursor_cache.get(key).await;
    }

    let pool = get_pool()?;
    let mut q = sqlx::QueryBuilder::new("SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE 1=1");
    apply_filters(&mut q, &filters);

    let mut use_offset = false;

    if let Some(c) = cursor {
        let parts: Vec<&str> = c.split(',').collect();
        if parts.len() == 2 {
            let val = parts[0];
            let id = parts[1];
            match filters.sort.as_str() {
                "downloads" => {
                    let downloads: i64 = val.parse().unwrap_or(0);
                    q.push(" AND (w.downloads, w.id) < (");
                    q.push_bind(downloads);
                    q.push(", ");
                    q.push_bind(id.to_string());
                    q.push(")");
                }
                "rating" => {
                    let likes: i64 = val.parse().unwrap_or(0);
                    q.push(" AND (w.likes, w.id) < (");
                    q.push_bind(likes);
                    q.push(", ");
                    q.push_bind(id.to_string());
                    q.push(")");
                }
                _ => {
                    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val) {
                        q.push(" AND (w.created_at, w.id) < (");
                        q.push_bind(date.with_timezone(&chrono::Utc));
                        q.push(", ");
                        q.push_bind(id.to_string());
                        q.push(")");
                    }
                }
            }
        }
    } else if page > 0 {
        use_offset = true;
    }

    match filters.sort.as_str() {
        "downloads" => {
            q.push(" ORDER BY w.downloads DESC, w.id DESC");
        }
        "rating" => {
            q.push(" ORDER BY w.likes DESC, w.id DESC");
        }
        _ => {
            q.push(" ORDER BY w.created_at DESC, w.id DESC");
        }
    }

    q.push(" LIMIT ");
    q.push_bind((limit + 1) as i64);

    if use_offset {
        q.push(" OFFSET ");
        q.push_bind((page * limit) as i64);
    }

    let rows = q.build().fetch_all(pool).await?;
    let mut results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
    
    let mut next_cursor = None;
    if results.len() > limit as usize {
        results.pop(); // Remove the extra item
        if let Some(last) = results.last() {
            match filters.sort.as_str() {
                "downloads" => {
                    next_cursor = Some(format!("{},{}", last.downloads, last.id));
                }
                "rating" => {
                    next_cursor = Some(format!("{},{}", last.likes, last.id));
                }
                _ => {
                    next_cursor = Some(format!("{},{}", last.created_at.to_rfc3339(), last.id));
                }
            }
        }
    }

    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;

    if let Some(nc) = next_cursor {
        let current_cursor_key = format!("cursor:all:{}:{}:{:?}", page, limit, filters);
        cursor_cache.insert(current_cursor_key, nc).await;
    }

    Ok(arc_results)
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

    let cursor_cache = crate::storage::cache::get_cursor_cache();
    let prev_cursor_key = if page > 0 {
        Some(format!("cursor:tag:{}:{}:{}:{:?}", tag, page - 1, limit, filters))
    } else {
        None
    };

    let mut cursor = None;
    if let Some(key) = &prev_cursor_key {
        cursor = cursor_cache.get(key).await;
    }

    let pool = get_pool()?;
    let tag_json = serde_json::json!(tag);

    let mut q = sqlx::QueryBuilder::new("SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE w.tags @> ");
    q.push_bind(tag_json);
    apply_filters(&mut q, &filters);

    let mut use_offset = false;

    if let Some(c) = cursor {
        let parts: Vec<&str> = c.split(',').collect();
        if parts.len() == 2 {
            let val = parts[0];
            let id = parts[1];
            match filters.sort.as_str() {
                "downloads" => {
                    let downloads: i64 = val.parse().unwrap_or(0);
                    q.push(" AND (w.downloads, w.id) < (");
                    q.push_bind(downloads);
                    q.push(", ");
                    q.push_bind(id.to_string());
                    q.push(")");
                }
                "rating" => {
                    let likes: i64 = val.parse().unwrap_or(0);
                    q.push(" AND (w.likes, w.id) < (");
                    q.push_bind(likes);
                    q.push(", ");
                    q.push_bind(id.to_string());
                    q.push(")");
                }
                _ => {
                    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val) {
                        q.push(" AND (w.created_at, w.id) < (");
                        q.push_bind(date.with_timezone(&chrono::Utc));
                        q.push(", ");
                        q.push_bind(id.to_string());
                        q.push(")");
                    }
                }
            }
        }
    } else if page > 0 {
        use_offset = true;
    }

    match filters.sort.as_str() {
        "downloads" => {
            q.push(" ORDER BY w.downloads DESC, w.id DESC");
        }
        "rating" => {
            q.push(" ORDER BY w.likes DESC, w.id DESC");
        }
        _ => {
            q.push(" ORDER BY w.created_at DESC, w.id DESC");
        }
    }

    q.push(" LIMIT ");
    q.push_bind((limit + 1) as i64);

    if use_offset {
        q.push(" OFFSET ");
        q.push_bind((page * limit) as i64);
    }

    let rows = q.build().fetch_all(pool).await?;

    let mut results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
    
    let mut next_cursor = None;
    if results.len() > limit as usize {
        results.pop(); // Remove the extra item
        if let Some(last) = results.last() {
            match filters.sort.as_str() {
                "downloads" => {
                    next_cursor = Some(format!("{},{}", last.downloads, last.id));
                }
                "rating" => {
                    next_cursor = Some(format!("{},{}", last.likes, last.id));
                }
                _ => {
                    next_cursor = Some(format!("{},{}", last.created_at.to_rfc3339(), last.id));
                }
            }
        }
    }

    let arc_results = std::sync::Arc::new(results);
    cache.insert(cache_key, arc_results.clone()).await;

    if let Some(nc) = next_cursor {
        let current_cursor_key = format!("cursor:tag:{}:{}:{}:{:?}", tag, page, limit, filters);
        cursor_cache.insert(current_cursor_key, nc).await;
    }

    Ok(arc_results)
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

    let mut embed_opt = None;
    if !query.is_empty()
        && let Some(tagger) = crate::ai::TAGGER.get()
            && let Ok(embed) = tagger.get_text_embedding(query) {
                embed_opt = Some(pgvector::Vector::from(embed));
            }

    let mut q = sqlx::QueryBuilder::new("SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE 1=1");

    if embed_opt.is_none() && !query.is_empty() {
        q.push(" AND w.search_vector @@ websearch_to_tsquery('english', ");
        q.push_bind(query);
        q.push(")");
    }

    apply_filters(&mut q, &filters);

    if let Some(embed) = embed_opt {
        q.push(" ORDER BY w.embedding <=> ");
        q.push_bind(embed);
    } else if !query.is_empty() {
        match filters.sort.as_str() {
            "downloads" => {
                q.push(" ORDER BY w.downloads DESC, ts_rank(w.search_vector, websearch_to_tsquery('english', ");
                q.push_bind(query);
                q.push(")) DESC");
            }
            "rating" => {
                q.push(
                    " ORDER BY w.likes DESC, ts_rank(w.search_vector, websearch_to_tsquery('english', ",
                );
                q.push_bind(query);
                q.push(")) DESC");
            }
            "date" => {
                q.push(" ORDER BY w.created_at DESC");
            }
            _ => {
                q.push(" ORDER BY ts_rank(w.search_vector, websearch_to_tsquery('english', ");
                q.push_bind(query);
                q.push(")) DESC");
            }
        }
    } else {
        match filters.sort.as_str() {
            "downloads" => {
                q.push(" ORDER BY w.downloads DESC");
            }
            "rating" => {
                q.push(" ORDER BY w.likes DESC");
            }
            _ => {
                q.push(" ORDER BY w.created_at DESC");
            }
        }
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

pub async fn get_similar_wallpapers_db(
    id: &str,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let pool = get_pool()?;

    // First get the embedding of the target wallpaper
    let target = sqlx::query!(
        "SELECT embedding as \"embedding?: pgvector::Vector\" FROM wallpapers WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    let embedding: Option<pgvector::Vector> = match target {
        Some(r) => r.embedding,
        None => return Ok(std::sync::Arc::new(vec![])),
    };

    if let Some(embed) = embedding {
        let limit_i64 = limit as i64;

        let collab_rows = sqlx::query!(
            r#"
            WITH recent_favs AS (
                SELECT user_id FROM user_favorites WHERE wallpaper_id = $1 LIMIT 100
            ),
            collab AS (
                SELECT f2.wallpaper_id, COUNT(*) as collab_score
                FROM recent_favs f1
                JOIN user_favorites f2 ON f1.user_id = f2.user_id
                WHERE f2.wallpaper_id != $1
                GROUP BY f2.wallpaper_id
            ),
            recent_downs AS (
                SELECT user_id FROM user_downloads WHERE wallpaper_id = $1 ORDER BY downloaded_at DESC LIMIT 100
            ),
            collab_down AS (
                SELECT d2.wallpaper_id, COUNT(*) as collab_score
                FROM recent_downs d1
                JOIN user_downloads d2 ON d1.user_id = d2.user_id
                WHERE d2.wallpaper_id != $1
                GROUP BY d2.wallpaper_id
            )
            SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, w.thumbnail_url, w.tags as "tags: sqlx::types::Json<Vec<String>>", w.primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live,
                   (COALESCE(c.collab_score, 0) * 2 + COALESCE(d.collab_score, 0)) as "total_score!"
            FROM wallpapers w
            JOIN users u ON w.author_id = u.id
            LEFT JOIN collab c ON w.id = c.wallpaper_id
            LEFT JOIN collab_down d ON w.id = d.wallpaper_id
            WHERE w.id != $1 AND w.is_private = false AND (c.collab_score > 0 OR d.collab_score > 0)
            ORDER BY (COALESCE(c.collab_score, 0) * 2 + COALESCE(d.collab_score, 0)) DESC
            LIMIT 50
            "#,
            id
        ).fetch_all(pool).await.unwrap_or_default();

        let visual_rows = sqlx::query!(
            r#"
            SELECT w.id, w.title, w.author_id, u.name as "author_name!", w.image_url, thumbnail_url, tags as "tags: sqlx::types::Json<Vec<String>>", primary_colors as "primary_colors: sqlx::types::Json<Vec<String>>", width, height, size_bytes, likes, downloads, w.created_at, is_private, is_live 
            FROM wallpapers w
            JOIN users u ON w.author_id = u.id
            WHERE w.id != $1 AND w.is_private = false
            ORDER BY w.embedding <=> $2 
            LIMIT $3
            "#,
            id,
            embed as _,
            limit_i64
        ).fetch_all(pool).await.unwrap_or_default();

        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for r in collab_rows {
            if !seen.contains(&r.id) {
                seen.insert(r.id.clone());
                results.push(Wallpaper {
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
                });
            }
        }

        for r in visual_rows {
            if !seen.contains(&r.id) {
                seen.insert(r.id.clone());
                results.push(Wallpaper {
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
                });
            }
        }

        results.truncate(limit as usize);
        return Ok(std::sync::Arc::new(results));
    }

    Ok(std::sync::Arc::new(vec![]))
}
use super::core::map_wallpaper_row;
use crate::Wallpaper;
use crate::storage::get_pool;

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

    if filters.safe_search {
        q.push(" AND NOT (w.tags @> '[\"nsfw\"]')");
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
    cursor: Option<String>,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache_key = format!("all:{:?}:{}:{:?}", cursor, limit, filters);

    let cursor_cloned = cursor.clone();
    let filters_cloned = filters.clone();
    crate::storage::wallpapers::core::fetch_and_cache_wallpaper_list(cache_key, || async move {
        let pool = get_pool()?;
        let mut q = sqlx::QueryBuilder::new(
            "SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE 1=1",
        );
        apply_filters(&mut q, &filters_cloned);

        if let Some(c) = &cursor_cloned {
            let parts: Vec<&str> = c.split(',').collect();
            if parts.len() == 2 {
                let val = parts[0];
                let id = parts[1];
                match filters_cloned.sort.as_str() {
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
        }

        match filters_cloned.sort.as_str() {
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
        q.push_bind(limit as i64);

        let rows = q.build().fetch_all(pool).await?;
        let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();

        Ok(results)
    }).await
}

pub async fn get_wallpapers_by_tag(
    tag: &str,
    cursor: Option<String>,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache_key = format!("tag:{}:{:?}:{}:{:?}", tag, cursor, limit, filters);

    let tag_cloned = tag.to_string();
    let cursor_cloned = cursor.clone();
    let filters_cloned = filters.clone();

    crate::storage::wallpapers::core::fetch_and_cache_wallpaper_list(cache_key, || async move {
        let pool = get_pool()?;
        let tag_json = serde_json::json!(tag_cloned);

        let mut q = sqlx::QueryBuilder::new(
            "SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE w.tags @> ",
        );
        q.push_bind(tag_json);
        apply_filters(&mut q, &filters_cloned);

        if let Some(c) = &cursor_cloned {
            let parts: Vec<&str> = c.split(',').collect();
            if parts.len() == 2 {
                let val = parts[0];
                let id = parts[1];
                match filters_cloned.sort.as_str() {
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
        }

        match filters_cloned.sort.as_str() {
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
        q.push_bind(limit as i64);

        let rows = q.build().fetch_all(pool).await?;

        let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
        Ok(results)
    }).await
}

pub async fn search_wallpapers_db(
    query: &str,
    cursor: Option<String>,
    limit: u32,
    filters: crate::FilterOptions,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let cache_key = format!("search:{}:{:?}:{}:{:?}", query, cursor, limit, filters);
    let query_cloned = query.to_string();
    let cursor_cloned = cursor.clone();
    let filters_cloned = filters.clone();

    crate::storage::wallpapers::core::fetch_and_cache_wallpaper_list(cache_key, || async move {
        let pool = get_pool()?;

        let mut embed_opt = None;
        if !query_cloned.is_empty()
            && let Some(tagger) = crate::ai::TAGGER.get() {
                let q_clone_for_ai = query_cloned.clone();
                let embed_res = tokio::task::spawn_blocking(move || {
                    tagger.get_text_embedding(&q_clone_for_ai)
                }).await;
                if let Ok(Ok(embed)) = embed_res {
                    embed_opt = Some(pgvector::Vector::from(embed));
                }
            }

        let mut q = sqlx::QueryBuilder::new(
            "SELECT w.id, w.title, w.author_id, u.name as author_name, w.image_url, w.thumbnail_url, w.tags, w.primary_colors, w.width, w.height, w.size_bytes, w.likes, w.downloads, w.created_at, w.is_private, w.is_live, w.phash FROM wallpapers w JOIN users u ON w.author_id = u.id WHERE 1=1",
        );

        if embed_opt.is_none() && !query_cloned.is_empty() {
            q.push(" AND w.search_vector @@ websearch_to_tsquery('english', ");
            q.push_bind(&query_cloned);
            q.push(")");
        }

        apply_filters(&mut q, &filters_cloned);

        if let Some(c) = &cursor_cloned {
            let parts: Vec<&str> = c.split(',').collect();
            if parts.len() == 2 {
                let val = parts[0];
                let id = parts[1];
                if embed_opt.is_none() && !query_cloned.is_empty() {
                    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val) {
                        q.push(" AND (w.created_at, w.id) < (");
                        q.push_bind(date.with_timezone(&chrono::Utc));
                        q.push(", ");
                        q.push_bind(id.to_string());
                        q.push(")");
                    }
                } else {
                    match filters_cloned.sort.as_str() {
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
            }
        }

        if let Some(embed) = embed_opt {
            q.push(" ORDER BY w.embedding <=> ");
            q.push_bind(embed);
        } else if !query_cloned.is_empty() {
            match filters_cloned.sort.as_str() {
                "downloads" => {
                    q.push(" ORDER BY w.downloads DESC, ts_rank(w.search_vector, websearch_to_tsquery('english', ");
                    q.push_bind(&query_cloned);
                    q.push(")) DESC, w.id DESC");
                }
                "rating" => {
                    q.push(
                        " ORDER BY w.likes DESC, ts_rank(w.search_vector, websearch_to_tsquery('english', ",
                    );
                    q.push_bind(&query_cloned);
                    q.push(")) DESC, w.id DESC");
                }
                "date" => {
                    q.push(" ORDER BY w.created_at DESC, w.id DESC");
                }
                _ => {
                    q.push(" ORDER BY ts_rank(w.search_vector, websearch_to_tsquery('english', ");
                    q.push_bind(&query_cloned);
                    q.push(")) DESC, w.created_at DESC, w.id DESC");
                }
            }
        } else {
            match filters_cloned.sort.as_str() {
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
        }

        q.push(" LIMIT ");
        q.push_bind(limit as i64);

        let rows = q.build().fetch_all(pool).await?;

        let results: Vec<Wallpaper> = rows.into_iter().map(map_wallpaper_row).collect();
        Ok(results)
    }).await
}

pub async fn get_similar_wallpapers_db(
    id: &str,
    limit: u32,
) -> anyhow::Result<std::sync::Arc<Vec<Wallpaper>>> {
    let pool = get_pool()?;

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
                CROSS JOIN LATERAL (
                    SELECT wallpaper_id FROM user_favorites
                    WHERE user_id = f1.user_id AND wallpaper_id != $1
                    LIMIT 50
                ) f2
                GROUP BY f2.wallpaper_id
            ),
            recent_downs AS (
                SELECT user_id FROM user_downloads WHERE wallpaper_id = $1 ORDER BY downloaded_at DESC LIMIT 100
            ),
            collab_down AS (
                SELECT d2.wallpaper_id, COUNT(*) as collab_score
                FROM recent_downs d1
                CROSS JOIN LATERAL (
                    SELECT wallpaper_id FROM user_downloads
                    WHERE user_id = d1.user_id AND wallpaper_id != $1
                    ORDER BY downloaded_at DESC LIMIT 50
                ) d2
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
                    description: None,
                    source_url: None,
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
                    description: None,
                    source_url: None,
                });
            }
        }

        results.truncate(limit as usize);
        return Ok(std::sync::Arc::new(results));
    }

    Ok(std::sync::Arc::new(vec![]))
}

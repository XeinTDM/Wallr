use super::get_pool;
use crate::WallpaperComment;

pub async fn get_comments_db(
    wallpaper_id: &str,
    page: u32,
    limit: u32,
) -> anyhow::Result<Vec<WallpaperComment>> {
    let pool = get_pool()?;

    let offset = (page * limit) as i64;
    let limit = limit as i64;

    let rows = sqlx::query!(
        r#"
        SELECT c.id, c.wallpaper_id, c.user_id, c.content, c.created_at, u.name as user_name, u.pfp_url as user_pfp, c.parent_id, c.is_pinned, c.is_hidden, c.is_edited
        FROM wallpaper_comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.wallpaper_id = $1
        ORDER BY c.is_pinned DESC, c.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        wallpaper_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let comments = rows
        .into_iter()
        .map(|r| WallpaperComment {
            id: r.id,
            wallpaper_id: r.wallpaper_id,
            user_id: r.user_id,
            user_name: r.user_name,
            user_pfp: r.user_pfp,
            content: r.content,
            created_at: r.created_at.to_rfc3339(),
            parent_id: r.parent_id,
            is_pinned: r.is_pinned,
            is_hidden: r.is_hidden,
            is_edited: r.is_edited,
        })
        .collect();

    Ok(comments)
}

pub async fn add_comment_db(comment: &WallpaperComment) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let dt = chrono::DateTime::parse_from_rfc3339(&comment.created_at)
        .unwrap_or_else(|_| chrono::Utc::now().into())
        .with_timezone(&chrono::Utc);

    sqlx::query!(
        r#"
        INSERT INTO wallpaper_comments (id, wallpaper_id, user_id, content, created_at, parent_id, is_pinned, is_hidden, is_edited)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        comment.id,
        comment.wallpaper_id,
        comment.user_id,
        comment.content,
        dt,
        comment.parent_id,
        comment.is_pinned,
        comment.is_hidden,
        comment.is_edited
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn check_duplicate_comment(
    wallpaper_id: &str,
    user_id: &str,
    content: &str,
) -> anyhow::Result<bool> {
    let pool = get_pool()?;

    let result = sqlx::query!(
        r#"
        SELECT id FROM wallpaper_comments 
        WHERE wallpaper_id = $1 AND user_id = $2 AND content = $3
        LIMIT 1
        "#,
        wallpaper_id,
        user_id,
        content
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

pub async fn check_comment_rate_limit(user_id: &str) -> anyhow::Result<()> {
    let cache = super::cache::get_comment_rate_limit_cache();
    let count = cache.get(user_id).await.unwrap_or(0);
    if count >= 5 {
        anyhow::bail!("api_err_comment_rate_limit");
    }
    cache.insert(user_id.to_string(), count + 1).await;
    Ok(())
}

pub async fn delete_comment_db(
    comment_id: &str,
    user_id: &str,
    user_name: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let result = sqlx::query!(
        r#"
        DELETE FROM wallpaper_comments 
        WHERE id = $1 AND (
            user_id = $2 OR 
            wallpaper_id IN (SELECT id FROM wallpapers WHERE author_id = (SELECT id FROM users WHERE name = $3))
        )
        "#,
        comment_id,
        user_id,
        user_name
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("api_err_comment_not_found_or_denied"));
    }

    Ok(())
}

pub async fn update_comment_db(
    comment_id: &str,
    user_id: &str,
    new_content: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    
    // First fetch the old content for history
    let old_comment = sqlx::query!("SELECT content FROM wallpaper_comments WHERE id = $1 AND user_id = $2", comment_id, user_id)
        .fetch_optional(pool)
        .await?;
        
    if let Some(old) = old_comment {
        let history_id = uuid::Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO comment_edit_history (id, comment_id, previous_content) VALUES ($1, $2, $3)",
            history_id,
            comment_id,
            old.content
        ).execute(pool).await?;
    }

    let result = sqlx::query!(
        "UPDATE wallpaper_comments SET content = $1, is_edited = true WHERE id = $2 AND user_id = $3",
        new_content,
        comment_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("api_err_comment_not_found_or_denied"));
    }
    Ok(())
}

pub async fn pin_comment_db(comment_id: &str, user_id: &str, pin: bool) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let result = sqlx::query!(
        r#"
        UPDATE wallpaper_comments 
        SET is_pinned = $1 
        WHERE id = $2 AND wallpaper_id IN (SELECT id FROM wallpapers WHERE author_id = $3)
        "#,
        pin,
        comment_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("api_err_comment_not_found_or_denied"));
    }
    Ok(())
}

pub async fn hide_comment_db(comment_id: &str, user_id: &str, hide: bool) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let result = sqlx::query!(
        r#"
        UPDATE wallpaper_comments 
        SET is_hidden = $1 
        WHERE id = $2 AND wallpaper_id IN (SELECT id FROM wallpapers WHERE author_id = $3)
        "#,
        hide,
        comment_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("api_err_comment_not_found_or_denied"));
    }
    Ok(())
}

pub async fn toggle_wallpaper_comments_db(wallpaper_id: &str, user_id: &str, disable: bool) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let result = sqlx::query!(
        "UPDATE wallpapers SET comments_disabled = $1 WHERE id = $2 AND author_id = $3",
        disable,
        wallpaper_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("api_err_wallpaper_not_found_or_denied"));
    }
    Ok(())
}

pub async fn report_comment_db(comment_id: &str, reporter_id: &str, reporter_name: &str, reason: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO reported_comments (id, comment_id, reporter_id, reporter_name, reason) VALUES ($1, $2, $3, $4, $5)",
        id,
        comment_id,
        reporter_id,
        reporter_name,
        reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_comment_edit_history_db(comment_id: &str) -> anyhow::Result<Vec<crate::models::CommentEditHistory>> {
    let pool = get_pool()?;
    let rows = sqlx::query!("SELECT id, comment_id, previous_content, edited_at FROM comment_edit_history WHERE comment_id = $1 ORDER BY edited_at DESC", comment_id)
        .fetch_all(pool)
        .await?;
        
    let history = rows.into_iter().map(|r| crate::models::CommentEditHistory {
        id: r.id,
        comment_id: r.comment_id,
        previous_content: r.previous_content,
        edited_at: r.edited_at,
    }).collect();
    
    Ok(history)
}

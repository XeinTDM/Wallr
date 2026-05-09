use crate::WallpaperComment;
use super::get_pool;

pub async fn get_comments_db(wallpaper_id: &str, page: u32, limit: u32) -> anyhow::Result<Vec<WallpaperComment>> {
    let pool = get_pool()?;
    
    let offset = (page * limit) as i64;
    let limit = limit as i64;

    let rows = sqlx::query!(
        r#"
        SELECT c.id, c.wallpaper_id, c.user_id, c.content, c.created_at, u.name as user_name, u.pfp_url as user_pfp
        FROM wallpaper_comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.wallpaper_id = $1
        ORDER BY c.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        wallpaper_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let comments = rows.into_iter().map(|r| WallpaperComment {
        id: r.id,
        wallpaper_id: r.wallpaper_id,
        user_id: r.user_id,
        user_name: r.user_name,
        user_pfp: r.user_pfp,
        content: r.content,
        created_at: r.created_at.to_rfc3339(),
    }).collect();

    Ok(comments)
}

pub async fn add_comment_db(comment: &WallpaperComment) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let dt = chrono::DateTime::parse_from_rfc3339(&comment.created_at)
        .unwrap_or_else(|_| chrono::Utc::now().into())
        .with_timezone(&chrono::Utc);

    sqlx::query!(
        r#"
        INSERT INTO wallpaper_comments (id, wallpaper_id, user_id, content, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        comment.id,
        comment.wallpaper_id,
        comment.user_id,
        comment.content,
        dt
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn check_duplicate_comment(wallpaper_id: &str, user_id: &str, content: &str) -> anyhow::Result<bool> {
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
        anyhow::bail!("You are posting comments too quickly. Please wait a minute.");
    }
    cache.insert(user_id.to_string(), count + 1).await;
    Ok(())
}

pub async fn delete_comment_db(comment_id: &str, user_id: &str, user_name: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let result = sqlx::query!(
        r#"
        DELETE FROM wallpaper_comments 
        WHERE id = $1 AND (
            user_id = $2 OR 
            wallpaper_id IN (SELECT id FROM wallpapers WHERE author = $3)
        )
        "#,
        comment_id,
        user_id,
        user_name
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Comment not found or permission denied"));
    }

    Ok(())
}

pub async fn update_comment_db(comment_id: &str, user_id: &str, new_content: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let result = sqlx::query!(
        "UPDATE wallpaper_comments SET content = $1 WHERE id = $2 AND user_id = $3",
        new_content, comment_id, user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Comment not found or permission denied"));
    }
    Ok(())
}

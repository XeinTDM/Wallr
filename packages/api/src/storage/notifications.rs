use super::get_pool;

pub async fn create_notification_db(user_id: &str, title: &str, message: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO notifications (id, user_id, title, message) VALUES ($1, $2, $3, $4)",
        id, user_id, title, message
    ).execute(pool).await?;
    Ok(())
}

pub async fn get_notifications_db(user_id: &str) -> anyhow::Result<Vec<crate::Notification>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        "SELECT id, title, message, is_read, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
        user_id
    ).fetch_all(pool).await?;
    
    Ok(rows.into_iter().map(|r| crate::Notification {
        id: r.id,
        title: r.title,
        message: r.message,
        is_read: r.is_read,
        created_at: r.created_at,
    }).collect())
}

pub async fn mark_notification_read_db(user_id: &str, notification_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2",
        notification_id, user_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn mark_all_notifications_read_db(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE",
        user_id
    ).execute(pool).await?;
    Ok(())
}

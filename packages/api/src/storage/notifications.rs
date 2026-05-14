use super::get_pool;

pub async fn create_notification_db(
    user_id: &str,
    title: &str,
    message: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO notifications (id, user_id, title, message) VALUES ($1, $2, $3, $4)",
        id,
        user_id,
        title,
        message
    )
    .execute(pool)
    .await?;

    // Dispatch external notifications based on user preferences
    if let Ok(user_record) = super::users::core::get_user_by_id(user_id).await {
        if let Some(record) = user_record {
            let u = record.user;
            if u.email_notifs {
                // Dispatch email
                println!("📧 DISPATCH EMAIL to {}: [{}] {}", u.email, title, message);
                // In a real app we'd use an SMTP client or Resend/SendGrid API here.
            }

            if u.push_notifs {
                if let Ok(pool) = get_pool() {
                    let subs = sqlx::query!(
                        "SELECT endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = $1",
                        user_id
                    ).fetch_all(pool).await.unwrap_or_default();

                    for sub in subs {
                        // Dispatch push
                        println!("📱 DISPATCH PUSH to {}: [{}] {}", sub.endpoint, title, message);
                        // In a real app we'd use a WebPush library here.
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn add_push_subscription_db(user_id: &str, endpoint: &str, p256dh: &str, auth: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    
    // Check if endpoint already exists for this user to avoid duplicates
    let existing = sqlx::query!("SELECT id FROM push_subscriptions WHERE user_id = $1 AND endpoint = $2", user_id, endpoint)
        .fetch_optional(pool).await?;
        
    if existing.is_none() {
        sqlx::query!(
            "INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth) VALUES ($1, $2, $3, $4, $5)",
            id, user_id, endpoint, p256dh, auth
        ).execute(pool).await?;
    }
    
    Ok(())
}

pub async fn remove_push_subscription_db(user_id: &str, endpoint: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("DELETE FROM push_subscriptions WHERE user_id = $1 AND endpoint = $2", user_id, endpoint)
        .execute(pool).await?;
    Ok(())
}

pub async fn get_notifications_db(user_id: &str) -> anyhow::Result<Vec<crate::Notification>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        "SELECT id, title, message, is_read, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
        user_id
    ).fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|r| crate::Notification {
            id: r.id,
            title: r.title,
            message: r.message,
            is_read: r.is_read,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn mark_notification_read_db(user_id: &str, notification_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2",
        notification_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_all_notifications_read_db(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE",
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

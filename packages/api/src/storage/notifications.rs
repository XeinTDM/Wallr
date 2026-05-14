use super::get_pool;
use lettre::{Message, AsyncTransport, AsyncSmtpTransport, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;
use web_push_native::{WebPushBuilder, Auth, p256::PublicKey, jwt_simple::algorithms::ES256KeyPair};

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
            let title_clone = title.to_string();
            let message_clone = message.to_string();
            let user_id_clone = user_id.to_string();
            let email_addr = u.email.clone();
            
            // Dispatch in background to avoid blocking the DB response
            tokio::spawn(async move {
                if u.email_notifs && !email_addr.is_empty() {
                    let email = Message::builder()
                        .from("noreply@wallr.example.com".parse().unwrap())
                        .to(email_addr.parse().unwrap())
                        .subject(&title_clone)
                        .body(message_clone.clone())
                        .unwrap();

                    // Ideally credentials come from env
                    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".into());
                    
                    if let Ok(mailer) = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host) {
                        let mailer = mailer.build();
                        match mailer.send(email).await {
                            Ok(_) => println!("📧 DISPATCHED EMAIL to {}", email_addr),
                            Err(e) => eprintln!("Failed to send email to {}: {:?}", email_addr, e),
                        }
                    }
                }

                if u.push_notifs {
                    if let Ok(pool) = get_pool() {
                        let subs = sqlx::query!(
                            "SELECT endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = $1",
                            user_id_clone
                        ).fetch_all(pool).await.unwrap_or_default();

                        if !subs.is_empty() {
                            // In production, vapid private key should come from env
                            let vapid_priv = std::env::var("VAPID_PRIVATE_KEY").unwrap_or_else(|_| "BIs7A-xR7HkF1x4H_yU9Y-hR9xP8yW8-Z7xP8yW8-Z7=".into());
                            use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
                            
                            if let Ok(vapid_bytes) = URL_SAFE_NO_PAD.decode(&vapid_priv) {
                                if let Ok(key_pair) = ES256KeyPair::from_bytes(&vapid_bytes) {
                                    let client = reqwest::Client::new();
                                    
                                    for sub in subs {
                                        if let (Ok(p256dh), Ok(auth_bytes)) = (URL_SAFE_NO_PAD.decode(&sub.p256dh), URL_SAFE_NO_PAD.decode(&sub.auth)) {
                                            if let (Ok(endpoint_url), Ok(pub_key)) = (sub.endpoint.parse(), PublicKey::from_sec1_bytes(&p256dh)) {
                                                let auth = Auth::clone_from_slice(&auth_bytes);
                                                let builder = WebPushBuilder::new(endpoint_url, pub_key, auth)
                                                    .with_vapid(&key_pair, "mailto:admin@wallr.example.com");
                                                    
                                                let payload = serde_json::json!({
                                                    "title": title_clone,
                                                    "body": message_clone,
                                                    "icon": "/assets/logo.png"
                                                }).to_string().into_bytes();
                                                
                                                if let Ok(request) = builder.build(payload) {
                                                    if let Ok(reqwest_req) = reqwest::Request::try_from(request) {
                                                        match client.execute(reqwest_req).await {
                                                            Ok(_) => println!("📱 DISPATCHED PUSH to {}", sub.endpoint),
                                                            Err(e) => eprintln!("Failed to send push to {}: {:?}", sub.endpoint, e),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
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

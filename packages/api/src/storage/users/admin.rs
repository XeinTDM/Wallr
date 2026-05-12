use crate::storage::cache::get_user_cache;
use crate::storage::get_pool;

pub async fn log_audit_action_db(
    admin_id: &str,
    admin_name: &str,
    action: &str,
    target_id: &str,
    target_type: &str,
    reason: Option<&str>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let new_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO audit_logs (id, admin_id, admin_name, action, target_id, target_type, reason) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        new_id,
        admin_id,
        admin_name,
        action,
        target_id,
        target_type,
        reason
    ).execute(pool).await?;

    Ok(())
}

pub async fn get_audit_logs_db(limit: u32) -> anyhow::Result<Vec<crate::AuditLog>> {
    let pool = get_pool()?;
    let limit = limit as i64;

    let rows = sqlx::query!(
        "SELECT id, admin_id, admin_name, action, target_id, target_type, reason, created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1",
        limit
    ).fetch_all(pool).await?;

    let logs = rows
        .into_iter()
        .map(|r| crate::AuditLog {
            id: r.id,
            admin_id: r.admin_id,
            admin_name: r.admin_name,
            action: r.action,
            target_id: r.target_id,
            target_type: r.target_type,
            reason: r.reason,
            created_at: r.created_at,
        })
        .collect();

    Ok(logs)
}

pub async fn get_recent_users_db(limit: u32) -> anyhow::Result<Vec<crate::User>> {
    let pool = get_pool()?;
    let limit = limit as i64;

    let rows = sqlx::query!(
        "SELECT id, name, email, pfp_url, banner_url, bio, social_links, role, is_banned, active_playlist_id, playlist_interval_secs FROM users ORDER BY created_at DESC LIMIT $1",
        limit
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| crate::User {
            id: r.id,
            name: r.name,
            email: r.email,
            pfp_url: r.pfp_url,
            banner_url: r.banner_url,
            bio: r.bio,
            social_links: r.social_links.and_then(|v| serde_json::from_value(v).ok()),
            role: r.role,
            is_banned: r.is_banned,
            active_playlist_id: r.active_playlist_id,
            playlist_interval_secs: r.playlist_interval_secs.unwrap_or(3600),
        })
        .collect();

    Ok(users)
}

pub async fn admin_bulk_delete_users_db(
    hours_ago: u32,
    pattern: Option<&str>,
) -> anyhow::Result<u64> {
    let pool = get_pool()?;

    let hours_interval = chrono::Duration::hours(hours_ago as i64);
    let cutoff = chrono::Utc::now() - hours_interval;

    let users_to_delete: Vec<String> = if let Some(pat) = pattern {
        if pat.trim().is_empty() {
            sqlx::query_scalar!("SELECT id FROM users WHERE created_at >= $1", cutoff)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_scalar!(
                "SELECT id FROM users WHERE created_at >= $1 AND name ~ $2",
                cutoff,
                pat
            )
            .fetch_all(pool)
            .await?
        }
    } else {
        sqlx::query_scalar!("SELECT id FROM users WHERE created_at >= $1", cutoff)
            .fetch_all(pool)
            .await?
    };

    let mut deleted_count = 0;
    for user_id in &users_to_delete {
        if crate::storage::users::delete_user(user_id).await.is_ok() {
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

pub async fn admin_ban_user_db(user_id: &str, banned: bool) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE users SET is_banned = $1 WHERE id = $2",
        banned,
        user_id
    )
    .execute(pool)
    .await?;

    get_user_cache().remove(user_id).await;

    if !banned {
        let _ = crate::storage::create_notification_db(
            user_id,
            "api_notif_account_restored_title",
            "api_notif_account_restored_body",
        )
        .await;
    }

    Ok(())
}

use crate::storage::get_pool;
use crate::UserRecord;

pub async fn follow_user_db(follower_id: &str, following_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let exists = sqlx::query!(
        "SELECT 1 as id FROM user_follows WHERE follower_id = $1 AND following_id = $2",
        follower_id,
        following_id
    )
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        sqlx::query!(
            "INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2)",
            follower_id,
            following_id
        )
        .execute(pool)
        .await?;

        if let Ok(Some(follower)) = crate::storage::users::get_user_by_id(follower_id).await {
            let _ = crate::storage::create_notification_db(
                following_id,
                "api_notif_new_follower_title",
                &format!("api_notif_new_follower_body|{}", follower.user.name),
            )
            .await;
        }
    }

    Ok(())
}

pub async fn unfollow_user_db(follower_id: &str, following_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "DELETE FROM user_follows WHERE follower_id = $1 AND following_id = $2",
        follower_id,
        following_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_followers_db(
    user_id: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<UserRecord>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        "SELECT u.* FROM users u INNER JOIN user_follows f ON u.id = f.follower_id WHERE f.following_id = $1 ORDER BY f.created_at DESC LIMIT $2 OFFSET $3",
        user_id, limit, offset
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| UserRecord {
            user: crate::User {
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
                email_notifs: r.email_notifs,
                push_notifs: r.push_notifs,
                download_quality: r.download_quality,
                auto_download_avif: r.auto_download_avif,
                safe_search: r.safe_search,
                },            password_hash: r.password_hash,
            token_version: r.token_version,
        })
        .collect();

    Ok(users)
}

pub async fn get_following_db(
    user_id: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<UserRecord>> {
    let pool = get_pool()?;
    let rows = sqlx::query!(
        "SELECT u.* FROM users u INNER JOIN user_follows f ON u.id = f.following_id WHERE f.follower_id = $1 ORDER BY f.created_at DESC LIMIT $2 OFFSET $3",
        user_id, limit, offset
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| UserRecord {
            user: crate::User {
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
                email_notifs: r.email_notifs,
                push_notifs: r.push_notifs,
                download_quality: r.download_quality,
                auto_download_avif: r.auto_download_avif,
                safe_search: r.safe_search,
                },            password_hash: r.password_hash,
            token_version: r.token_version,
        })
        .collect();

    Ok(users)
}

pub async fn check_is_following_db(follower_id: &str, following_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;
    let row = sqlx::query!(
        "SELECT 1 as exists FROM user_follows WHERE follower_id = $1 AND following_id = $2",
        follower_id,
        following_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

pub async fn get_follow_counts(user_id: &str) -> anyhow::Result<(u32, u32)> {
    let pool = get_pool()?;
    let followers_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM user_follows WHERE following_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    let following_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM user_follows WHERE follower_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok((
        followers_row.count.unwrap_or(0) as u32,
        following_row.count.unwrap_or(0) as u32,
    ))
}

pub async fn get_suggested_users_db(user_id: &str, limit: i64) -> anyhow::Result<Vec<UserRecord>> {
    let pool = get_pool()?;
    
    let rows = sqlx::query!(
        "SELECT u.*, 
            (SELECT COUNT(*) FROM user_follows f WHERE f.following_id = u.id) as follower_count
         FROM users u
         WHERE u.id != $1 
         AND u.is_banned = false
         AND NOT EXISTS (
             SELECT 1 FROM user_follows uf WHERE uf.follower_id = $1 AND uf.following_id = u.id
         )
         ORDER BY follower_count DESC, u.created_at DESC
         LIMIT $2",
        user_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| UserRecord {
            user: crate::User {
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
                email_notifs: r.email_notifs,
                push_notifs: r.push_notifs,
                download_quality: r.download_quality,
                auto_download_avif: r.auto_download_avif,
                safe_search: r.safe_search,
                },            password_hash: r.password_hash,
            token_version: r.token_version,
        })
        .collect();

    Ok(users)
}

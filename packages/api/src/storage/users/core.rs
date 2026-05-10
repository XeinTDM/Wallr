use crate::storage::cache::get_user_cache;
use crate::storage::get_pool;
use crate::{User, UserRecord};

pub async fn get_user_by_email(email: &str) -> anyhow::Result<Option<UserRecord>> {
    let pool = get_pool()?;
    let row = sqlx::query!("SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| UserRecord {
        user: User {
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
        },
        password_hash: r.password_hash,
        token_version: r.token_version,
    }))
}

pub async fn get_user_by_name(name: &str) -> anyhow::Result<Option<UserRecord>> {
    let pool = get_pool()?;
    let row = sqlx::query!("SELECT * FROM users WHERE LOWER(name) = LOWER($1)", name)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| UserRecord {
        user: User {
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
        },
        password_hash: r.password_hash,
        token_version: r.token_version,
    }))
}

pub async fn get_user_by_id(id: &str) -> anyhow::Result<Option<UserRecord>> {
    let cache = get_user_cache();
    if let Some(cached) = cache.get(id).await {
        return Ok(cached);
    }

    let pool = get_pool()?;
    let row = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;

    let result = row.map(|r| UserRecord {
        user: User {
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
        },
        password_hash: r.password_hash,
        token_version: r.token_version,
    });
    cache.insert(id.to_string(), result.clone()).await;
    Ok(result)
}

pub async fn create_user(record: &UserRecord) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, pfp_url, password_hash, banner_url, token_version, bio, social_links, role, active_playlist_id, playlist_interval_secs)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        record.user.id,
        record.user.name,
        record.user.email,
        record.user.pfp_url,
        record.password_hash,
        record.user.banner_url,
        record.token_version,
        record.user.bio,
        record.user.social_links.as_ref().map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Null)),
        record.user.role,
        record.user.active_playlist_id,
        record.user.playlist_interval_secs
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_user_media(
    user_id: &str,
    media_type: &str,
    file_url: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    if media_type == "pfp" {
        sqlx::query!(
            "UPDATE users SET pfp_url = $1 WHERE id = $2",
            file_url,
            user_id
        )
        .execute(pool)
        .await?;
    } else if media_type == "banner" {
        sqlx::query!(
            "UPDATE users SET banner_url = $1 WHERE id = $2",
            file_url,
            user_id
        )
        .execute(pool)
        .await?;
    }

    get_user_cache().remove(user_id).await;

    Ok(())
}

pub async fn update_profile(
    user_id: &str,
    name: &str,
    email: &str,
    bio: Option<&str>,
    social_links: Option<&serde_json::Value>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE users SET name = $1, email = $2, bio = $3, social_links = $4 WHERE id = $5",
        name,
        email,
        bio,
        social_links,
        user_id
    )
    .execute(pool)
    .await?;

    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn delete_user(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let mut tx = pool.begin().await?;

    let user = sqlx::query!("SELECT name FROM users WHERE id = $1", user_id)
        .fetch_optional(&mut *tx)
        .await?;

    if let Some(u) = user {
        let wp_ids: Vec<String> =
            sqlx::query_scalar!("SELECT id FROM wallpapers WHERE author = $1", u.name)
                .fetch_all(&mut *tx)
                .await?;

        sqlx::query!("DELETE FROM user_favorites WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM wallpapers WHERE author = $1", u.name)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        get_user_cache().remove(user_id).await;

        let wp_cache = crate::storage::cache::get_wallpaper_cache();
        for id in wp_ids {
            wp_cache.remove(&id).await;
        }
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();
    }

    Ok(())
}

pub async fn search_users(query: &str, limit: u32) -> anyhow::Result<Vec<User>> {
    let pool = get_pool()?;

    let search_pattern = format!("%{}%", query.to_lowercase());

    let rows = sqlx::query!(
        "SELECT id, name, email, pfp_url, banner_url, bio, social_links, role, is_banned, active_playlist_id, playlist_interval_secs FROM users WHERE LOWER(name) LIKE $1 LIMIT $2",
        search_pattern,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| User {
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

pub async fn update_user_playlist(
    user_id: &str,
    collection_id: Option<&str>,
    interval_secs: Option<i32>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    get_user_cache().remove(user_id).await;

    sqlx::query!(
        "UPDATE users SET active_playlist_id = $1, playlist_interval_secs = COALESCE($2, playlist_interval_secs) WHERE id = $3",
        collection_id,
        interval_secs,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

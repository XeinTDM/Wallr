use crate::{User, UserRecord};
use super::cache::get_user_cache;
use super::get_pool;

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
        INSERT INTO users (id, name, email, pfp_url, password_hash, banner_url, token_version, bio, social_links, role)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        record.user.role
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
        sqlx::query!("UPDATE users SET pfp_url = $1 WHERE id = $2", file_url, user_id)
            .execute(pool)
            .await?;
    } else if media_type == "banner" {
        sqlx::query!("UPDATE users SET banner_url = $1 WHERE id = $2", file_url, user_id)
            .execute(pool)
            .await?;
    }

    get_user_cache().remove(user_id).await;

    Ok(())
}

pub async fn revoke_all_sessions(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("UPDATE users SET token_version = token_version + 1 WHERE id = $1", user_id)
        .execute(pool)
        .await?;
        
    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn update_password(user_id: &str, new_password_hash: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("UPDATE users SET password_hash = $1, token_version = token_version + 1 WHERE id = $2", new_password_hash, user_id)
        .execute(pool)
        .await?;
        
    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn update_profile(user_id: &str, name: &str, email: &str, bio: Option<&str>, social_links: Option<&serde_json::Value>) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("UPDATE users SET name = $1, email = $2, bio = $3, social_links = $4 WHERE id = $5", name, email, bio, social_links, user_id)
        .execute(pool)
        .await?;
        
    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn export_user_data(user_id: &str) -> anyhow::Result<String> {
    let user_record = get_user_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let favorites = super::wallpapers::get_user_favorites(user_id, 0, 1000).await?;
    let uploads = super::wallpapers::get_user_uploads(&user_record.user.name, 0, 1000).await?;

    let profile_json = serde_json::to_string_pretty(&user_record.user)?;
    let favorites_json = serde_json::to_string_pretty(&favorites)?;
    let uploads_json = serde_json::to_string_pretty(&uploads)?;

    let storage_path = super::files::get_storage_path();
    
    let temp_dir = storage_path.join("tmp");
    if !tokio::fs::try_exists(&temp_dir).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&temp_dir).await?;
    }
    
    let safe_username: String = user_record.user.name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
    let temp_filename = format!("{}_export_{}.tar.gz", safe_username, uuid::Uuid::new_v4());
    let output_path = temp_dir.join(&temp_filename);
    let output_path_str = output_path.to_string_lossy().to_string();

    let pfp_url = user_record.user.pfp_url.clone();
    let banner_url = user_record.user.banner_url.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let file = std::fs::File::create(&output_path)?;
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        {
            let mut tar = tar::Builder::new(&mut encoder);

            let mut header = tar::Header::new_gnu();
            header.set_size(profile_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "profile.json", profile_json.as_bytes())?;

            let mut header = tar::Header::new_gnu();
            header.set_size(favorites_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "favorites.json", favorites_json.as_bytes())?;

            let mut header = tar::Header::new_gnu();
            header.set_size(uploads_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "uploads.json", uploads_json.as_bytes())?;

            for wp in uploads.iter() {
                let filename = format!("{}_master.avif", wp.id);
                let path = storage_path.join(&filename);
                if let Ok(mut file) = std::fs::File::open(&path) {
                    let safe_title: String = wp.title.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
                    let archive_path = format!("uploads/{}_{}.avif", safe_title, wp.id);
                    let _ = tar.append_file(archive_path, &mut file);
                }
            }

            if pfp_url.starts_with("/assets/uploads/") {
                if let Some(filename) = pfp_url.split('/').last() {
                    let path = storage_path.join(filename);
                    if let Ok(mut file) = std::fs::File::open(&path) {
                        let _ = tar.append_file(format!("profile/{}", filename), &mut file);
                    }
                }
            }

            if let Some(banner_str) = &banner_url {
                if banner_str.starts_with("/assets/uploads/") {
                    if let Some(filename) = banner_str.split('/').last() {
                        let path = storage_path.join(filename);
                        if let Ok(mut file) = std::fs::File::open(&path) {
                            let _ = tar.append_file(format!("profile/{}", filename), &mut file);
                        }
                    }
                }
            }

            tar.finish()?;
        }
        
        encoder.finish()?;
        Ok(())
    }).await??;

    Ok(output_path_str)
}

pub async fn delete_user(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    
    let mut tx = pool.begin().await?;
    
    let user = sqlx::query!("SELECT name FROM users WHERE id = $1", user_id)
        .fetch_optional(&mut *tx)
        .await?;
        
    if let Some(u) = user {
        // Find all wallpapers to clean up cache later
        let wp_ids: Vec<String> = sqlx::query_scalar!("SELECT id FROM wallpapers WHERE author = $1", u.name)
            .fetch_all(&mut *tx)
            .await?;

        // Delete favorites
        sqlx::query!("DELETE FROM user_favorites WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;
            
        // Delete wallpapers uploaded by user
        sqlx::query!("DELETE FROM wallpapers WHERE author = $1", u.name)
            .execute(&mut *tx)
            .await?;
            
        // Delete user record
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;
            
        tx.commit().await?;
        
        get_user_cache().remove(user_id).await;
        
        let wp_cache = super::cache::get_wallpaper_cache();
        for id in wp_ids {
            wp_cache.remove(&id).await;
        }
        super::cache::get_wallpaper_list_cache().invalidate_all();
    }
    
    Ok(())
}

pub async fn search_users(query: &str, limit: u32) -> anyhow::Result<Vec<User>> {
    let pool = get_pool()?;
    
    let search_pattern = format!("%{}%", query.to_lowercase());
    
    let rows = sqlx::query!(
        "SELECT id, name, email, pfp_url, banner_url, bio, social_links, role, is_banned FROM users WHERE LOWER(name) LIKE $1 LIMIT $2",
        search_pattern,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    let users = rows.into_iter().map(|r| User {
        id: r.id,
        name: r.name,
        email: r.email,
        pfp_url: r.pfp_url,
        banner_url: r.banner_url,
        bio: r.bio,
        social_links: r.social_links.and_then(|v| serde_json::from_value(v).ok()),
        role: r.role,
        is_banned: r.is_banned,
    }).collect();

    Ok(users)
}

pub async fn follow_user_db(follower_id: &str, following_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        follower_id, following_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unfollow_user_db(follower_id: &str, following_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "DELETE FROM user_follows WHERE follower_id = $1 AND following_id = $2",
        follower_id, following_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn check_is_following_db(follower_id: &str, following_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;
    let row = sqlx::query!(
        "SELECT 1 as exists FROM user_follows WHERE follower_id = $1 AND following_id = $2",
        follower_id, following_id
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
        following_row.count.unwrap_or(0) as u32
    ))
}

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

    let logs = rows.into_iter().map(|r| crate::AuditLog {
        id: r.id,
        admin_id: r.admin_id,
        admin_name: r.admin_name,
        action: r.action,
        target_id: r.target_id,
        target_type: r.target_type,
        reason: r.reason,
        created_at: r.created_at,
    }).collect();

    Ok(logs)
}

pub async fn get_recent_users_db(limit: u32) -> anyhow::Result<Vec<crate::User>> {
    let pool = get_pool()?;
    let limit = limit as i64;

    let rows = sqlx::query!(
        "SELECT * FROM users ORDER BY created_at DESC LIMIT $1",
        limit
    ).fetch_all(pool).await?;

    let users = rows.into_iter().map(|r| crate::User {
        id: r.id,
        name: r.name,
        email: r.email,
        pfp_url: r.pfp_url,
        banner_url: r.banner_url,
        bio: r.bio,
        social_links: r.social_links.and_then(|v| serde_json::from_value(v).ok()),
        role: r.role,
        is_banned: r.is_banned,
    }).collect();

    Ok(users)
}

pub async fn admin_bulk_delete_users_db(hours_ago: u32, pattern: Option<&str>) -> anyhow::Result<u64> {
    let pool = get_pool()?;
    
    let hours_interval = chrono::Duration::hours(hours_ago as i64);
    let cutoff = chrono::Utc::now() - hours_interval;

    let users_to_delete: Vec<String> = if let Some(pat) = pattern {
        if pat.trim().is_empty() {
            sqlx::query_scalar!("SELECT id FROM users WHERE created_at >= $1", cutoff)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_scalar!("SELECT id FROM users WHERE created_at >= $1 AND name ~ $2", cutoff, pat)
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
        if delete_user(user_id).await.is_ok() {
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}

pub async fn admin_ban_user_db(user_id: &str, banned: bool) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!("UPDATE users SET is_banned = $1 WHERE id = $2", banned, user_id)
        .execute(pool)
        .await?;
        
    get_user_cache().remove(user_id).await;
    
    if !banned {
        let _ = crate::storage::create_notification_db(
            user_id, 
            "Account Restored", 
            "Your account has been unbanned. Welcome back to Wallr!"
        ).await;
    }
    
    Ok(())
}

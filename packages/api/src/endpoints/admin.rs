use crate::models::*;
use dioxus::prelude::*;

#[server]
pub async fn admin_delete_wallpaper(
    wallpaper_id: String,
    reason: Option<String>,
) -> Result<(), ServerFnError> {
    let admin = require_admin().await?;
    crate::storage::delete_wallpaper(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    crate::storage::log_audit_action_db(
        &admin.id,
        &admin.name,
        "DELETE",
        &wallpaper_id,
        "WALLPAPER",
        reason.as_deref(),
    )
    .await
    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_admin_stats_db()
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_audit_logs(limit: u32) -> Result<Vec<AuditLog>, ServerFnError> {
    let _admin = require_admin().await?;
    crate::storage::get_audit_logs_db(limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_recent_users(limit: u32) -> Result<Vec<User>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_recent_users_db(limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn admin_bulk_delete_users(
    hours_ago: u32,
    pattern: Option<String>,
) -> Result<u64, ServerFnError> {
    let admin = require_super_admin().await?;
    let deleted_count = crate::storage::admin_bulk_delete_users_db(hours_ago, pattern.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if deleted_count > 0 {
        crate::storage::log_audit_action_db(
            &admin.id,
            &admin.name,
            "BULK_DELETE",
            &format!("{} users", deleted_count),
            "USERS",
            Some(&format!("Hours ago: {}, Pattern: {:?}", hours_ago, pattern)),
        )
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    }

    Ok(deleted_count)
}

#[server]
pub async fn admin_ban_user(user_id: String, banned: bool) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    let target_user_opt = crate::storage::get_user_by_id(&user_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(target_user) = target_user_opt {
        if target_user.user.role == "super_admin" && admin.role != "super_admin" {
            return Err(ServerFnError::new(
                "Only a super admin can ban another super admin.",
            ));
        }
        if target_user.user.role == "admin" && admin.role == "moderator" {
            return Err(ServerFnError::new("api_err_mod_ban_admin"));
        }
    }
    if admin.id == user_id {
        return Err(ServerFnError::new("api_err_ban_self"));
    }
    crate::storage::admin_ban_user_db(&user_id, banned)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let action = if banned { "BAN_USER" } else { "UNBAN_USER" };
    crate::storage::log_audit_action_db(&admin.id, &admin.name, action, &user_id, "USER", None)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn report_wallpaper(wallpaper_id: String, reason: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::report_wallpaper_db(&wallpaper_id, &user.id, &reason)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn admin_ban_wallpaper_and_hash(
    wallpaper_id: String,
    reason: String,
) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    let wp = crate::storage::get_wallpaper_by_id(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?
        .ok_or_else(|| {
            crate::error::ApiError::from(anyhow::anyhow!("Wallpaper not found"))
                .into_server_fn_err()
        })?;

    if let Some(phash) = wp.phash {
        crate::storage::add_banned_hash(&phash, &admin.id, &reason)
            .await
            .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    }

    crate::storage::delete_wallpaper(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_reported_wallpapers(
    status: Option<String>,
) -> Result<Vec<ReportedWallpaper>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_reported_wallpapers_db(status.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn resolve_report(report_id: String, action: String) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    crate::storage::resolve_report_db(&report_id, &action, &admin.id, &admin.name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_user_role(user_id: String, new_role: String) -> Result<(), ServerFnError> {
    let _admin = require_admin().await?;
    crate::storage::update_user_role_db(&user_id, &new_role)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

use crate::auth::*;
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
            
        crate::storage::log_audit_action_db(
            &admin.id,
            &admin.name,
            "BAN_HASH",
            &phash,
            "HASH",
            Some(&reason),
        )
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    }

    crate::storage::delete_wallpaper(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    crate::storage::log_audit_action_db(
        &admin.id,
        &admin.name,
        "DELETE",
        &wallpaper_id,
        "WALLPAPER",
        Some(&reason),
    )
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
pub async fn resolve_report(report_id: String, action: String, notes: Option<String>) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    crate::storage::resolve_report_db(&report_id, &action, &admin.id, &admin.name, notes.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_user_role(user_id: String, new_role: String) -> Result<(), ServerFnError> {
    let admin = require_admin().await?;

    let valid_roles = ["user", "moderator", "admin", "super_admin"];
    if !valid_roles.contains(&new_role.as_str()) {
        return Err(ServerFnError::new("Invalid role"));
    }

    if admin.id == user_id {
        return Err(ServerFnError::new("Cannot modify own role"));
    }

    let target_user_opt = crate::storage::get_user_by_id(&user_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let target_user = match target_user_opt {
        Some(u) => u,
        None => return Err(ServerFnError::new("User not found")),
    };

    if target_user.user.role == "super_admin" && admin.role != "super_admin" {
        return Err(ServerFnError::new(
            "Only a super_admin can modify another super_admin",
        ));
    }

    if new_role == "super_admin" && admin.role != "super_admin" {
        return Err(ServerFnError::new(
            "Only a super_admin can grant super_admin role",
        ));
    }

    crate::storage::update_user_role_db(&user_id, &new_role)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn submit_dmca_claim(
    wallpaper_id: String,
    claimant_name: String,
    claimant_email: String,
    original_url: Option<String>,
    description: String,
    digital_signature: String,
    evidence_url: Option<String>,
) -> Result<(), ServerFnError> {
    crate::storage::submit_dmca_claim_db(
        &wallpaper_id,
        &claimant_name,
        &claimant_email,
        original_url.as_deref(),
        &description,
        &digital_signature,
        evidence_url.as_deref(),
    )
    .await
    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_dmca_claims(status: Option<String>) -> Result<Vec<DmcaClaim>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_dmca_claims_db(status.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn resolve_dmca_claim(claim_id: String, action: String, notes: Option<String>) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    crate::storage::resolve_dmca_claim_db(&claim_id, &action, &admin.id, &admin.name, notes.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn submit_moderation_appeal(
    target_id: String,
    target_type: String,
    reason: String,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::create_appeal_db(&target_id, &target_type, &user.id, &reason)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_moderation_appeals(
    status: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ModerationAppeal>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_appeals_db(status.as_deref(), limit, offset)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn resolve_moderation_appeal(
    appeal_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    
    // Process the appeal
    crate::storage::resolve_appeal_db(&appeal_id, &admin.id, &status, notes.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
        
    // Transparently notify the user
    if let Ok(Some(appeal)) = crate::storage::get_appeal_by_id_db(&appeal_id).await {
        let _ = crate::storage::create_notification_db(
            &appeal.user_id,
            "Moderation Appeal Update",
            &format!("Your appeal regarding {} has been marked as: {}", appeal.target_id, status),
        ).await;
    }

    Ok(())
}

#[server]
pub async fn submit_dmca_counter_notice(
    claim_id: String,
    content: String,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::submit_dmca_counter_notice_db(&claim_id, &user.id, &content)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_dmca_counter_notices(claim_id: String) -> Result<Vec<crate::models::DmcaCounterNotice>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_dmca_counter_notices_db(&claim_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn mark_dmca_claim_as_duplicate(
    claim_id: String,
    duplicate_of_id: String,
    notes: Option<String>,
) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    crate::storage::mark_dmca_claim_as_duplicate_db(&claim_id, &duplicate_of_id, &admin.id, &admin.name, notes.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}



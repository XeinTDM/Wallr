use crate::storage::get_pool;
use crate::storage::wallpapers::core::delete_wallpaper;
use crate::storage::create_notification_db;

pub async fn submit_dmca_claim_db(
    wallpaper_id: &str,
    claimant_name: &str,
    claimant_email: &str,
    original_url: Option<&str>,
    description: &str,
    digital_signature: &str,
    evidence_url: Option<&str>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();

    let wp = sqlx::query!("SELECT id FROM wallpapers WHERE id = $1", wallpaper_id)
        .fetch_optional(pool)
        .await?;

    if wp.is_none() {
        return Err(anyhow::anyhow!("Wallpaper not found"));
    }

    sqlx::query!(
        "INSERT INTO dmca_claims (id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, evidence_url) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, evidence_url
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_dmca_claims_db(
    status: Option<&str>,
) -> anyhow::Result<Vec<crate::models::DmcaClaim>> {
    let pool = get_pool()?;
    let mut results = Vec::new();

    if let Some(s) = status {
        let rows = sqlx::query!(
            "SELECT id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, status, notes, evidence_url, duplicate_of_id, created_at FROM dmca_claims WHERE status = $1 ORDER BY created_at DESC",
            s
        )
        .fetch_all(pool)
        .await?;

        for r in rows {
            results.push(crate::models::DmcaClaim {
                id: r.id,
                wallpaper_id: r.wallpaper_id,
                claimant_name: r.claimant_name,
                claimant_email: r.claimant_email,
                original_url: r.original_url,
                description: r.description,
                digital_signature: r.digital_signature,
                status: r.status,
                notes: r.notes,
                evidence_url: r.evidence_url,
                duplicate_of_id: r.duplicate_of_id,
                created_at: r.created_at,
            });
        }
    } else {
        let rows = sqlx::query!(
            "SELECT id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, status, notes, evidence_url, duplicate_of_id, created_at FROM dmca_claims ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;

        for r in rows {
            results.push(crate::models::DmcaClaim {
                id: r.id,
                wallpaper_id: r.wallpaper_id,
                claimant_name: r.claimant_name,
                claimant_email: r.claimant_email,
                original_url: r.original_url,
                description: r.description,
                digital_signature: r.digital_signature,
                status: r.status,
                notes: r.notes,
                evidence_url: r.evidence_url,
                duplicate_of_id: r.duplicate_of_id,
                created_at: r.created_at,
            });
        }
    }

    Ok(results)
}

pub async fn resolve_dmca_claim_db(
    claim_id: &str,
    action: &str,
    admin_id: &str,
    admin_name: &str,
    notes: Option<&str>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let claim = sqlx::query!(
        "SELECT wallpaper_id FROM dmca_claims WHERE id = $1",
        claim_id
    )
    .fetch_optional(pool)
    .await?;

    let wallpaper_id = match claim {
        Some(c) => c.wallpaper_id,
        None => return Err(anyhow::anyhow!("Claim not found")),
    };

    let wp = sqlx::query!("SELECT author_id, title FROM wallpapers WHERE id = $1", wallpaper_id)
        .fetch_optional(pool)
        .await?;

    if action == "delete_wallpaper" {
        delete_wallpaper(&wallpaper_id).await?;
        sqlx::query!(
            "UPDATE dmca_claims SET status = 'resolved_deleted', notes = $2 WHERE id = $1",
            claim_id, notes
        )
        .execute(pool)
        .await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "DELETE_DMCA_WALLPAPER",
            &wallpaper_id,
            "WALLPAPER",
            Some(&format!("From DMCA claim {}", claim_id)),
        )
        .await?;

        if let Some(w) = wp {
            create_notification_db(&w.author_id, "Wallpaper Deleted", &format!("Your wallpaper '{}' was deleted due to a DMCA claim.", w.title)).await.ok();
        }
    } else if action == "takedown" {
        sqlx::query!(
            "UPDATE wallpapers SET moderation_status = 'takedown', is_private = true WHERE id = $1",
            wallpaper_id
        ).execute(pool).await?;

        sqlx::query!(
            "UPDATE dmca_claims SET status = 'resolved_takedown', notes = $2 WHERE id = $1",
            claim_id, notes
        ).execute(pool).await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "TAKEDOWN_DMCA_WALLPAPER",
            &wallpaper_id,
            "WALLPAPER",
            Some(&format!("From DMCA claim {}", claim_id)),
        ).await?;

        if let Some(w) = wp {
            create_notification_db(&w.author_id, "Wallpaper Takedown", &format!("Your wallpaper '{}' was taken down due to a DMCA claim.", w.title)).await.ok();
        }
    } else if action == "dismiss" {
        sqlx::query!(
            "UPDATE dmca_claims SET status = 'dismissed', notes = $2 WHERE id = $1",
            claim_id, notes
        )
        .execute(pool)
        .await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "DISMISS_DMCA",
            claim_id,
            "DMCA_CLAIM",
            None,
        )
        .await?;
    } else {
        return Err(anyhow::anyhow!("Invalid action"));
    }

    Ok(())
}

pub async fn submit_dmca_counter_notice_db(
    claim_id: &str,
    user_id: &str,
    content: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO dmca_counter_notices (id, claim_id, user_id, content) VALUES ($1, $2, $3, $4)",
        id, claim_id, user_id, content
    ).execute(pool).await?;

    Ok(())
}

pub async fn get_dmca_counter_notices_db(
    claim_id: &str,
) -> anyhow::Result<Vec<crate::models::DmcaCounterNotice>> {
    let pool = get_pool()?;
    
    let rows = sqlx::query!(
        "SELECT id, claim_id, user_id, content, status, created_at FROM dmca_counter_notices WHERE claim_id = $1 ORDER BY created_at DESC",
        claim_id
    ).fetch_all(pool).await?;

    let mut notices = Vec::new();
    for r in rows {
        notices.push(crate::models::DmcaCounterNotice {
            id: r.id,
            claim_id: r.claim_id,
            user_id: r.user_id,
            content: r.content,
            status: r.status,
            created_at: r.created_at,
        });
    }

    Ok(notices)
}

pub async fn mark_dmca_claim_as_duplicate_db(
    claim_id: &str,
    duplicate_of_id: &str,
    admin_id: &str,
    admin_name: &str,
    notes: Option<&str>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    sqlx::query!(
        "UPDATE dmca_claims SET status = 'duplicate', duplicate_of_id = $2, notes = $3 WHERE id = $1",
        claim_id, duplicate_of_id, notes
    ).execute(pool).await?;

    crate::storage::log_audit_action_db(
        admin_id,
        admin_name,
        "DUPLICATE_DMCA",
        claim_id,
        "DMCA_CLAIM",
        Some(&format!("Duplicate of {}", duplicate_of_id)),
    ).await?;

    Ok(())
}

use crate::storage::get_pool;
use crate::storage::wallpapers::core::delete_wallpaper;

pub async fn submit_dmca_claim_db(
    wallpaper_id: &str,
    claimant_name: &str,
    claimant_email: &str,
    original_url: Option<&str>,
    description: &str,
    digital_signature: &str,
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
        "INSERT INTO dmca_claims (id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature
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
            "SELECT id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, status, created_at FROM dmca_claims WHERE status = $1 ORDER BY created_at DESC",
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
                created_at: r.created_at,
            });
        }
    } else {
        let rows = sqlx::query!(
            "SELECT id, wallpaper_id, claimant_name, claimant_email, original_url, description, digital_signature, status, created_at FROM dmca_claims ORDER BY created_at DESC"
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

    if action == "delete_wallpaper" {
        delete_wallpaper(&wallpaper_id).await?;
        sqlx::query!(
            "UPDATE dmca_claims SET status = 'resolved_deleted' WHERE id = $1",
            claim_id
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
    } else if action == "dismiss" {
        sqlx::query!(
            "UPDATE dmca_claims SET status = 'dismissed' WHERE id = $1",
            claim_id
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

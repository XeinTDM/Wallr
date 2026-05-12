use crate::storage::get_pool;
use super::core::delete_wallpaper;
use std::sync::RwLock;
use bk_tree::BKTree;

pub struct Hamming;
impl bk_tree::Metric<Vec<u8>> for Hamming {
    fn distance(&self, a: &Vec<u8>, b: &Vec<u8>) -> u32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum()
    }
    
    fn threshold_distance(&self, a: &Vec<u8>, b: &Vec<u8>, threshold: u32) -> Option<u32> {
        let dist = self.distance(a, b);
        if dist <= threshold { Some(dist) } else { None }
    }
}

pub static BANNED_HASH_TREE: std::sync::OnceLock<std::sync::RwLock<bk_tree::BKTree<Vec<u8>, Hamming>>> = std::sync::OnceLock::new();


pub async fn report_wallpaper_db(
    wallpaper_id: &str,
    reporter_id: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO reported_wallpapers (id, wallpaper_id, reporter_id, reason) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        id, wallpaper_id, reporter_id, reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_reported_wallpapers_db(
    status: Option<&str>,
) -> anyhow::Result<Vec<crate::ReportedWallpaper>> {
    let pool = get_pool()?;

    let mut results = Vec::new();

    if let Some(s) = status {
        let rows = sqlx::query!(
            r#"
            SELECT r.id, r.wallpaper_id, r.reporter_id, r.reason, r.status, r.created_at,
                   u.name as reporter_name, w.thumbnail_url as wallpaper_thumbnail
            FROM reported_wallpapers r
            JOIN users u ON r.reporter_id = u.id
            LEFT JOIN wallpapers w ON r.wallpaper_id = w.id
            WHERE r.status = $1
            ORDER BY r.created_at DESC
            "#,
            s
        )
        .fetch_all(pool)
        .await?;

        for r in rows {
            results.push(crate::ReportedWallpaper {
                id: r.id,
                wallpaper_id: r.wallpaper_id,
                reporter_id: r.reporter_id,
                reporter_name: r.reporter_name,
                reason: r.reason,
                status: r.status,
                created_at: r.created_at,
                wallpaper_thumbnail: Some(r.wallpaper_thumbnail),
            });
        }
    } else {
        let rows = sqlx::query!(
            r#"
            SELECT r.id, r.wallpaper_id, r.reporter_id, r.reason, r.status, r.created_at,
                   u.name as reporter_name, w.thumbnail_url as wallpaper_thumbnail
            FROM reported_wallpapers r
            JOIN users u ON r.reporter_id = u.id
            LEFT JOIN wallpapers w ON r.wallpaper_id = w.id
            ORDER BY r.created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        for r in rows {
            results.push(crate::ReportedWallpaper {
                id: r.id,
                wallpaper_id: r.wallpaper_id,
                reporter_id: r.reporter_id,
                reporter_name: r.reporter_name,
                reason: r.reason,
                status: r.status,
                created_at: r.created_at,
                wallpaper_thumbnail: Some(r.wallpaper_thumbnail),
            });
        }
    }

    Ok(results)
}

pub async fn resolve_report_db(
    report_id: &str,
    action: &str,
    admin_id: &str,
    admin_name: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let report = sqlx::query!(
        "SELECT wallpaper_id FROM reported_wallpapers WHERE id = $1",
        report_id
    )
    .fetch_optional(pool)
    .await?;

    let wallpaper_id = match report {
        Some(r) => r.wallpaper_id,
        None => return Err(anyhow::anyhow!("Report not found")),
    };

    if action == "delete_wallpaper" {
        delete_wallpaper(&wallpaper_id).await?;
        sqlx::query!(
            "UPDATE reported_wallpapers SET status = 'resolved_deleted' WHERE id = $1",
            report_id
        )
        .execute(pool)
        .await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "DELETE_REPORTED_WALLPAPER",
            &wallpaper_id,
            "WALLPAPER",
            Some(&format!("From report {}", report_id)),
        )
        .await?;
    } else if action == "dismiss" {
        sqlx::query!(
            "UPDATE reported_wallpapers SET status = 'dismissed' WHERE id = $1",
            report_id
        )
        .execute(pool)
        .await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "DISMISS_REPORT",
            report_id,
            "REPORT",
            None,
        )
        .await?;
    } else {
        return Err(anyhow::anyhow!("Invalid action"));
    }

    Ok(())
}

pub static HASH_TREE_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub async fn check_banned_phash(phash: &[u8]) -> anyhow::Result<bool> {
    let tree_lock = BANNED_HASH_TREE.get_or_init(|| RwLock::new(BKTree::new(Hamming)));
    
    if !HASH_TREE_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
        let pool = get_pool()?;
        let rows = sqlx::query!("SELECT phash FROM banned_hashes")
            .fetch_all(pool)
            .await?;
            
        let mut tree = tree_lock.write().unwrap();
        if !HASH_TREE_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
             for row in rows {
                let banned: Vec<u8> = row.phash;
                tree.add(banned);
             }
             HASH_TREE_LOADED.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
    
    let tree = tree_lock.read().unwrap();
    let phash_vec = phash.to_vec();
    let mut matches = tree.find(&phash_vec, 5);
    
    Ok(matches.next().is_some())
}

pub async fn add_banned_hash(phash: &[u8], admin_id: &str, reason: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO banned_hashes (id, phash, reason, added_by) VALUES ($1, $2, $3, $4)",
        id,
        phash,
        reason,
        admin_id
    )
    .execute(pool)
    .await?;
    
    if let Some(tree_lock) = BANNED_HASH_TREE.get()
        && let Ok(mut tree) = tree_lock.write() {
            tree.add(phash.to_vec());
        }
    
    Ok(())
}

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
    
    // Ensure wallpaper exists
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

pub async fn get_dmca_claims_db(status: Option<&str>) -> anyhow::Result<Vec<crate::models::DmcaClaim>> {
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

    let claim = sqlx::query!("SELECT wallpaper_id FROM dmca_claims WHERE id = $1", claim_id)
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


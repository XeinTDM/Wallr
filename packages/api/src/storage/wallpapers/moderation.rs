use crate::storage::get_pool;
use super::core::delete_wallpaper;

struct Hamming;
impl bk_tree::Metric<Vec<u8>> for Hamming {
    fn distance(&self, a: &Vec<u8>, b: &Vec<u8>) -> u64 {
        a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones() as u64).sum()
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

pub async fn check_banned_phash(phash: &[u8]) -> anyhow::Result<bool> {
    let tree_lock = BANNED_HASH_TREE.get_or_init(|| RwLock::new(BKTree::new(Hamming)));
    
    // Check if tree is empty, if so, initialize it
    let is_empty = {
        let tree = tree_lock.read().unwrap();
        tree.is_empty()
    };
    
    if is_empty {
        let pool = get_pool()?;
        let rows = sqlx::query!("SELECT phash FROM banned_hashes")
            .fetch_all(pool)
            .await?;
            
        let mut tree = tree_lock.write().unwrap();
        // Check again to avoid race condition
        if tree.is_empty() {
             for row in rows {
                let banned: Vec<u8> = row.phash;
                tree.add(banned);
             }
        }
    }
    
    let tree = tree_lock.read().unwrap();
    let matches = tree.find(phash.to_vec(), 5);
    
    Ok(!matches.is_empty())
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
    
    if let Some(tree_lock) = BANNED_HASH_TREE.get() {
        if let Ok(mut tree) = tree_lock.write() {
            tree.add(phash.to_vec());
        }
    }
    
    Ok(())
}

use crate::storage::get_pool;
use crate::storage::wallpapers::core::delete_wallpaper;
use crate::storage::create_notification_db;

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
            SELECT r.id, r.wallpaper_id, r.reporter_id, r.reason, r.status, r.notes, r.created_at,
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
                notes: r.notes,
                created_at: r.created_at,
                wallpaper_thumbnail: r.wallpaper_thumbnail,
            });
        }
    } else {
        let rows = sqlx::query!(
            r#"
            SELECT r.id, r.wallpaper_id, r.reporter_id, r.reason, r.status, r.notes, r.created_at,
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
                notes: r.notes,
                created_at: r.created_at,
                wallpaper_thumbnail: r.wallpaper_thumbnail,
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
    notes: Option<&str>,
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

    let wp = sqlx::query!("SELECT author_id, title FROM wallpapers WHERE id = $1", wallpaper_id)
        .fetch_optional(pool)
        .await?;

    if action == "delete_wallpaper" {
        delete_wallpaper(&wallpaper_id).await?;
        sqlx::query!(
            "UPDATE reported_wallpapers SET status = 'resolved_deleted', notes = $2 WHERE id = $1",
            report_id, notes
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
        
        if let Some(w) = wp {
            create_notification_db(&w.author_id, "Wallpaper Deleted", &format!("Your wallpaper '{}' was deleted due to reports.", w.title)).await.ok();
        }
    } else if action == "takedown" {
        sqlx::query!(
            "UPDATE wallpapers SET moderation_status = 'takedown', is_private = true WHERE id = $1",
            wallpaper_id
        ).execute(pool).await?;

        sqlx::query!(
            "UPDATE reported_wallpapers SET status = 'resolved_takedown', notes = $2 WHERE id = $1",
            report_id, notes
        ).execute(pool).await?;

        crate::storage::log_audit_action_db(
            admin_id,
            admin_name,
            "TAKEDOWN_REPORTED_WALLPAPER",
            &wallpaper_id,
            "WALLPAPER",
            Some(&format!("From report {}", report_id)),
        ).await?;

        if let Some(w) = wp {
            create_notification_db(&w.author_id, "Wallpaper Takedown", &format!("Your wallpaper '{}' was taken down due to reports.", w.title)).await.ok();
        }
    } else if action == "dismiss" {
        sqlx::query!(
            "UPDATE reported_wallpapers SET status = 'dismissed', notes = $2 WHERE id = $1",
            report_id, notes
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
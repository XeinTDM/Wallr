use crate::models::ModerationAppeal;
use crate::storage::get_pool;

pub async fn create_appeal_db(
    target_id: &str,
    target_type: &str,
    user_id: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let new_id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO moderation_appeals (id, target_id, target_type, user_id, reason) VALUES ($1, $2, $3, $4, $5)",
        new_id,
        target_id,
        target_type,
        user_id,
        reason
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_appeals_db(
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<ModerationAppeal>> {
    let pool = get_pool()?;

    let rows = if let Some(s) = status {
        sqlx::query!(
            "SELECT id, target_id, target_type, user_id, reason, status, reviewer_id, review_notes, created_at, resolved_at FROM moderation_appeals WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            s,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query!(
            "SELECT id, target_id, target_type, user_id, reason, status, reviewer_id, review_notes, created_at, resolved_at FROM moderation_appeals ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(pool)
        .await?
    };

    let appeals = rows
        .into_iter()
        .map(|r| ModerationAppeal {
            id: r.id,
            target_id: r.target_id,
            target_type: r.target_type,
            user_id: r.user_id,
            reason: r.reason,
            status: r.status,
            reviewer_id: r.reviewer_id,
            review_notes: r.review_notes,
            created_at: r.created_at,
            resolved_at: r.resolved_at,
        })
        .collect();

    Ok(appeals)
}

pub async fn resolve_appeal_db(
    appeal_id: &str,
    reviewer_id: &str,
    status: &str,
    notes: Option<&str>,
) -> anyhow::Result<()> {
    let pool = get_pool()?;

    let resolved_at = chrono::Utc::now();
    sqlx::query!(
        "UPDATE moderation_appeals SET status = $1, reviewer_id = $2, review_notes = $3, resolved_at = $4 WHERE id = $5",
        status,
        reviewer_id,
        notes,
        resolved_at,
        appeal_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_appeal_by_id_db(id: &str) -> anyhow::Result<Option<ModerationAppeal>> {
    let pool = get_pool()?;
    
    let row = sqlx::query!(
        "SELECT id, target_id, target_type, user_id, reason, status, reviewer_id, review_notes, created_at, resolved_at FROM moderation_appeals WHERE id = $1",
        id
    ).fetch_optional(pool).await?;

    Ok(row.map(|r| ModerationAppeal {
        id: r.id,
        target_id: r.target_id,
        target_type: r.target_type,
        user_id: r.user_id,
        reason: r.reason,
        status: r.status,
        reviewer_id: r.reviewer_id,
        review_notes: r.review_notes,
        created_at: r.created_at,
        resolved_at: r.resolved_at,
    }))
}

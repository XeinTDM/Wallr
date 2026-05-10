use crate::storage::cache::get_user_cache;
use crate::storage::get_pool;

pub async fn revoke_all_sessions(user_id: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE users SET token_version = token_version + 1 WHERE id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn update_password(user_id: &str, new_password_hash: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE users SET password_hash = $1, token_version = token_version + 1 WHERE id = $2",
        new_password_hash,
        user_id
    )
    .execute(pool)
    .await?;

    get_user_cache().remove(user_id).await;
    Ok(())
}

pub async fn create_password_reset_token_db(user_id: &str) -> anyhow::Result<String> {
    let pool = get_pool()?;
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    sqlx::query!(
        "DELETE FROM password_reset_tokens WHERE user_id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO password_reset_tokens (token, user_id, expires_at) VALUES ($1, $2, $3)",
        token,
        user_id,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn consume_password_reset_token_db(
    token: &str,
    new_password_hash: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let now = chrono::Utc::now();

    let token_record = sqlx::query!(
        "SELECT user_id, expires_at FROM password_reset_tokens WHERE token = $1",
        token
    )
    .fetch_optional(pool)
    .await?;

    let record = match token_record {
        Some(r) => r,
        None => return Err(anyhow::anyhow!("api_err_invalid_token")),
    };

    if record.expires_at < now {
        sqlx::query!("DELETE FROM password_reset_tokens WHERE token = $1", token)
            .execute(pool)
            .await?;
        return Err(anyhow::anyhow!("api_err_token_expired"));
    }

    update_password(&record.user_id, new_password_hash).await?;
    revoke_all_sessions(&record.user_id).await?;

    sqlx::query!("DELETE FROM password_reset_tokens WHERE token = $1", token)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_user_role_db(user_id: &str, new_role: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    sqlx::query!(
        "UPDATE users SET role = $1 WHERE id = $2",
        new_role,
        user_id
    )
    .execute(pool)
    .await?;
    get_user_cache().remove(user_id).await;
    Ok(())
}

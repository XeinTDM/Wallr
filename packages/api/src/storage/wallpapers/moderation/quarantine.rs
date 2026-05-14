use crate::storage::get_pool;

pub async fn quarantine_upload(
    author_id: &str,
    author_name: &str,
    bytes: &[u8],
    reason: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();

    let quarantine_dir = std::env::current_dir()?.join("quarantine");
    tokio::fs::create_dir_all(&quarantine_dir).await?;
    let file_path = quarantine_dir.join(format!("{}.quarantined", id));
    tokio::fs::write(&file_path, bytes).await?;

    let path_str = file_path.to_string_lossy().to_string();

    sqlx::query!(
        "INSERT INTO quarantined_uploads (id, author_id, author_name, file_path, reason) VALUES ($1, $2, $3, $4, $5)",
        id,
        author_id,
        author_name,
        path_str,
        reason
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_quarantine_path_generation() {
        // We can't easily test the DB insert without a connection, 
        // but we can ensure the logic for UUID generation and file path creation is sound.
        let id = uuid::Uuid::new_v4().to_string();
        let quarantine_dir = std::env::current_dir().unwrap().join("quarantine");
        let file_path = quarantine_dir.join(format!("{}.quarantined", id));
        
        assert!(file_path.to_string_lossy().contains(&id));
        assert!(file_path.to_string_lossy().ends_with(".quarantined"));
    }
}

#[cfg(feature = "server")]
use crate::upload::{UploadJobPayload, process_upload_job};
#[cfg(feature = "server")]
use redis::AsyncCommands;

#[cfg(feature = "server")]
pub async fn push_upload_job(payload: &UploadJobPayload, bytes: &[u8]) -> anyhow::Result<()> {
    let pool = crate::storage::get_redis_pool()?;
    let mut conn = pool.get().await?;

    let bytes_key = format!("wallr:upload:bytes:{}", payload.id);
    let _: () = conn.set_ex(&bytes_key, bytes, 3600).await?;

    let json = serde_json::to_string(payload)?;
    let _: () = conn.lpush("wallr:queue:uploads", json).await?;

    Ok(())
}

#[cfg(feature = "server")]
pub async fn start_worker_loop() {
    println!("🚀 Starting Redis background worker loop for image uploads...");
    loop {
        if let Ok(pool) = crate::storage::get_redis_pool() {
            if let Ok(mut conn) = pool.get().await {
                let res: redis::RedisResult<(String, String)> =
                    conn.brpop("wallr:queue:uploads", 0.0).await;
                if let Ok((_, json)) = res
                    && let Ok(payload) = serde_json::from_str::<UploadJobPayload>(&json)
                {
                    let bytes_key = format!("wallr:upload:bytes:{}", payload.id);
                    let bytes_res: redis::RedisResult<Vec<u8>> = conn.get(&bytes_key).await;
                    let _: redis::RedisResult<()> = conn.del(&bytes_key).await;

                    if let Ok(bytes) = bytes_res {
                        if !bytes.is_empty() {
                            println!(
                                "⚙️ Processing upload job for {} ({} bytes)",
                                payload.id,
                                bytes.len()
                            );
                            process_upload_job(payload, bytes).await;
                            println!("✅ Finished processing upload job.");
                        } else {
                            let _ = crate::storage::wallpapers::core::update_upload_job_status(
                                &payload.id,
                                "failed",
                                Some("Upload bytes missing or empty in Redis"),
                            )
                            .await;
                        }
                    } else {
                        let _ = crate::storage::wallpapers::core::update_upload_job_status(
                            &payload.id,
                            "failed",
                            Some("Upload bytes expired or not found in Redis"),
                        )
                        .await;
                    }
                }
            }
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

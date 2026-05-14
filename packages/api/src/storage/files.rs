use std::path::PathBuf;

#[cfg(feature = "server")]
#[derive(Clone)]
pub enum StorageProvider {
    S3 {
        client: aws_sdk_s3::Client,
        bucket: String,
        public_url: String,
    },
    Local {
        base_path: PathBuf,
        public_url: String,
    },
}

#[cfg(feature = "server")]
static PROVIDER: tokio::sync::OnceCell<StorageProvider> = tokio::sync::OnceCell::const_new();

#[cfg(feature = "server")]
pub async fn get_provider() -> StorageProvider {
    PROVIDER.get_or_init(|| async {
        let account_id = std::env::var("R2_ACCOUNT_ID").unwrap_or_default();
        let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
        let bucket = std::env::var("R2_BUCKET_NAME").unwrap_or_default();
        let public_url = std::env::var("R2_PUBLIC_URL").unwrap_or_default();

        if account_id.is_empty() || access_key.is_empty() || secret_key.is_empty() || bucket.is_empty() {
            // Local dev fallback
            let base_path = get_storage_path();
            let _ = tokio::fs::create_dir_all(&base_path).await;
            let local_url = if public_url.is_empty() {
                "/uploads".to_string()
            } else {
                public_url.trim_end_matches('/').to_string()
            };
            return StorageProvider::Local {
                base_path,
                public_url: local_url,
            };
        }

        if public_url.is_empty() {
            // Validation: Error or warn if R2_PUBLIC_URL is not set but CDN is requested
            panic!("CDN validation failed: R2_PUBLIC_URL must be set when using S3 storage provider");
        } else if public_url.contains("cdn.example.com") {
            panic!("CDN validation failed: R2_PUBLIC_URL cannot be the default cdn.example.com");
        }

        use aws_sdk_s3::config::{Credentials, Region};
        let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);
        let credentials = Credentials::new(access_key, secret_key, None, None, "cloudflare_r2");

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new("auto"))
            .endpoint_url(endpoint_url)
            .load()
            .await;

        let client = aws_sdk_s3::Client::new(&config);

        StorageProvider::S3 {
            client,
            bucket,
            public_url: public_url.trim_end_matches('/').to_string(),
        }
    }).await.clone()
}

pub fn get_storage_path() -> PathBuf {
    PathBuf::from("packages/ui/assets/uploads")
}

#[cfg(feature = "server")]
pub async fn save_image_file(id: &str, suffix: &str, ext: &str, data: &[u8]) -> anyhow::Result<String> {
    let filename = format!("{}_{}.{}", id, suffix, ext);
    let content_type = match ext {
        "avif" => "image/avif",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    };

    let provider = get_provider().await;
    match provider {
        StorageProvider::S3 { client, bucket, public_url } => {
            client
                .put_object()
                .bucket(&bucket)
                .key(&filename)
                .body(data.to_vec().into())
                .content_type(content_type)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to upload to S3: {}", e))?;
            Ok(format!("{}/{}", public_url, filename))
        }
        StorageProvider::Local { base_path, public_url } => {
            let _ = tokio::fs::create_dir_all(&base_path).await;
            let file_path = base_path.join(&filename);
            tokio::fs::write(&file_path, data).await?;
            Ok(format!("{}/{}", public_url, filename))
        }
    }
}

#[cfg(feature = "server")]
pub async fn get_image_url(id: &str, suffix: &str, ext: &str, is_private: bool) -> anyhow::Result<String> {
    let filename = format!("{}_{}.{}", id, suffix, ext);
    let provider = get_provider().await;
    match provider {
        StorageProvider::S3 { client, bucket, public_url } => {
            if is_private {
                use aws_sdk_s3::presigning::PresigningConfig;
                let config = PresigningConfig::expires_in(std::time::Duration::from_secs(3600))?;
                let req = client.get_object().bucket(&bucket).key(&filename).presigned(config).await?;
                Ok(req.uri().to_string())
            } else {
                Ok(format!("{}/{}", public_url, filename))
            }
        }
        StorageProvider::Local { public_url, .. } => {
            Ok(format!("{}/{}", public_url, filename))
        }
    }
}

#[cfg(feature = "server")]
pub async fn image_exists(id: &str, suffix: &str, ext: &str) -> anyhow::Result<bool> {
    let filename = format!("{}_{}.{}", id, suffix, ext);
    let provider = get_provider().await;
    match provider {
        StorageProvider::S3 { client, bucket, .. } => {
            let resp = client.head_object().bucket(&bucket).key(&filename).send().await;
            match resp {
                Ok(_) => Ok(true),
                Err(aws_sdk_s3::error::SdkError::ServiceError(e)) if e.err().is_not_found() => Ok(false),
                Err(_) => Ok(false), // Or handle other errors
            }
        }
        StorageProvider::Local { base_path, .. } => {
            let file_path = base_path.join(&filename);
            Ok(tokio::fs::try_exists(&file_path).await.unwrap_or(false))
        }
    }
}

#[cfg(feature = "server")]
pub async fn get_image_bytes(id: &str, suffix: &str, ext: &str) -> anyhow::Result<Vec<u8>> {
    let filename = format!("{}_{}.{}", id, suffix, ext);
    let provider = get_provider().await;
    match provider {
        StorageProvider::S3 { client, bucket, .. } => {
            let resp = client.get_object().bucket(&bucket).key(&filename).send().await
                .map_err(|e| anyhow::anyhow!("Failed to get from S3: {}", e))?;
            let data = resp.body.collect().await
                .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?
                .into_bytes();
            Ok(data.to_vec())
        }
        StorageProvider::Local { base_path, .. } => {
            let file_path = base_path.join(&filename);
            let data = tokio::fs::read(&file_path).await?;
            Ok(data)
        }
    }
}

#[cfg(feature = "server")]
pub async fn delete_all_wallpaper_files(id: &str) -> anyhow::Result<()> {
    let provider = get_provider().await;
    match provider {
        StorageProvider::S3 { client, bucket, .. } => {
            let prefix = format!("{}_", id);
            let resp = client.list_objects_v2().bucket(&bucket).prefix(&prefix).send().await;
            if let Ok(out) = resp {
                let contents = out.contents();
                for obj in contents {
                    if let Some(key) = obj.key() {
                        let _ = client.delete_object().bucket(&bucket).key(key).send().await;
                    }
                }
            }
        }
        StorageProvider::Local { base_path, .. } => {
            let mut entries = tokio::fs::read_dir(&base_path).await?;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(&format!("{}_", id)) {
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }
    Ok(())
}


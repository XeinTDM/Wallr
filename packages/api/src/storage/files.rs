use std::path::PathBuf;

#[cfg(feature = "server")]
static S3_CLIENT: tokio::sync::OnceCell<aws_sdk_s3::Client> = tokio::sync::OnceCell::const_new();

#[cfg(feature = "server")]
pub async fn get_s3_client() -> aws_sdk_s3::Client {
    S3_CLIENT.get_or_init(|| async {
        use aws_sdk_s3::config::{Credentials, Region};

        let account_id = std::env::var("R2_ACCOUNT_ID").unwrap_or_default();
        let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
        
        let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);

        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "cloudflare_r2",
        );

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new("auto"))
            .endpoint_url(endpoint_url)
            .load()
            .await;

        aws_sdk_s3::Client::new(&config)
    }).await.clone()
}

pub fn get_storage_path() -> PathBuf {
    PathBuf::from("packages/ui/assets/uploads")
}

#[cfg(feature = "server")]
pub async fn save_image_file(id: &str, suffix: &str, ext: &str, data: &[u8]) -> anyhow::Result<String> {
    let bucket = std::env::var("R2_BUCKET_NAME").unwrap_or_default();
    let public_url = std::env::var("R2_PUBLIC_URL").unwrap_or_default();
    let filename = format!("{}_{}.{}", id, suffix, ext);
    
    let client = get_s3_client().await;
    
    let content_type = match ext {
        "avif" => "image/avif",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    };

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

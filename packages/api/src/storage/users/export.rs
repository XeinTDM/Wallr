pub async fn export_user_data(user_id: &str) -> anyhow::Result<String> {
    let user_record = crate::storage::users::get_user_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("api_err_user_not_found"))?;

    let pool = crate::storage::get_pool()?;

    let get_json_array = |query_str: &str, bind_val: &str| async {
        let rows = sqlx::query(query_str)
            .bind(bind_val)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
        let mut results = Vec::new();
        for row in rows {
            use sqlx::Row;
            if let Ok(val) = row.try_get::<serde_json::Value, _>(0) {
                results.push(val);
            }
        }
        serde_json::to_string_pretty(&serde_json::Value::Array(results))
            .unwrap_or_else(|_| "[]".to_string())
    };

    let profile_json = serde_json::to_string_pretty(&user_record.user)?;
    let favorites_json = get_json_array("SELECT row_to_json(t) FROM (SELECT * FROM interactions WHERE user_id = $1 AND interaction_type = 'like') t", user_id).await;
    let collections_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM user_collections WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let comments_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM wallpaper_comments WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let follows_json = get_json_array("SELECT row_to_json(t) FROM (SELECT * FROM follows WHERE follower_id = $1 OR following_id = $1) t", user_id).await;
    let downloads_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM download_history WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let reports_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM reported_wallpapers WHERE reporter_id = $1) t",
        user_id,
    )
    .await;
    let notifications_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM notifications WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let dmca_claims_json = get_json_array("SELECT row_to_json(t) FROM (SELECT * FROM dmca_claims WHERE claimant_email = (SELECT email FROM users WHERE id = $1)) t", user_id).await;
    let dmca_counter_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM dmca_counter_notices WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let oauth_json = get_json_array(
        "SELECT row_to_json(t) FROM (SELECT * FROM user_oauth WHERE user_id = $1) t",
        user_id,
    )
    .await;
    let audit_json = get_json_array("SELECT row_to_json(t) FROM (SELECT * FROM audit_logs WHERE target_id = $1 OR admin_id = $1) t", user_id).await;

    let uploads =
        crate::storage::wallpapers::get_user_uploads(&user_record.user.name, 0, 10000).await?;
    let uploads_json = serde_json::to_string_pretty(&uploads)?;

    let storage_path = crate::storage::files::get_storage_path();
    let temp_dir = storage_path.join("tmp");
    if !tokio::fs::try_exists(&temp_dir).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&temp_dir).await?;
    }

    let safe_username: String = user_record
        .user
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let export_id = uuid::Uuid::new_v4();
    let export_dir = temp_dir.join(format!("{}_export_{}", safe_username, export_id));
    tokio::fs::create_dir_all(&export_dir).await?;

    tokio::fs::write(export_dir.join("profile.json"), profile_json).await?;
    tokio::fs::write(export_dir.join("favorites.json"), favorites_json).await?;
    tokio::fs::write(export_dir.join("uploads.json"), uploads_json).await?;
    tokio::fs::write(export_dir.join("collections.json"), collections_json).await?;
    tokio::fs::write(export_dir.join("comments.json"), comments_json).await?;
    tokio::fs::write(export_dir.join("follows.json"), follows_json).await?;
    tokio::fs::write(export_dir.join("downloads.json"), downloads_json).await?;
    tokio::fs::write(export_dir.join("reports.json"), reports_json).await?;
    tokio::fs::write(export_dir.join("notifications.json"), notifications_json).await?;
    tokio::fs::write(export_dir.join("dmca_claims.json"), dmca_claims_json).await?;
    tokio::fs::write(
        export_dir.join("dmca_counter_notices.json"),
        dmca_counter_json,
    )
    .await?;
    tokio::fs::write(export_dir.join("oauth.json"), oauth_json).await?;
    tokio::fs::write(export_dir.join("audit.json"), audit_json).await?;

    let uploads_dir = export_dir.join("uploads");
    tokio::fs::create_dir_all(&uploads_dir).await?;

    let profile_dir = export_dir.join("profile");
    tokio::fs::create_dir_all(&profile_dir).await?;

    async fn download_variant(id: &str, suffix: &str, ext: &str, target_dir: &std::path::PathBuf) {
        if crate::storage::files::image_exists(id, suffix, ext)
            .await
            .unwrap_or(false)
        {
            if let Ok(bytes) = crate::storage::files::get_image_bytes(id, suffix, ext).await {
                let path = target_dir.join(format!("{}_{}.{}", id, suffix, ext));
                let _ = tokio::fs::write(path, bytes).await;
            }
        }
    }

    let mut join_set = tokio::task::JoinSet::new();
    for wp in uploads.iter() {
        let id = wp.id.clone();
        let u_dir = uploads_dir.clone();
        join_set.spawn(async move {
            download_variant(&id, "master", "avif", &u_dir).await;
            download_variant(&id, "master", "jpg", &u_dir).await;
            download_variant(&id, "master", "png", &u_dir).await;
            download_variant(&id, "thumbnail", "avif", &u_dir).await;
            download_variant(&id, "thumbnail", "jpg", &u_dir).await;
            download_variant(&id, "live", "mp4", &u_dir).await;
            download_variant(&id, "live", "webm", &u_dir).await;
        });
    }

    while let Some(_) = join_set.join_next().await {}

    let pfp_url = user_record.user.pfp_url.clone();
    let banner_url = user_record.user.banner_url.clone();

    if pfp_url.starts_with("/assets/uploads/") {
        if let Some(filename) = pfp_url.split('/').next_back() {
            let id_part = filename.split('_').next().unwrap_or(filename);
            let suffix_part = filename
                .split('_')
                .nth(1)
                .unwrap_or("pfp")
                .split('.')
                .next()
                .unwrap_or("pfp");
            let ext = filename.split('.').next_back().unwrap_or("jpg");
            download_variant(id_part, suffix_part, ext, &profile_dir).await;
        }
    }

    if let Some(banner_str) = &banner_url {
        if banner_str.starts_with("/assets/uploads/") {
            if let Some(filename) = banner_str.split('/').next_back() {
                let id_part = filename.split('_').next().unwrap_or(filename);
                let suffix_part = filename
                    .split('_')
                    .nth(1)
                    .unwrap_or("banner")
                    .split('.')
                    .next()
                    .unwrap_or("banner");
                let ext = filename.split('.').next_back().unwrap_or("jpg");
                download_variant(id_part, suffix_part, ext, &profile_dir).await;
            }
        }
    }

    let temp_filename = format!("{}_export_{}.tar.gz", safe_username, export_id);
    let output_path = temp_dir.join(&temp_filename);
    let output_path_str = output_path.to_string_lossy().to_string();

    crate::get_heavy_runtime()
        .spawn_blocking(move || -> anyhow::Result<()> {
            let file = std::fs::File::create(&output_path)?;
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut tar = tar::Builder::new(encoder);
            tar.append_dir_all("", &export_dir)?;
            tar.finish()?;
            let _ = std::fs::remove_dir_all(&export_dir);
            Ok(())
        })
        .await??;

    Ok(output_path_str)
}

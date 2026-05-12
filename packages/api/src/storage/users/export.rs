pub async fn export_user_data(user_id: &str) -> anyhow::Result<String> {
    let user_record = crate::storage::users::get_user_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("api_err_user_not_found"))?;

    let favorites = crate::storage::wallpapers::get_user_favorites(user_id, 0, 1000).await?;
    let uploads = crate::storage::wallpapers::get_user_uploads(&user_record.user.name, 0, 1000).await?;

    let profile_json = serde_json::to_string_pretty(&user_record.user)?;
    let favorites_json = serde_json::to_string_pretty(&favorites)?;
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
    let temp_filename = format!("{}_export_{}.tar.gz", safe_username, uuid::Uuid::new_v4());
    let output_path = temp_dir.join(&temp_filename);
    let output_path_str = output_path.to_string_lossy().to_string();

    let pfp_url = user_record.user.pfp_url.clone();
    let banner_url = user_record.user.banner_url.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let file = std::fs::File::create(&output_path)?;
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        {
            let mut tar = tar::Builder::new(&mut encoder);

            let mut header = tar::Header::new_gnu();
            header.set_size(profile_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "profile.json", profile_json.as_bytes())?;

            let mut header = tar::Header::new_gnu();
            header.set_size(favorites_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "favorites.json", favorites_json.as_bytes())?;

            let mut header = tar::Header::new_gnu();
            header.set_size(uploads_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "uploads.json", uploads_json.as_bytes())?;

            for wp in uploads.iter() {
                let filename = format!("{}_master.avif", wp.id);
                let path = storage_path.join(&filename);
                if let Ok(mut file) = std::fs::File::open(&path) {
                    let safe_title: String = wp
                        .title
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '_' })
                        .collect();
                    let archive_path = format!("uploads/{}_{}.avif", safe_title, wp.id);
                    let _ = tar.append_file(archive_path, &mut file);
                }
            }

            if pfp_url.starts_with("/assets/uploads/")
                && let Some(filename) = pfp_url.split('/').next_back() {
                    let path = storage_path.join(filename);
                    if let Ok(mut file) = std::fs::File::open(&path) {
                        let _ = tar.append_file(format!("profile/{}", filename), &mut file);
                    }
                }

            if let Some(banner_str) = &banner_url
                && banner_str.starts_with("/assets/uploads/")
                    && let Some(filename) = banner_str.split('/').next_back() {
                        let path = storage_path.join(filename);
                        if let Ok(mut file) = std::fs::File::open(&path) {
                            let _ = tar.append_file(format!("profile/{}", filename), &mut file);
                        }
                    }

            tar.finish()?;
        }

        encoder.finish()?;
        Ok(())
    })
    .await??;

    Ok(output_path_str)
}

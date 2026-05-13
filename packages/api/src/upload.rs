use crate::models::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadJobPayload {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub author_name: String,
    pub user_tags: Vec<String>,
    pub is_private: bool,
}

#[cfg(feature = "server")]
pub async fn upload_raw_impl(
    title: String,
    author_id: String,
    author_name: String,
    user_tags: Vec<String>,
    bytes: Vec<u8>,
    is_private: bool,
) -> anyhow::Result<String> {
    if crate::ai::TAGGER.get().is_none() {
        return Err(anyhow::anyhow!(
            "AI Tagger is still initializing, please try again in a few moments."
        ));
    }

    let id = blake3::hash(&bytes).to_hex().to_string();

    crate::storage::wallpapers::core::create_upload_job(&id, &author_id, &title).await?;

    let payload = UploadJobPayload {
        id: id.clone(),
        title,
        author_id,
        author_name,
        user_tags,
        is_private,
    };

    crate::storage::queue::push_upload_job(&payload, &bytes).await?;

    Ok(id)
}

#[cfg(feature = "server")]
pub async fn process_upload_job(payload: UploadJobPayload, bytes: Vec<u8>) {
    let result: anyhow::Result<()> = async {
        let original_bytes_len = bytes.len() as u64;

        let is_mp4 = bytes.len() > 8 && &bytes[4..8] == b"ftyp";
        let is_webm = bytes.len() > 4 && bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3];
        let is_live = is_mp4 || is_webm;

        let (img, image_url) = if is_live {
            let ext = if is_mp4 { "mp4" } else { "webm" };
            let image_url = crate::storage::save_image_file(&payload.id, "master", ext, &bytes).await?;

            let temp_dir = std::env::temp_dir();
            let video_path = temp_dir.join(format!("{}.{}", payload.id, ext));
            let thumb_path = temp_dir.join(format!("{}_thumb.jpg", payload.id));
            tokio::fs::write(&video_path, &bytes).await?;

            let status_res = tokio::process::Command::new("ffmpeg")
                .arg("-i")
                .arg(&video_path)
                .arg("-vframes")
                .arg("1")
                .arg("-q:v")
                .arg("2")
                .arg("-y")
                .arg(&thumb_path)
                .status()
                .await;

            let status = match status_res {
                Ok(s) => s,
                Err(e) => {
                    let _ = tokio::fs::remove_file(&video_path).await;
                    let _ = tokio::fs::remove_file(&thumb_path).await;
                    return Err(anyhow::anyhow!("Failed to run ffmpeg: {}", e));
                }
            };

            if !status.success() {
                let _ = tokio::fs::remove_file(&video_path).await;
                let _ = tokio::fs::remove_file(&thumb_path).await;
                return Err(anyhow::anyhow!("ffmpeg failed to extract frame"));
            }

            let thumb_bytes = tokio::fs::read(&thumb_path).await;
            let _ = tokio::fs::remove_file(&video_path).await;
            let _ = tokio::fs::remove_file(&thumb_path).await;

            let thumb_bytes = thumb_bytes?;
            let img = crate::get_heavy_runtime().spawn_blocking(move || ::image::load_from_memory(&thumb_bytes))
                .await?
                .map_err(|e| anyhow::anyhow!("Failed to decode thumbnail: {}", e))?;

            (img, image_url)
        } else {
            let image_url = crate::storage::save_image_file(&payload.id, "master", "jpg", &bytes).await?;
            let img = crate::get_heavy_runtime().spawn_blocking({
                let bytes = bytes.clone();
                move || ::image::load_from_memory(&bytes)
            })
            .await?
            .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
            (img, image_url)
        };

        let (width, height) = (img.width(), img.height());

        let (phash_bytes, primary_colors, mut tags, embedding, is_nsfw, thumb_data, img_opt) = crate::get_heavy_runtime().spawn_blocking({
            let _id = payload.id.clone();
            let img = img.clone();
            move || -> anyhow::Result<_> {
                let hasher = img_hash::HasherConfig::new()
                    .hash_alg(img_hash::HashAlg::Gradient)
                    .to_hasher();

                let rgba = img.to_rgba8();
                let img_for_hash = img_hash::image::RgbaImage::from_raw(img.width(), img.height(), rgba.into_raw())
                    .ok_or_else(|| anyhow::anyhow!("Failed to convert image for hashing"))?;
                let phash = hasher.hash_image(&img_for_hash);
                let phash_bytes = phash.as_bytes().to_vec();

                let primary_colors = crate::processor::extract_dominant_colors(&img);

                let (tags, embedding, is_nsfw) = if let Some(tagger) = crate::ai::TAGGER.get() {
                    tagger
                        .tag_image(&img)
                        .unwrap_or_else(|_| (vec!["misc".to_string()], vec![0.0; 512], false))
                } else {
                    (vec!["misc".to_string()], vec![0.0; 512], false)
                };

                let thumb_data = crate::processor::generate_thumbnail(&img, 800);
                
                let final_img_opt = if is_live { None } else { Some(img) };

                Ok((phash_bytes, primary_colors, tags, embedding, is_nsfw, thumb_data, final_img_opt))
            }
        })
        .await??;

        let thumbnail_url = crate::storage::save_image_file(&payload.id, "thumb", "jpg", &thumb_data).await?;

        let is_banned = crate::storage::check_banned_phash(&phash_bytes).await?;
        if is_banned {
            return Err(anyhow::anyhow!("Upload rejected due to illegal content policy."));
        }

        if is_nsfw && !tags.contains(&"nsfw".to_string()) {
            tags.push("nsfw".to_string());
        }

        for ut in payload.user_tags {
            if !tags.contains(&ut) {
                tags.push(ut);
            }
        }

        if is_live && !tags.contains(&"live".to_string()) {
            tags.push("live".to_string());
        }

        if width >= 7680 && height >= 4320 {
            if !tags.contains(&"8k".to_string()) {
                tags.push("8k".to_string());
            }
        } else if width >= 3840 && height >= 2160 {
            if !tags.contains(&"4k".to_string()) {
                tags.push("4k".to_string());
            }
        } else if width >= 2560 && height >= 1440 {
            if !tags.contains(&"2k".to_string()) {
                tags.push("2k".to_string());
            }
        } else if width >= 1920 && height >= 1080 && !tags.contains(&"hd".to_string()) {
            tags.push("hd".to_string());
        }

        let mut final_image_url = image_url;
        let mut final_size_bytes = original_bytes_len;

        if !is_live
            && let Some(img) = img_opt {
                let bg_id = payload.id.clone();
                let avif_data = crate::get_heavy_runtime().spawn_blocking(move || crate::processor::convert_to_avif(&img)).await??;
                let avif_url = crate::storage::save_image_file(&bg_id, "master", "avif", &avif_data).await?;
                final_image_url = avif_url;
                final_size_bytes = avif_data.len() as u64;
            }

        let wallpaper = Wallpaper {
            id: payload.id.clone(),
            title: payload.title,
            author_id: payload.author_id,
            author_name: payload.author_name,
            image_url: final_image_url,
            thumbnail_url,
            tags,
            primary_colors,
            dimensions: (width, height),
            size_bytes: final_size_bytes,
            likes: 0,
            downloads: 0,
            created_at: chrono::Utc::now(),
            is_private: payload.is_private,
            is_live,
            embedding: Some(embedding),
            phash: Some(phash_bytes),
        };

        crate::storage::save_wallpaper_data(&wallpaper).await?;
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

        Ok(())
    }.await;

    if let Err(e) = result {
        let _ = crate::storage::wallpapers::core::update_upload_job_status(&payload.id, "failed", Some(&e.to_string())).await;
    } else {
        let _ = crate::storage::wallpapers::core::update_upload_job_status(&payload.id, "completed", None).await;
    }
}

#[cfg(feature = "server")]
pub async fn upload_media_impl(
    user_id: String,
    media_type: String,
    bytes: Vec<u8>,
) -> anyhow::Result<String> {
    let avif_data = crate::get_heavy_runtime().spawn_blocking({
        move || {
            let img = ::image::load_from_memory(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
            crate::processor::convert_to_avif(&img)
        }
    })
    .await??;

    let file_url = crate::storage::save_image_file(&user_id, &media_type, "avif", &avif_data).await?;

    crate::storage::update_user_media(&user_id, &media_type, &file_url).await?;
    Ok(file_url)
}
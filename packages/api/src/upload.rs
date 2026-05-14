use crate::models::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UploadJobPayload {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub user_tags: Vec<String>,
    pub is_private: bool,
}

#[cfg(feature = "server")]
struct AnalysisResult {
    phash_bytes: Vec<u8>,
    primary_colors: Vec<String>,
    tags: Vec<String>,
    embedding: Vec<f32>,
    is_nsfw: bool,
    thumb_data: Vec<u8>,
    final_img_opt: Option<image::DynamicImage>,
    extra_checks: Vec<(Vec<u8>, Vec<f32>)>,
}

#[cfg(feature = "server")]
pub async fn upload_raw_impl(
    title: String,
    description: Option<String>,
    source_url: Option<String>,
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

    if let Some(reason) =
        crate::storage::wallpapers::moderation::check_predatory_metadata(&title, &user_tags)
    {
        crate::storage::admin_ban_user_db(&author_id, true)
            .await
            .ok();
        crate::storage::freeze_user_wallpapers_db(&author_id)
            .await
            .ok();

        let full_reason = format!("Predatory metadata keyword: {}", reason);
        crate::storage::wallpapers::moderation::quarantine_upload(
            &author_id,
            &author_name,
            &bytes,
            &full_reason,
        )
        .await
        .ok();
        crate::storage::log_audit_action_db(
            "SYSTEM",
            "SYSTEM",
            "AUTO_BAN_CSAM",
            &author_id,
            "USER",
            Some(&full_reason),
        )
        .await
        .ok();

        return Err(anyhow::anyhow!(
            "Upload rejected due to illegal content policy."
        ));
    }

    let sha256_hex = {
        use sha2::Digest;
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    };

    if crate::storage::wallpapers::moderation::check_banned_exact_hash(&sha256_hex)
        .await
        .unwrap_or(false)
    {
        crate::storage::admin_ban_user_db(&author_id, true)
            .await
            .ok();
        crate::storage::freeze_user_wallpapers_db(&author_id)
            .await
            .ok();

        let full_reason = "Banned exact file hash (SHA-256)".to_string();
        crate::storage::wallpapers::moderation::quarantine_upload(
            &author_id,
            &author_name,
            &bytes,
            &full_reason,
        )
        .await
        .ok();
        crate::storage::log_audit_action_db(
            "SYSTEM",
            "SYSTEM",
            "AUTO_BAN_CSAM",
            &author_id,
            "USER",
            Some(&full_reason),
        )
        .await
        .ok();

        return Err(anyhow::anyhow!(
            "Upload rejected due to illegal content policy."
        ));
    }

    let id = blake3::hash(&bytes).to_hex().to_string();

    crate::storage::wallpapers::core::create_upload_job(&id, &author_id, &title).await?;

    let payload = UploadJobPayload {
        id: id.clone(),
        title,
        description,
        source_url,
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

        let (img, extra_frames, image_url) =
            extract_media(&payload.id, &bytes, is_live, is_mp4).await?;
        let (width, height) = (img.width(), img.height());

        let analysis = analyze_media(img, extra_frames, is_live).await?;

        let thumbnail_url =
            crate::storage::save_image_file(&payload.id, "thumb", "jpg", &analysis.thumb_data)
                .await?;

        enforce_moderation_ban(
            &payload.author_id,
            &payload.author_name,
            &bytes,
            &analysis.phash_bytes,
            &analysis.embedding,
            &analysis.extra_checks,
            "",
        )
        .await?;

        let mut tags = analysis.tags;
        if analysis.is_nsfw && !tags.contains(&"nsfw".to_string()) {
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

        if width >= 7680 && height >= 4320 && !tags.contains(&"8k".to_string()) {
            tags.push("8k".to_string());
        } else if width >= 3840 && height >= 2160 && !tags.contains(&"4k".to_string()) {
            tags.push("4k".to_string());
        } else if width >= 2560 && height >= 1440 && !tags.contains(&"2k".to_string()) {
            tags.push("2k".to_string());
        } else if width >= 1920 && height >= 1080 && !tags.contains(&"hd".to_string()) {
            tags.push("hd".to_string());
        }

        let mut final_image_url = image_url;
        let mut final_size_bytes = original_bytes_len;

        if !is_live {
            if let Some(img) = analysis.final_img_opt {
                let avif_data = crate::get_heavy_runtime()
                    .spawn_blocking(move || crate::processor::convert_to_avif(&img))
                    .await??;
                let avif_url =
                    crate::storage::save_image_file(&payload.id, "master", "avif", &avif_data)
                        .await?;
                final_image_url = avif_url;
                final_size_bytes = avif_data.len() as u64;
            }
        }

        let wallpaper = Wallpaper {
            id: payload.id.clone(),
            title: payload.title,
            author_id: payload.author_id,
            author_name: payload.author_name,
            image_url: final_image_url,
            thumbnail_url,
            tags,
            primary_colors: analysis.primary_colors,
            dimensions: (width, height),
            size_bytes: final_size_bytes,
            likes: 0,
            downloads: 0,
            created_at: chrono::Utc::now(),
            is_private: payload.is_private,
            is_live,
            embedding: Some(analysis.embedding),
            phash: Some(analysis.phash_bytes),
            description: payload.description,
            source_url: payload.source_url,
        };

        crate::storage::save_wallpaper_data(&wallpaper).await?;
        crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

        Ok(())
    }
    .await;

    if let Err(e) = result {
        let _ = crate::storage::wallpapers::core::update_upload_job_status(
            &payload.id,
            "failed",
            Some(&e.to_string()),
        )
        .await;
    } else {
        let _ = crate::storage::wallpapers::core::update_upload_job_status(
            &payload.id,
            "completed",
            None,
        )
        .await;
    }
}

#[cfg(feature = "server")]
pub async fn upload_media_impl(
    user_id: String,
    media_type: String,
    bytes: Vec<u8>,
) -> anyhow::Result<String> {
    let sha256_hex = {
        use sha2::Digest;
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    };

    if crate::storage::wallpapers::moderation::check_banned_exact_hash(&sha256_hex)
        .await
        .unwrap_or(false)
    {
        crate::storage::admin_ban_user_db(&user_id, true).await.ok();
        crate::storage::freeze_user_wallpapers_db(&user_id)
            .await
            .ok();

        let full_reason = format!("Banned exact file hash (SHA-256) as {}", media_type);
        crate::storage::wallpapers::moderation::quarantine_upload(
            &user_id,
            "Unknown (Media Upload)",
            &bytes,
            &full_reason,
        )
        .await
        .ok();
        crate::storage::log_audit_action_db(
            "SYSTEM",
            "SYSTEM",
            "AUTO_BAN_CSAM",
            &user_id,
            "USER",
            Some(&full_reason),
        )
        .await
        .ok();

        return Err(anyhow::anyhow!(
            "Upload rejected due to illegal content policy."
        ));
    }

    let (avif_data, phash_bytes, embedding) = crate::get_heavy_runtime()
        .spawn_blocking({
            let bytes_clone = bytes.clone();
            move || -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<f32>)> {
                let img = ::image::load_from_memory(&bytes_clone)
                    .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

                let hasher = img_hash::HasherConfig::new()
                    .hash_alg(img_hash::HashAlg::Gradient)
                    .to_hasher();

                let rgba = img.to_rgba8();
                let img_for_hash = img_hash::image::RgbaImage::from_raw(
                    img.width(),
                    img.height(),
                    rgba.into_raw(),
                )
                .ok_or_else(|| anyhow::anyhow!("Failed to convert image for hashing"))?;
                let phash = hasher.hash_image(&img_for_hash);
                let phash_bytes = phash.as_bytes().to_vec();

                let embedding = if let Some(tagger) = crate::ai::TAGGER.get() {
                    tagger
                        .tag_image(&img)
                        .map(|(_, emb, _)| emb)
                        .unwrap_or_else(|_| vec![0.0; 512])
                } else {
                    vec![0.0; 512]
                };

                let avif = crate::processor::convert_to_avif(&img)?;
                Ok((avif, phash_bytes, embedding))
            }
        })
        .await??;

    let context = format!("as {}", media_type);
    enforce_moderation_ban(
        &user_id,
        "Unknown (Media Upload)",
        &bytes,
        &phash_bytes,
        &embedding,
        &[],
        &context,
    )
    .await?;

    let file_url =
        crate::storage::save_image_file(&user_id, &media_type, "avif", &avif_data).await?;
    crate::storage::update_user_media(&user_id, &media_type, &file_url).await?;
    Ok(file_url)
}

#[cfg(feature = "server")]
async fn extract_media(
    id: &str,
    bytes: &[u8],
    is_live: bool,
    is_mp4: bool,
) -> anyhow::Result<(image::DynamicImage, Vec<image::DynamicImage>, String)> {
    if is_live {
        let ext = if is_mp4 { "mp4" } else { "webm" };
        let image_url = crate::storage::save_image_file(id, "master", ext, bytes).await?;

        let temp_dir = std::env::temp_dir();
        let video_path = temp_dir.join(format!("{}.{}", id, ext));
        let thumb_pattern = temp_dir.join(format!("{}_thumb_%d.jpg", id));
        tokio::fs::write(&video_path, bytes).await?;

        let status_res = tokio::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .arg("-vf")
            .arg("fps=1")
            .arg("-vframes")
            .arg("5")
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(&thumb_pattern)
            .status()
            .await;

        let status = match status_res {
            Ok(s) => s,
            Err(e) => {
                let _ = tokio::fs::remove_file(&video_path).await;
                return Err(anyhow::anyhow!("Failed to run ffmpeg: {}", e));
            }
        };

        if !status.success() {
            let _ = tokio::fs::remove_file(&video_path).await;
            return Err(anyhow::anyhow!("ffmpeg failed to extract frames"));
        }

        let mut extracted_images = Vec::new();
        for i in 1..=5 {
            let frame_path = temp_dir.join(format!("{}_thumb_{}.jpg", id, i));
            if let Ok(thumb_bytes) = tokio::fs::read(&frame_path).await {
                if let Ok(Ok(img)) = crate::get_heavy_runtime()
                    .spawn_blocking(move || ::image::load_from_memory(&thumb_bytes))
                    .await
                {
                    extracted_images.push(img);
                }
                let _ = tokio::fs::remove_file(&frame_path).await;
            }
        }

        let _ = tokio::fs::remove_file(&video_path).await;

        if extracted_images.is_empty() {
            return Err(anyhow::anyhow!("ffmpeg failed to extract any frames"));
        }

        let main_img = extracted_images.remove(0);
        Ok((main_img, extracted_images, image_url))
    } else {
        let image_url = crate::storage::save_image_file(id, "master", "jpg", bytes).await?;
        let img = crate::get_heavy_runtime()
            .spawn_blocking({
                let bytes = bytes.to_vec();
                move || ::image::load_from_memory(&bytes)
            })
            .await?
            .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
        Ok((img, Vec::new(), image_url))
    }
}

#[cfg(feature = "server")]
async fn analyze_media(
    img: image::DynamicImage,
    extra_frames: Vec<image::DynamicImage>,
    is_live: bool,
) -> anyhow::Result<AnalysisResult> {
    crate::get_heavy_runtime()
        .spawn_blocking(move || -> anyhow::Result<AnalysisResult> {
            let hasher = img_hash::HasherConfig::new()
                .hash_alg(img_hash::HashAlg::Gradient)
                .to_hasher();

            let rgba = img.to_rgba8();
            let img_for_hash =
                img_hash::image::RgbaImage::from_raw(img.width(), img.height(), rgba.into_raw())
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

            let mut extra_checks = Vec::new();
            for frame in extra_frames {
                let frame_rgba = frame.to_rgba8();
                if let Some(frame_for_hash) = img_hash::image::RgbaImage::from_raw(
                    frame.width(),
                    frame.height(),
                    frame_rgba.into_raw(),
                ) {
                    let f_phash = hasher.hash_image(&frame_for_hash).as_bytes().to_vec();
                    let f_emb = if let Some(tagger) = crate::ai::TAGGER.get() {
                        tagger
                            .tag_image(&frame)
                            .map(|(_, e, _)| e)
                            .unwrap_or_else(|_| vec![0.0; 512])
                    } else {
                        vec![0.0; 512]
                    };
                    extra_checks.push((f_phash, f_emb));
                }
            }

            Ok(AnalysisResult {
                phash_bytes,
                primary_colors,
                tags,
                embedding,
                is_nsfw,
                thumb_data,
                final_img_opt,
                extra_checks,
            })
        })
        .await?
}

#[cfg(feature = "server")]
async fn enforce_moderation_ban(
    author_id: &str,
    author_name: &str,
    bytes: &[u8],
    phash_bytes: &[u8],
    embedding: &[f32],
    extra_checks: &[(Vec<u8>, Vec<f32>)],
    context: &str,
) -> anyhow::Result<()> {
    let mut is_banned_hash = crate::storage::check_banned_phash(phash_bytes).await?;
    let mut is_banned_embedding =
        crate::storage::wallpapers::moderation::check_banned_embedding(embedding)
            .await
            .unwrap_or(false);

    for (f_phash, f_emb) in extra_checks {
        if is_banned_hash || is_banned_embedding {
            break;
        }
        if crate::storage::check_banned_phash(f_phash)
            .await
            .unwrap_or(false)
        {
            is_banned_hash = true;
        }
        if crate::storage::wallpapers::moderation::check_banned_embedding(f_emb)
            .await
            .unwrap_or(false)
        {
            is_banned_embedding = true;
        }
    }

    if is_banned_hash || is_banned_embedding {
        crate::storage::admin_ban_user_db(author_id, true)
            .await
            .ok();
        crate::storage::freeze_user_wallpapers_db(author_id)
            .await
            .ok();

        let reason = if is_banned_embedding {
            "Banned CLIP Embedding"
        } else {
            "Banned pHash"
        };

        let full_reason = if context.is_empty() {
            format!("Attempted to upload banned content ({})", reason)
        } else {
            format!(
                "Attempted to upload banned content ({}) {}",
                reason, context
            )
        };

        crate::storage::wallpapers::moderation::quarantine_upload(
            author_id,
            author_name,
            bytes,
            &full_reason,
        )
        .await
        .ok();
        crate::storage::log_audit_action_db(
            "SYSTEM",
            "SYSTEM",
            "AUTO_BAN_CSAM",
            author_id,
            "USER",
            Some(&full_reason),
        )
        .await
        .ok();

        return Err(anyhow::anyhow!(
            "Upload rejected due to illegal content policy."
        ));
    }

    Ok(())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_upload_job_payload_serialization() {
        let payload = UploadJobPayload {
            id: "job123".to_string(),
            title: "Test Wallpaper".to_string(),
            description: Some("A nice wallpaper".to_string()),
            source_url: None,
            author_id: "user123".to_string(),
            author_name: "TestUser".to_string(),
            user_tags: vec!["nature".to_string(), "landscape".to_string()],
            is_private: false,
        };

        let serialized = serde_json::to_string(&payload).expect("Should serialize");
        assert!(serialized.contains("job123"));
        assert!(serialized.contains("nature"));

        let deserialized: UploadJobPayload = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.id, payload.id);
        assert_eq!(deserialized.user_tags, payload.user_tags);
    }
}

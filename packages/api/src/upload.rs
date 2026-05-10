use crate::models::*;
use dioxus::prelude::*;

#[cfg(feature = "server")]
pub async fn upload_raw_impl(
    title: String,
    author: String,
    user_tags: Vec<String>,
    bytes: Vec<u8>,
    is_private: bool,
) -> anyhow::Result<String> {
    if crate::ai::TAGGER.get().is_none() {
        return Err(anyhow::anyhow!(
            "AI Tagger is still initializing, please try again in a few moments."
        ));
    }

    let original_bytes_len = bytes.len() as u64;

    let is_mp4 = bytes.len() > 8 && &bytes[4..8] == b"ftyp";
    let is_webm = bytes.len() > 4 && bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3];
    let is_live = is_mp4 || is_webm;

    let id = blake3::hash(&bytes).to_hex().to_string();

    let (img, image_url) = if is_live {
        let ext = if is_mp4 { "mp4" } else { "webm" };
        let image_url = tokio::task::spawn_blocking({
            let id = id.clone();
            let bytes = bytes.clone();
            move || crate::storage::save_image_file(&id, "master", ext, &bytes)
        })
        .await??;

        let temp_dir = std::env::temp_dir();
        let video_path = temp_dir.join(format!("{}.{}", id, ext));
        let thumb_path = temp_dir.join(format!("{}_thumb.jpg", id));
        tokio::fs::write(&video_path, &bytes).await?;

        let status = tokio::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(&thumb_path)
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run ffmpeg: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("ffmpeg failed to extract frame"));
        }

        let thumb_bytes = tokio::fs::read(&thumb_path).await?;
        let img = tokio::task::spawn_blocking(move || ::image::load_from_memory(&thumb_bytes))
            .await?
            .map_err(|e| anyhow::anyhow!("Failed to decode thumbnail: {}", e))?;

        let _ = tokio::fs::remove_file(video_path).await;
        let _ = tokio::fs::remove_file(thumb_path).await;

        (img, image_url)
    } else {
        let image_url = tokio::task::spawn_blocking({
            let id = id.clone();
            let bytes = bytes.clone();
            move || crate::storage::save_image_file(&id, "master", "jpg", &bytes)
        })
        .await??;
        let img = tokio::task::spawn_blocking({
            let bytes = bytes.clone();
            move || ::image::load_from_memory(&bytes)
        })
        .await?
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
        (img, image_url)
    };

    let (width, height) = (img.width(), img.height());

    let (phash_bytes, primary_colors, mut tags, embedding, is_nsfw, thumbnail_url, img_opt) = tokio::task::spawn_blocking({
        let id = id.clone();
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
            let thumbnail_url = crate::storage::save_image_file(&id, "thumb", "jpg", &thumb_data)?;

            let final_img_opt = if is_live { None } else { Some(img) };

            Ok((phash_bytes, primary_colors, tags, embedding, is_nsfw, thumbnail_url, final_img_opt))
        }
    })
    .await??;

    let is_banned = crate::storage::check_banned_phash(&phash_bytes).await?;
    if is_banned {
        return Err(anyhow::anyhow!(
            "Upload rejected due to illegal content policy."
        ));
    }

    if is_nsfw && !tags.contains(&"nsfw".to_string()) {
        tags.push("nsfw".to_string());
    }

    for ut in user_tags {
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
    } else if width >= 1920 && height >= 1080 {
        if !tags.contains(&"hd".to_string()) {
            tags.push("hd".to_string());
        }
    }

    let wallpaper = Wallpaper {
        id: id.clone(),
        title,
        author,
        image_url,
        thumbnail_url,
        tags,
        primary_colors,
        dimensions: (width, height),
        size_bytes: original_bytes_len,
        likes: 0,
        downloads: 0,
        created_at: chrono::Utc::now(),
        is_private,
        is_live,
        embedding: Some(embedding),
        phash: Some(phash_bytes),
    };

    crate::storage::save_wallpaper_data(&wallpaper).await?;

    if !is_live {
        if let Some(img) = img_opt {
            let bg_id = id.clone();
            tokio::spawn(async move {
                let avif_result =
                    tokio::task::spawn_blocking(move || crate::processor::convert_to_avif(&img))
                        .await;

                if let Ok(Ok(avif_data)) = avif_result {
                    let avif_url =
                        crate::storage::save_image_file(&bg_id, "master", "avif", &avif_data)
                            .unwrap_or_default();
                    if let Ok(pool) = crate::storage::get_pool() {
                        let size = avif_data.len() as i64;
                        let _ = sqlx::query!(
                            "UPDATE wallpapers SET size_bytes = $1, image_url = $2 WHERE id = $3",
                            size,
                            avif_url,
                            bg_id
                        )
                        .execute(pool)
                        .await;
                        crate::storage::cache::get_wallpaper_cache()
                            .remove(&bg_id)
                            .await;
                    }
                }
            });
        }
    }

    crate::storage::cache::get_wallpaper_list_cache().invalidate_all();

    Ok(id)
}

#[cfg(feature = "server")]
pub async fn upload_media_impl(
    user_id: String,
    media_type: String,
    bytes: Vec<u8>,
) -> anyhow::Result<String> {
    let file_url = tokio::task::spawn_blocking({
        let user_id = user_id.clone();
        let media_type = media_type.clone();
        move || {
            let img = ::image::load_from_memory(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
            let avif_data = crate::processor::convert_to_avif(&img)?;
            crate::storage::save_image_file(&user_id, &media_type, "avif", &avif_data)
        }
    })
    .await??;

    crate::storage::update_user_media(&user_id, &media_type, &file_url).await?;
    Ok(file_url)
}

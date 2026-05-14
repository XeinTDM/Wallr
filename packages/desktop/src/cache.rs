use std::path::PathBuf;

pub async fn get_cached_wallpaper(url: &str, wp_id: &str) -> Result<PathBuf, String> {
    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".desktop_data")
        .join("cache");

    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    let ext = if url.ends_with(".mp4") {
        "mp4"
    } else if url.ends_with(".webm") {
        "webm"
    } else if url.ends_with(".jpg") || url.ends_with(".jpeg") {
        "jpg"
    } else {
        "avif"
    };

    let cache_path = cache_dir.join(format!("{}.{}", wp_id, ext));

    if cache_path.exists() {
        return Ok(cache_path);
    }

    // Download the file
    let real_url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!(
            "https://wallr.app{}",
            url.replace("/assets/uploads/", "/upload/")
        )
    };

    let bytes = reqwest::get(&real_url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    std::fs::write(&cache_path, bytes).map_err(|e| e.to_string())?;

    Ok(cache_path)
}

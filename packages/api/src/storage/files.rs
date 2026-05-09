use std::fs;
use std::path::PathBuf;

pub fn get_storage_path() -> PathBuf {
    PathBuf::from("packages/ui/assets/uploads")
}

pub fn save_image_file(id: &str, suffix: &str, ext: &str, data: &[u8]) -> anyhow::Result<String> {
    let dir = get_storage_path();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let filename = format!("{}_{}.{}", id, suffix, ext);
    let path = dir.join(&filename);
    fs::write(path, data)?;

    Ok(format!("/assets/uploads/{}", filename))
}

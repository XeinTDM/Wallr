use global_hotkey::hotkey::{Code, Modifiers};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey_next_wp: (Option<Modifiers>, Code),
    pub hotkey_prev_wp: (Option<Modifiers>, Code),
    pub hotkey_save_wp: (Option<Modifiers>, Code),
    pub hotkey_toggle_ui: (Option<Modifiers>, Code),
    pub wallpaper_mode: String, // "Center", "Crop", "Fit", "Span", "Stretch", "Tile"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey_next_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowRight),
            hotkey_prev_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft),
            hotkey_save_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS),
            hotkey_toggle_ui: (Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH),
            wallpaper_mode: "Crop".to_string(),
        }
    }
}

impl AppConfig {
    fn get_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".desktop_data")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&data) {
                    return config;
                }
            }
        }
        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

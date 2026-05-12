use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallpaper {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub author_name: String,
    pub image_url: String,
    pub thumbnail_url: String,
    pub tags: Vec<String>,
    pub primary_colors: Vec<String>,
    pub dimensions: (u32, u32),
    pub size_bytes: u64,
    pub likes: u32,
    pub downloads: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_private: bool,
    pub is_live: bool,
    pub embedding: Option<Vec<f32>>,
    pub phash: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FilterOptions {
    pub resolution: String,
    pub sort: String,
    pub aspect_ratio: String,
    pub color: String,
    pub ai_filter: String,
    pub timeframe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub item_count: u32,
    pub cover_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserCollection {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub item_count: u32,
    pub cover_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub pfp_url: String,
    pub banner_url: Option<String>,
    pub bio: Option<String>,
    pub social_links: Option<std::collections::HashMap<String, String>>,
    pub role: String,
    pub is_banned: bool,
    pub active_playlist_id: Option<String>,
    pub playlist_interval_secs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreatorAnalytics {
    pub total_uploads: u32,
    pub total_likes: u32,
    pub total_downloads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminStats {
    pub total_users: u32,
    pub total_wallpapers: u32,
    pub total_downloads: u32,
    pub total_likes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLog {
    pub id: String,
    pub admin_id: String,
    pub admin_name: String,
    pub action: String,
    pub target_id: String,
    pub target_type: String,
    pub reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WallpaperComment {
    pub id: String,
    pub wallpaper_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_pfp: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportedWallpaper {
    pub id: String,
    pub wallpaper_id: String,
    pub reporter_id: String,
    pub reporter_name: String,
    pub reason: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub wallpaper_thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRecord {
    pub user: User,
    pub password_hash: String,
    pub token_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DmcaClaim {
    pub id: String,
    pub wallpaper_id: String,
    pub claimant_name: String,
    pub claimant_email: String,
    pub original_url: Option<String>,
    pub description: String,
    pub digital_signature: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UploadJob {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

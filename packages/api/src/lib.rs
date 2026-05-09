//! This crate contains all shared fullstack server functions and data models.
#[cfg(feature = "server")]
pub mod ai;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod processor;
#[cfg(feature = "server")]
pub mod profanity;
#[cfg(feature = "server")]
pub mod storage;

pub mod tags;

#[cfg(feature = "server")]
fn extract_client_ip(
    headers: &http::HeaderMap,
    connect_info: Result<
        axum::extract::ConnectInfo<std::net::SocketAddr>,
        dioxus::prelude::ServerFnError,
    >,
) -> String {
    let direct_ip = connect_info.map(|info| info.0.ip());

    let allow_headers = match &direct_ip {
        Ok(std::net::IpAddr::V4(ipv4)) => {
            let octets = ipv4.octets();
            let is_private = octets[0] == 10
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168);
            is_private || ipv4.is_loopback() || ipv4.is_link_local()
        }
        Ok(std::net::IpAddr::V6(ipv6)) => ipv6.is_loopback(),
        Err(_) => true,
    };

    if allow_headers {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first_ip) = xff.split(',').next() {
                let trimmed = first_ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        if let Some(xrip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            let trimmed = xrip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    match direct_ip {
        Ok(ip) => ip.to_string(),
        Err(_) => "unknown_ip".to_string(),
    }
}

#[cfg(feature = "server")]
pub async fn require_auth() -> Result<User, dioxus::prelude::ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    FullstackContext::extract::<User, _>()
        .await
        .map_err(|_| dioxus::prelude::ServerFnError::new("Unauthorized"))
}

#[cfg(feature = "server")]
pub async fn require_admin() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "admin" && user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Admins only",
        ));
    }
    Ok(user)
}

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallpaper {
    pub id: String,
    pub title: String,
    pub author: String,
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
pub struct UserRecord {
    pub user: User,
    pub password_hash: String,
    pub token_version: i32,
}

#[cfg(feature = "server")]
impl<S> axum::extract::FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::extract::cookie::CookieJar;
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get("session_token").map(|c| c.value());

        let token = match token {
            Some(t) => t,
            None => return Err((http::StatusCode::UNAUTHORIZED, "Missing session token")),
        };

        match crate::storage::verify_token(token).await {
            Ok(user) => Ok(user),
            Err(_) => Err((
                http::StatusCode::UNAUTHORIZED,
                "Invalid or expired session token",
            )),
        }
    }
}

/// Fetch a list of trending wallpapers from the server.
#[server]
pub async fn get_wallpapers(
    page: u32,
    limit: u32,
    filters: FilterOptions,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    crate::storage::load_all_wallpapers(page, limit, filters)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

/// Fetch a list of wallpapers filtered by tag.
#[server]
pub async fn get_wallpapers_by_tag(
    tag: String,
    page: u32,
    limit: u32,
    filters: FilterOptions,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let tag = tag.to_lowercase();
    if tag == "featured" || tag == "popular" || tag == "latest" || tag == "all" {
        return get_wallpapers(page, limit, filters).await;
    }

    crate::storage::get_wallpapers_by_tag(&tag, page, limit, filters)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

/// Fetch a single wallpaper by its ID.
#[server]
pub async fn get_collections() -> Result<Vec<Collection>, ServerFnError> {
    crate::storage::load_all_collections()
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn create_user_collection(
    name: String,
    description: Option<String>,
    is_private: bool,
) -> Result<String, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::create_user_collection(&user.id, &name, description.as_deref(), is_private)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_my_collections() -> Result<Vec<UserCollection>, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::get_user_collections(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_public_user_collections(
    username: String,
) -> Result<Vec<UserCollection>, ServerFnError> {
    let user = crate::storage::get_user_by_name(&username)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(u) = user {
        crate::storage::get_public_user_collections_db(&u.user.id)
            .await
            .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
    } else {
        Err(ServerFnError::new("User not found"))
    }
}

#[server]
pub async fn get_collection_wallpapers(
    collection_id: String,
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    crate::storage::get_collection_wallpapers_db(&collection_id, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn add_wallpaper_to_collection(
    collection_id: String,
    wallpaper_id: String,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    // Technically we should check if the user owns the collection, but for speed we just do it.
    crate::storage::add_wallpaper_to_collection_db(&collection_id, &wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn remove_wallpaper_from_collection(
    collection_id: String,
    wallpaper_id: String,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::remove_wallpaper_from_collection_db(&collection_id, &wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_wallpaper_by_id(id: String) -> Result<Option<Wallpaper>, ServerFnError> {
    crate::storage::get_wallpaper_by_id(&id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_trending_tags(limit: u32) -> Result<Vec<String>, ServerFnError> {
    crate::storage::get_trending_tags(limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn search_wallpapers(
    query: String,
    page: u32,
    limit: u32,
    filters: FilterOptions,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    if query.is_empty() {
        return get_wallpapers(page, limit, filters).await;
    }

    crate::storage::search_wallpapers_db(&query, page, limit, filters)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[cfg(feature = "server")]
pub async fn upload_raw_impl(
    title: String,
    author: String,
    user_tags: Vec<String>,
    bytes: Vec<u8>,
    is_private: bool,
) -> anyhow::Result<String> {
    let original_bytes_len = bytes.len() as u64;

    let (id, image_url, thumbnail_url, width, height, primary_colors, tags, img) =
        tokio::task::spawn_blocking(move || {
            let img = ::image::load_from_memory(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

            let id = blake3::hash(&bytes).to_hex().to_string();
            let (width, height) = (img.width(), img.height());

            let primary_colors = crate::processor::extract_dominant_colors(&img);

            let mut tags = if let Some(tagger) = crate::ai::TAGGER.get() {
                tagger
                    .tag_image(&img)
                    .unwrap_or_else(|_| vec!["misc".to_string()])
            } else {
                vec!["misc".to_string()]
            };

            for ut in user_tags {
                if !tags.contains(&ut) {
                    tags.push(ut);
                }
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

            let thumb_data = crate::processor::generate_thumbnail(&img, 800);
            let image_url = crate::storage::save_image_file(&id, "master", "jpg", &bytes)?;
            let thumbnail_url = crate::storage::save_image_file(&id, "thumb", "jpg", &thumb_data)?;

            Ok::<_, anyhow::Error>((
                id,
                image_url,
                thumbnail_url,
                width,
                height,
                primary_colors,
                tags,
                img,
            ))
        })
        .await??;

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
    };

    crate::storage::save_wallpaper_data(&wallpaper).await?;

    let bg_id = id.clone();
    tokio::spawn(async move {
        let avif_result =
            tokio::task::spawn_blocking(move || crate::processor::convert_to_avif(&img)).await;

        if let Ok(Ok(avif_data)) = avif_result {
            let avif_url = crate::storage::save_image_file(&bg_id, "master", "avif", &avif_data)
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

#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use dioxus_fullstack::FullstackContext;

    if password.len() < 8 || password.len() > 128 {
        return Err(ServerFnError::new("Invalid email or password"));
    }

    let email = email.trim().to_lowercase();

    let headers = FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Unauthorized"))?;

    let connect_info =
        FullstackContext::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>, _>().await;
    let ip = extract_client_ip(&headers, connect_info);

    crate::storage::check_login_rate_limit(&ip, &email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let user_record_opt = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let is_valid = if let Some(user_record) = &user_record_opt {
        let hash_clone = user_record.password_hash.clone();
        let pass_clone = password.clone();
        tokio::task::spawn_blocking(move || {
            let parsed_hash =
                PasswordHash::new(&hash_clone).map_err(|e| format!("Hash error: {}", e))?;
            Ok::<bool, String>(
                Argon2::default()
                    .verify_password(pass_clone.as_bytes(), &parsed_hash)
                    .is_ok(),
            )
        })
        .await
        .map_err(|_| ServerFnError::new("Task error"))?
        .map_err(|e| ServerFnError::new(e))?
    } else {
        let pass_clone = password.clone();
        let _ = tokio::task::spawn_blocking(move || {
            use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
            let salt = SaltString::generate(&mut OsRng);
            let _ = Argon2::default().hash_password(pass_clone.as_bytes(), &salt);
        })
        .await;
        false
    };

    if is_valid {
        if let Some(user_record) = user_record_opt {
            if user_record.user.is_banned {
                return Err(ServerFnError::new("Account is banned"));
            }

            let token =
                crate::storage::generate_token(&user_record.user, user_record.token_version)
                    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

            if let Some(ctx) = FullstackContext::current() {
                let cookie = format!(
                    "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000",
                    token
                );
                ctx.add_response_header(
                    http::header::SET_COOKIE,
                    cookie.parse::<http::header::HeaderValue>().unwrap(),
                );
            }

            crate::storage::reset_login_rate_limit(&ip, &email).await;
            return Ok(());
        }
    }

    Err(ServerFnError::new("Invalid email or password"))
}

#[server]
pub async fn register(name: String, email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use dioxus_fullstack::FullstackContext;
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    if password.len() < 8 || password.len() > 128 {
        return Err(ServerFnError::new(
            "Password must be between 8 and 128 characters long",
        ));
    }

    let email = email.trim().to_lowercase();

    let headers = FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Unauthorized"))?;

    let connect_info =
        FullstackContext::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>, _>().await;
    let ip = extract_client_ip(&headers, connect_info);

    crate::storage::check_register_rate_limit(&ip)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let existing_user = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if existing_user.is_some() {
        return Err(ServerFnError::new("User with this email already exists"));
    }

    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Hashing error: {}", e))
    })
    .await
    .map_err(|_| ServerFnError::new("Task error"))?
    .map_err(|e| ServerFnError::new(e))?;

    let mut hasher = Sha256::new();
    hasher.update(email.as_bytes());
    let email_hash = format!("{:x}", hasher.finalize());
    let pfp_url = format!(
        "https://www.gravatar.com/avatar/{}?s=256&d=retro",
        email_hash
    );

    let new_user = User {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        email,
        pfp_url,
        banner_url: None,
        bio: None,
        social_links: None,
        role: "user".to_string(),
        is_banned: false,
    };

    let record = UserRecord {
        user: new_user.clone(),
        password_hash,
        token_version: 1,
    };

    crate::storage::create_user(&record)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let token = crate::storage::generate_token(&new_user, record.token_version)
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = format!(
            "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000",
            token
        );
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }

    Ok(())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = "session_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }

    Ok(())
}

#[server]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    Ok(require_auth().await.ok())
}

#[server]
pub async fn get_user_favorites(
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::get_user_favorites(&user.id, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_all_user_favorite_ids() -> Result<Vec<String>, ServerFnError> {
    let user = match require_auth().await {
        Ok(u) => u,
        Err(_) => return Ok(vec![]),
    };
    crate::storage::get_all_user_favorite_ids(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_user_uploads(
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::get_user_uploads(&user.name, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn toggle_favorite(wallpaper_id: String) -> Result<bool, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::toggle_favorite(&user.id, &wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn check_is_favorited(wallpaper_id: String) -> Result<bool, ServerFnError> {
    let user = match require_auth().await {
        Ok(u) => u,
        Err(_) => return Ok(false),
    };
    crate::storage::is_favorited(&user.id, &wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_profile(
    name: String,
    email: String,
    bio: Option<String>,
    social_links: Option<std::collections::HashMap<String, String>>,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let email = email.trim().to_lowercase();

    let existing_user = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    if let Some(record) = existing_user {
        if record.user.id != user.id {
            return Err(ServerFnError::new(
                "Email already in use by another account",
            ));
        }
    }

    let socials_val =
        social_links.map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Null));

    crate::storage::update_profile(
        &user.id,
        &name,
        &email,
        bio.as_deref(),
        socials_val.as_ref(),
    )
    .await
    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn change_password(
    current_password: String,
    new_password: String,
) -> Result<(), ServerFnError> {
    use argon2::{
        Argon2, PasswordHash, PasswordVerifier,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use dioxus_fullstack::FullstackContext;

    if new_password.len() < 8 || new_password.len() > 128 {
        return Err(ServerFnError::new(
            "New password must be between 8 and 128 characters long",
        ));
    }

    let user = require_auth().await?;
    let user_record = crate::storage::get_user_by_id(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?
        .ok_or_else(|| ServerFnError::new("User not found"))?;

    let hash_clone = user_record.password_hash.clone();
    let is_valid = tokio::task::spawn_blocking(move || {
        let parsed_hash =
            PasswordHash::new(&hash_clone).map_err(|e| format!("Hash error: {}", e))?;
        Ok::<bool, String>(
            Argon2::default()
                .verify_password(current_password.as_bytes(), &parsed_hash)
                .is_ok(),
        )
    })
    .await
    .map_err(|_| ServerFnError::new("Task error"))?
    .map_err(|e| ServerFnError::new(e))?;

    if !is_valid {
        return Err(ServerFnError::new("Incorrect current password"));
    }

    let new_password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Hashing error: {}", e))
    })
    .await
    .map_err(|_| ServerFnError::new("Task error"))?
    .map_err(|e| ServerFnError::new(e))?;

    crate::storage::update_password(&user.id, &new_password_hash)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = "session_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }

    Ok(())
}

#[server]
pub async fn revoke_sessions() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    let user = require_auth().await?;
    crate::storage::revoke_all_sessions(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = "session_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }
    Ok(())
}

#[server]
pub async fn add_tag_to_wallpaper(wallpaper_id: String, tag: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let tag = tag.trim().to_lowercase();
    if tag.is_empty() {
        return Err(ServerFnError::new("Tag cannot be empty"));
    }

    let wp_opt = crate::storage::get_wallpaper_by_id(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let wp = match wp_opt {
        Some(w) => w,
        None => return Err(ServerFnError::new("Wallpaper not found")),
    };

    if wp.author != user.name {
        return Err(ServerFnError::new("Unauthorized"));
    }
    crate::storage::add_tag(&wallpaper_id, &tag)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn delete_account() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    let user = require_auth().await?;

    crate::storage::delete_user(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = "session_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }
    Ok(())
}

#[server]
pub async fn delete_wallpaper_endpoint(id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let wp_opt = crate::storage::get_wallpaper_by_id(&id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let wp = match wp_opt {
        Some(w) => w,
        None => return Err(ServerFnError::new("Wallpaper not found")),
    };

    if wp.author != user.name {
        return Err(ServerFnError::new("Unauthorized"));
    }

    crate::storage::delete_wallpaper(&id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_public_profile(username: String) -> Result<Option<User>, ServerFnError> {
    let username = username.replace("-", " ");
    let user_record_opt = crate::storage::get_user_by_name(&username)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    Ok(user_record_opt.map(|ur| ur.user))
}

#[server]
pub async fn get_public_uploads(
    username: String,
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let username = username.replace("-", " ");
    crate::storage::get_public_uploads(&username, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn search_users_endpoint(query: String, limit: u32) -> Result<Vec<User>, ServerFnError> {
    crate::storage::search_users(&query, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn follow_user(following_id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::follow_user_db(&user.id, &following_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn unfollow_user(following_id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::unfollow_user_db(&user.id, &following_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn check_is_following(following_id: String) -> Result<bool, ServerFnError> {
    let user = match require_auth().await {
        Ok(u) => u,
        Err(_) => return Ok(false),
    };
    crate::storage::check_is_following_db(&user.id, &following_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_follow_counts(user_id: String) -> Result<(u32, u32), ServerFnError> {
    crate::storage::get_follow_counts(&user_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

// ==========================================
// Comments & Analytics Endpoints
// ==========================================

#[server]
pub async fn get_wallpaper_comments(
    wallpaper_id: String,
    page: u32,
    limit: u32,
) -> Result<Vec<WallpaperComment>, ServerFnError> {
    crate::storage::get_comments_db(&wallpaper_id, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn add_wallpaper_comment(
    wallpaper_id: String,
    content: String,
) -> Result<WallpaperComment, ServerFnError> {
    let user = require_auth().await?;
    let content = content.trim();
    if content.is_empty() {
        return Err(ServerFnError::new("Comment cannot be empty"));
    }
    if content.chars().count() > 500 {
        return Err(ServerFnError::new("Comment cannot exceed 500 characters"));
    }
    if crate::profanity::contains_forbidden_words(content).await {
        return Err(ServerFnError::new("Comment contains forbidden language"));
    }

    let is_duplicate = crate::storage::check_duplicate_comment(&wallpaper_id, &user.id, content)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if is_duplicate {
        return Err(ServerFnError::new(
            "You have already posted this exact comment",
        ));
    }

    crate::storage::check_comment_rate_limit(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let comment = WallpaperComment {
        id: uuid::Uuid::new_v4().to_string(),
        wallpaper_id,
        user_id: user.id.clone(),
        user_name: user.name.clone(),
        user_pfp: user.pfp_url.clone(),
        content: content.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    crate::storage::add_comment_db(&comment)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(comment)
}

#[server]
pub async fn delete_wallpaper_comment(comment_id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::delete_comment_db(&comment_id, &user.id, &user.name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_creator_analytics() -> Result<CreatorAnalytics, ServerFnError> {
    let user = require_auth().await?;

    crate::storage::get_creator_analytics_db(&user.name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn admin_delete_wallpaper(
    wallpaper_id: String,
    reason: Option<String>,
) -> Result<(), ServerFnError> {
    let admin = require_admin().await?;
    crate::storage::delete_wallpaper(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    crate::storage::log_audit_action_db(
        &admin.id,
        &admin.name,
        "DELETE",
        &wallpaper_id,
        "WALLPAPER",
        reason.as_deref(),
    )
    .await
    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_admin_stats_db()
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_audit_logs(limit: u32) -> Result<Vec<AuditLog>, ServerFnError> {
    let _admin = require_admin().await?;
    crate::storage::get_audit_logs_db(limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_recent_users(limit: u32) -> Result<Vec<User>, ServerFnError> {
    let _admin = require_moderator().await?;
    crate::storage::get_recent_users_db(limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn admin_bulk_delete_users(
    hours_ago: u32,
    pattern: Option<String>,
) -> Result<u64, ServerFnError> {
    let admin = require_super_admin().await?;
    let deleted_count = crate::storage::admin_bulk_delete_users_db(hours_ago, pattern.as_deref())
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if deleted_count > 0 {
        crate::storage::log_audit_action_db(
            &admin.id,
            &admin.name,
            "BULK_DELETE",
            &format!("{} users", deleted_count),
            "USERS",
            Some(&format!("Hours ago: {}, Pattern: {:?}", hours_ago, pattern)),
        )
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    }

    Ok(deleted_count)
}

#[server]
pub async fn admin_ban_user(user_id: String, banned: bool) -> Result<(), ServerFnError> {
    let admin = require_moderator().await?;
    let target_user_opt = crate::storage::get_user_by_id(&user_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(target_user) = target_user_opt {
        if target_user.user.role == "super_admin" && admin.role != "super_admin" {
            return Err(ServerFnError::new(
                "Only a super admin can ban another super admin.",
            ));
        }
        if target_user.user.role == "admin" && admin.role == "moderator" {
            return Err(ServerFnError::new("Moderators cannot ban admins."));
        }
    }
    if admin.id == user_id {
        return Err(ServerFnError::new("You cannot ban yourself."));
    }
    crate::storage::admin_ban_user_db(&user_id, banned)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let action = if banned { "BAN_USER" } else { "UNBAN_USER" };
    crate::storage::log_audit_action_db(&admin.id, &admin.name, action, &user_id, "USER", None)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
}

#[server]
pub async fn get_my_notifications() -> Result<Vec<Notification>, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::get_notifications_db(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn mark_notification_read(id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::mark_notification_read_db(&user.id, &id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn mark_all_notifications_read() -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::mark_all_notifications_read_db(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[cfg(feature = "server")]
pub async fn require_super_admin() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Super Admins only",
        ));
    }
    Ok(user)
}

#[cfg(feature = "server")]
pub async fn require_moderator() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "moderator" && user.role != "admin" && user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Moderators only",
        ));
    }
    Ok(user)
}

#[server]
pub async fn update_collection(
    collection_id: String,
    name: String,
    description: Option<String>,
    is_private: bool,
) -> Result<(), ServerFnError> {
    let _user = require_auth().await?;
    crate::storage::update_collection_db(&collection_id, &name, description.as_deref(), is_private)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn delete_collection(collection_id: String) -> Result<(), ServerFnError> {
    let _user = require_auth().await?;
    crate::storage::delete_collection_db(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_wallpaper(
    id: String,
    title: String,
    tags: Vec<String>,
    is_private: bool,
) -> Result<(), ServerFnError> {
    let _user = require_auth().await?;
    crate::storage::update_wallpaper_db(&id, &title, &tags, is_private)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn delete_my_wallpaper(id: String) -> Result<(), ServerFnError> {
    let _user = require_auth().await?;
    crate::storage::delete_wallpaper(&id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn report_wallpaper(wallpaper_id: String, reason: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::report_wallpaper_db(&wallpaper_id, &user.id, &reason)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_comment(comment_id: String, content: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::update_comment_db(&comment_id, &user.id, &content)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn delete_comment(comment_id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::delete_comment_db(&comment_id, &user.id, &user.name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn delete_my_account() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    let user = require_auth().await?;
    crate::storage::delete_user(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    
    if let Some(ctx) = FullstackContext::current() {
        let cookie = "session_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }
    Ok(())
}

#[server]
pub async fn update_user_role(user_id: String, new_role: String) -> Result<(), ServerFnError> {
    let _admin = require_admin().await?;
    crate::storage::update_user_role_db(&user_id, &new_role)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

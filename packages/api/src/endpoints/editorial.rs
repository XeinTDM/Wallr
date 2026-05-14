use crate::models::{EditorialCollection, Wallpaper};
use dioxus::prelude::*;

#[server]
pub async fn get_published_editorial_collections() -> Result<Vec<EditorialCollection>, ServerFnError> {
    crate::storage::get_published_editorial_collections().await.map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn get_all_editorial_collections() -> Result<Vec<EditorialCollection>, ServerFnError> {
    // Only allow admins to list all (including drafts)
    let user = crate::auth::require_auth().await?;
    if user.user.role != "admin" {
        return Err(ServerFnError::ServerError("unauthorized".into()));
    }
    crate::storage::get_all_editorial_collections().await.map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn get_editorial_collection_wallpapers(
    collection_id: String,
    page: i64,
) -> Result<Vec<Wallpaper>, ServerFnError> {
    let limit = 50;
    let offset = (page.max(1) - 1) * limit;
    crate::storage::get_editorial_collection_wallpapers(&collection_id, limit, offset)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn create_editorial_collection(
    title: String,
    description: String,
    cover_url: Option<String>,
    is_published: bool,
) -> Result<String, ServerFnError> {
    let user = crate::auth::require_auth().await?;
    if user.user.role != "admin" {
        return Err(ServerFnError::ServerError("unauthorized".into()));
    }
    crate::storage::create_editorial_collection(&title, &description, cover_url.as_deref(), is_published)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn update_editorial_collection(
    id: String,
    title: String,
    description: String,
    cover_url: Option<String>,
    is_published: bool,
) -> Result<(), ServerFnError> {
    let user = crate::auth::require_auth().await?;
    if user.user.role != "admin" {
        return Err(ServerFnError::ServerError("unauthorized".into()));
    }
    crate::storage::update_editorial_collection(&id, &title, &description, cover_url.as_deref(), is_published)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn add_wallpaper_to_editorial_collection(
    collection_id: String,
    wallpaper_id: String,
    sort_order: i32,
) -> Result<(), ServerFnError> {
    let user = crate::auth::require_auth().await?;
    if user.user.role != "admin" {
        return Err(ServerFnError::ServerError("unauthorized".into()));
    }
    crate::storage::add_wallpaper_to_editorial_collection(&collection_id, &wallpaper_id, sort_order)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn remove_wallpaper_from_editorial_collection(
    collection_id: String,
    wallpaper_id: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::require_auth().await?;
    if user.user.role != "admin" {
        return Err(ServerFnError::ServerError("unauthorized".into()));
    }
    crate::storage::remove_wallpaper_from_editorial_collection(&collection_id, &wallpaper_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

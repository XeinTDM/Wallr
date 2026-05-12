use crate::models::*;
use dioxus::prelude::*;

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

#[server]
pub async fn add_tag_to_wallpaper(wallpaper_id: String, tag: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let tag = tag.trim().to_lowercase();
    if tag.is_empty() {
        return Err(ServerFnError::new("api_err_tag_empty"));
    }

    let wp_opt = crate::storage::get_wallpaper_by_id(&wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let wp = match wp_opt {
        Some(w) => w,
        None => return Err(ServerFnError::new("api_err_wp_not_found")),
    };

    if wp.author_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }
    crate::storage::add_tag(&wallpaper_id, &tag)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

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
        None => return Err(ServerFnError::new("api_err_wp_not_found")),
    };

    if wp.author_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }

    crate::storage::delete_wallpaper(&id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(())
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
pub async fn get_similar_wallpapers(
    id: String,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    crate::storage::get_similar_wallpapers_db(&id, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

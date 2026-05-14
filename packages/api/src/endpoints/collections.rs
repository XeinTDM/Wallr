use crate::models::*;
use dioxus::prelude::*;

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
        Err(ServerFnError::new("api_err_user_not_found"))
    }
}

#[server]
pub async fn get_collection_wallpapers(
    collection_id: String,
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let user_opt = require_auth().await.ok();
    let caller_id = user_opt.as_ref().map(|u| u.id.as_str());
    let is_admin = user_opt.as_ref().map_or(false, |u| u.role == "admin" || u.role == "super_admin");

    crate::storage::get_collection_wallpapers_db(&collection_id, page, limit, caller_id, is_admin)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn add_wallpaper_to_collection(
    collection_id: String,
    wallpaper_id: String,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let owner_opt = crate::storage::get_collection_owner(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let owner_id = match owner_opt {
        Some(id) => id,
        None => return Err(ServerFnError::new("api_err_collection_not_found")),
    };

    if owner_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }

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
    let owner_opt = crate::storage::get_collection_owner(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let owner_id = match owner_opt {
        Some(id) => id,
        None => return Err(ServerFnError::new("api_err_collection_not_found")),
    };

    if owner_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }

    crate::storage::remove_wallpaper_from_collection_db(&collection_id, &wallpaper_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn update_collection(
    collection_id: String,
    name: String,
    description: Option<String>,
    is_private: bool,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let owner_opt = crate::storage::get_collection_owner(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let owner_id = match owner_opt {
        Some(id) => id,
        None => return Err(ServerFnError::new("api_err_collection_not_found")),
    };

    if owner_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }

    crate::storage::update_collection_db(&collection_id, &name, description.as_deref(), is_private)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn delete_collection(collection_id: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    let owner_opt = crate::storage::get_collection_owner(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let owner_id = match owner_opt {
        Some(id) => id,
        None => return Err(ServerFnError::new("api_err_collection_not_found")),
    };

    if owner_id != user.id && user.role != "admin" {
        return Err(ServerFnError::new("api_err_unauthorized"));
    }

    crate::storage::delete_collection_db(&collection_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}



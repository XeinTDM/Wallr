use crate::models::*;
use dioxus::prelude::*;
use crate::auth::*;

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
pub async fn check_favorites(wallpaper_ids: Vec<String>) -> Result<Vec<String>, ServerFnError> {
    let user = match require_auth().await {
        Ok(u) => u,
        Err(_) => return Ok(vec![]),
    };
    crate::storage::check_favorites_db(&user.id, &wallpaper_ids)
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

#[server]
pub async fn get_user_download_history(
    page: u32,
    limit: u32,
) -> Result<std::sync::Arc<Vec<Wallpaper>>, ServerFnError> {
    let user = require_auth().await?;
    crate::storage::get_user_download_history_db(&user.id, page, limit)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_followers(
    username: String,
    page: u32,
    limit: u32,
) -> Result<Vec<User>, ServerFnError> {
    let limit_i64 = limit as i64;
    let offset_i64 = (page * limit) as i64;

    let target_user = crate::storage::get_user_by_name(&username)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?
        .ok_or_else(|| dioxus::prelude::ServerFnError::new("api_err_user_not_found"))?;

    let followers = crate::storage::get_followers_db(&target_user.user.id, limit_i64, offset_i64)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(followers.into_iter().map(|u| u.user).collect())
}

#[server]
pub async fn get_following(
    username: String,
    page: u32,
    limit: u32,
) -> Result<Vec<User>, ServerFnError> {
    let limit_i64 = limit as i64;
    let offset_i64 = (page * limit) as i64;

    let target_user = crate::storage::get_user_by_name(&username)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?
        .ok_or_else(|| dioxus::prelude::ServerFnError::new("api_err_user_not_found"))?;

    let following = crate::storage::get_following_db(&target_user.user.id, limit_i64, offset_i64)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    Ok(following.into_iter().map(|u| u.user).collect())
}



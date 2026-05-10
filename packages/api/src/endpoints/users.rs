use crate::models::*;
use dioxus::prelude::*;

#[server]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    Ok(require_auth().await.ok())
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
pub async fn set_active_playlist(
    collection_id: Option<String>,
    interval_secs: Option<i32>,
) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::users::update_user_playlist(&user.id, collection_id.as_deref(), interval_secs)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(())
}

#[server]
pub async fn get_active_playlist_items() -> Result<(Vec<Wallpaper>, i32), ServerFnError> {
    let user = require_auth().await?;
    let db_user = crate::storage::users::get_user_by_id(&user.id)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    
    let db_user = db_user.ok_or_else(|| ServerFnError::new("User not found"))?;
    let interval = db_user.user.playlist_interval_secs;
    
    if let Some(col_id) = db_user.user.active_playlist_id {
        let items = crate::storage::collections::get_collection_wallpapers_db(&col_id, 0, 100)
            .await
            .map_err(|e| ServerFnError::new(e))?;
        Ok((items.to_vec(), interval))
    } else {
        Ok((vec![], interval))
    }
}

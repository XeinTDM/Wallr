use crate::models::*;
use dioxus::prelude::*;
use crate::auth::*;

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
    if let Some(record) = existing_user
        && record.user.id != user.id {
            return Err(ServerFnError::new(
                "Email already in use by another account",
            ));
        }

    let existing_name = crate::storage::get_user_by_name(&name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    if let Some(record) = existing_name
        && record.user.id != user.id {
            return Err(ServerFnError::new(
                "api_err_username_exists",
            ));
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
pub async fn get_public_profile(username: String) -> Result<Option<PublicUser>, ServerFnError> {
    let username = username.replace("-", " ");
    let user_record_opt = crate::storage::get_user_by_name(&username)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
    Ok(user_record_opt.map(|ur| PublicUser::from(ur.user)))
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
pub async fn search_users_endpoint(query: String, limit: u32) -> Result<Vec<PublicUser>, ServerFnError> {
    crate::storage::search_users(&query, limit)
        .await
        .map(|users| users.into_iter().map(PublicUser::from).collect())
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
    
    if let Some(ref col_id) = collection_id {
        let owner_opt = crate::storage::collections::get_collection_owner(col_id)
            .await
            .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;
            
        let owner_id = match owner_opt {
            Some(id) => id,
            None => return Err(ServerFnError::new("api_err_collection_not_found")),
        };
        
        if owner_id != user.id && user.role != "admin" && user.role != "super_admin" {
            return Err(ServerFnError::new("api_err_unauthorized"));
        }
    }

    crate::storage::users::update_user_playlist(&user.id, collection_id.as_deref(), interval_secs)
        .await
        .map_err(ServerFnError::new)?;
    Ok(())
}

#[server]
pub async fn get_active_playlist_items() -> Result<(Vec<Wallpaper>, i32), ServerFnError> {
    let user = require_auth().await?;
    let db_user = crate::storage::users::get_user_by_id(&user.id)
        .await
        .map_err(ServerFnError::new)?;
    
    let db_user = db_user.ok_or_else(|| ServerFnError::new("User not found"))?;
    let interval = db_user.user.playlist_interval_secs;
    let caller_id = Some(db_user.user.id.as_str());
    let is_admin = db_user.user.role == "admin" || db_user.user.role == "super_admin";
    
    if let Some(col_id) = db_user.user.active_playlist_id {
        let items = crate::storage::collections::get_collection_wallpapers_db(&col_id, 0, 100, caller_id, is_admin)
            .await
            .map_err(ServerFnError::new)?;
        Ok((items.to_vec(), interval))
    } else {
        Ok((vec![], interval))
    }
}



use crate::models::*;
use dioxus::prelude::*;

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
    parent_id: Option<String>,
) -> Result<WallpaperComment, ServerFnError> {
    let user = require_auth().await?;
    let content = content.trim();
    if content.is_empty() {
        return Err(ServerFnError::new("api_err_comment_empty"));
    }
    if content.chars().count() > 500 {
        return Err(ServerFnError::new("api_err_comment_toolong"));
    }
    if crate::profanity::contains_forbidden_words(content).await {
        return Err(ServerFnError::new("api_err_comment_forbidden"));
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
        parent_id,
        is_pinned: false,
        is_hidden: false,
        is_edited: false,
    };

    crate::storage::add_comment_db(&comment)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Ok(Some(wp)) = crate::storage::get_wallpaper_by_id(&comment.wallpaper_id).await {
        if wp.author_id != user.id {
            let short_comment = if comment.content.chars().count() > 30 {
                let mut c = comment.content.chars().take(27).collect::<String>();
                c.push_str("...");
                c
            } else {
                comment.content.clone()
            };
            let _ = crate::storage::create_notification_db(
                &wp.author_id,
                "New Comment",
                &format!("{} commented on '{}': \"{}\"", user.name, wp.title, short_comment),
            )
            .await;
        }

        let mentions: std::collections::HashSet<&str> = comment.content
            .split_whitespace()
            .filter(|w| w.starts_with('@') && w.len() > 1)
            .map(|w| w.trim_start_matches('@').trim_end_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        for mention in mentions {
            if let Ok(Some(mentioned_user)) = crate::storage::get_user_by_name(mention).await {
                if mentioned_user.user.id != user.id && mentioned_user.user.id != wp.author_id {
                    let _ = crate::storage::create_notification_db(
                        &mentioned_user.user.id,
                        "Mentioned in Comment",
                        &format!("{} mentioned you in a comment on '{}'", user.name, wp.title),
                    )
                    .await;
                }
            }
        }
    }

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
pub async fn pin_wallpaper_comment(comment_id: String, pin: bool) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::pin_comment_db(&comment_id, &user.id, pin)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn hide_wallpaper_comment(comment_id: String, hide: bool) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::hide_comment_db(&comment_id, &user.id, hide)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn toggle_wallpaper_comments(wallpaper_id: String, disable: bool) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::toggle_wallpaper_comments_db(&wallpaper_id, &user.id, disable)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn report_wallpaper_comment(comment_id: String, reason: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::report_comment_db(&comment_id, &user.id, &user.name, &reason)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn get_comment_edit_history(comment_id: String) -> Result<Vec<crate::models::CommentEditHistory>, ServerFnError> {
    // Anyone can view edit history of public comments
    crate::storage::get_comment_edit_history_db(&comment_id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

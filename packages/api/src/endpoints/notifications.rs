use crate::models::*;
use dioxus::prelude::*;

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

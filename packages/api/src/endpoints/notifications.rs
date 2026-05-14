use crate::models::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
}

#[server]
pub async fn subscribe_push_notifications(sub: PushSubscription) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::add_push_subscription_db(&user.id, &sub.endpoint, &sub.keys.p256dh, &sub.keys.auth)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[server]
pub async fn unsubscribe_push_notifications(endpoint: String) -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    crate::storage::remove_push_subscription_db(&user.id, &endpoint)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
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



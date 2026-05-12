use crate::models::*;
use dioxus::prelude::*;
use crate::auth::*;

#[server]
pub async fn get_creator_analytics() -> Result<CreatorAnalytics, ServerFnError> {
    let user = require_auth().await?;

    crate::storage::get_creator_analytics_db(&user.name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}



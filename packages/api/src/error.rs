#[cfg(feature = "server")]
use thiserror::Error;

#[cfg(feature = "server")]
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("AI processing error: {0}")]
    Ai(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    Unexpected(#[from] anyhow::Error),
}

#[cfg(feature = "server")]
impl ApiError {
    pub fn into_server_fn_err(self) -> dioxus::prelude::ServerFnError {
        match self {
            ApiError::Database(e) => {
                eprintln!("Database error: {:?}", e);
                dioxus::prelude::ServerFnError::new("Internal Server Error")
            }
            ApiError::Io(e) => {
                eprintln!("IO error: {:?}", e);
                dioxus::prelude::ServerFnError::new("Internal Server Error")
            }
            ApiError::Serialization(e) => {
                eprintln!("Serialization error: {:?}", e);
                dioxus::prelude::ServerFnError::new("Internal Server Error")
            }
            ApiError::Unexpected(e) => {
                let err_str = e.to_string();
                if err_str.contains("Too many") || err_str.contains("User not found") || err_str.contains("Token revoked") {
                    dioxus::prelude::ServerFnError::new(err_str)
                } else {
                    eprintln!("Unexpected internal error: {:?}", e);
                    dioxus::prelude::ServerFnError::new("Internal Server Error")
                }
            }
            ApiError::Auth(msg) | ApiError::RateLimited(msg) | ApiError::BadRequest(msg) | ApiError::NotFound(msg) | ApiError::Image(msg) | ApiError::Ai(msg) => {
                dioxus::prelude::ServerFnError::new(msg)
            }
        }
    }
}

#[cfg(feature = "server")]
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, msg) = match self {
            ApiError::Database(e) => {
                eprintln!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            ApiError::Io(e) => {
                eprintln!("IO error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            ApiError::Serialization(e) => {
                eprintln!("Serialization error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            ApiError::Unexpected(e) => {
                let err_str = e.to_string();
                if err_str.contains("Too many") || err_str.contains("User not found") || err_str.contains("Token revoked") {
                    (StatusCode::BAD_REQUEST, err_str)
                } else {
                    eprintln!("Unexpected internal error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
                }
            }
            ApiError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::RateLimited(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            ApiError::BadRequest(msg) | ApiError::Image(msg) | ApiError::Ai(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        (status, msg).into_response()
    }
}

#[cfg(feature = "server")]
pub type ApiResult<T> = Result<T, ApiError>;

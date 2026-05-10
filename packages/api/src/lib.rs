//! This crate contains all shared fullstack server functions and data models.
#[cfg(feature = "server")]
pub mod ai;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod oauth;
#[cfg(feature = "server")]
pub mod processor;
#[cfg(feature = "server")]
pub mod profanity;
#[cfg(feature = "server")]
pub mod storage;

pub mod tags;


pub mod models;
pub mod auth;
#[cfg(feature = "server")]
pub mod upload;
pub mod endpoints;

pub use models::*;
pub use auth::*;
#[cfg(feature = "server")]
pub use upload::*;
pub use endpoints::wallpapers::*;
pub use endpoints::collections::*;
pub use endpoints::users::*;
pub use endpoints::social::*;
pub use endpoints::comments::*;
pub use endpoints::admin::*;
pub use endpoints::notifications::*;
pub use endpoints::analytics::*;

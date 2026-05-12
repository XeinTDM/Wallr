#![cfg(feature = "server")]

use axum::{
    Router,
    extract::{Path, Query},
    response::{IntoResponse, Redirect},
    routing::get,
};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub fn oauth_router() -> Router {
    Router::new()
        .route("/{provider}/login", get(oauth_login))
        .route("/{provider}/callback", get(oauth_callback))
}

fn get_client(provider: &str) -> Option<BasicClient> {
    let (client_id, client_secret, auth_url, token_url, redirect_url) = match provider {
        "google" => (
            std::env::var("GOOGLE_CLIENT_ID").ok()?,
            std::env::var("GOOGLE_CLIENT_SECRET").ok()?,
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "/api/oauth/google/callback",
        ),
        "github" => (
            std::env::var("GITHUB_CLIENT_ID").ok()?,
            std::env::var("GITHUB_CLIENT_SECRET").ok()?,
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
            "/api/oauth/github/callback",
        ),
        "discord" => (
            std::env::var("DISCORD_CLIENT_ID").ok()?,
            std::env::var("DISCORD_CLIENT_SECRET").ok()?,
            "https://discord.com/api/oauth2/authorize",
            "https://discord.com/api/oauth2/token",
            "/api/oauth/discord/callback",
        ),
        _ => return None,
    };

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    Some(
        BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url.to_string()).unwrap(),
            Some(TokenUrl::new(token_url.to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(format!("{}{}", app_url, redirect_url)).unwrap()),
    )
}

async fn oauth_login(Path(provider): Path<String>) -> impl IntoResponse {
    let client = match get_client(&provider) {
        Some(c) => c,
        None => return Redirect::to("/login?error=unsupported_provider").into_response(),
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(match provider.as_str() {
            "google" => "email profile".to_string(),
            "github" => "user:email".to_string(),
            "discord" => "identify email".to_string(),
            _ => "".to_string(),
        }))
        .url();

    let cookie = format!(
        "oauth_csrf={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=3600",
        csrf_token.secret()
    );

    let mut response = Redirect::to(auth_url.as_ref()).into_response();
    response
        .headers_mut()
        .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());

    response
}

#[derive(Deserialize)]
struct UserInfo {
    id: Option<String>,
    sub: Option<String>,
    email: Option<String>,
    name: Option<String>,
    login: Option<String>,
    username: Option<String>,
    picture: Option<String>,
    avatar_url: Option<String>,
    avatar: Option<String>,
}

async fn oauth_callback(
    headers: axum::http::HeaderMap,
    Path(provider): Path<String>,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let expected_state = headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|val| val.to_str().ok())
        .flat_map(|cookie_str| cookie_str.split(';'))
        .map(|s| s.trim())
        .find(|s| s.starts_with("oauth_csrf="))
        .map(|s| s.trim_start_matches("oauth_csrf=").to_string());

    if Some(query.state.clone()) != expected_state {
        return Redirect::to("/login?error=invalid_csrf_token").into_response();
    }

    let client = match get_client(&provider) {
        Some(c) => c,
        None => return Redirect::to("/login?error=unsupported_provider").into_response(),
    };

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=token_exchange_failed").into_response(),
    };

    let user_info_url = match provider.as_str() {
        "google" => "https://www.googleapis.com/oauth2/v3/userinfo",
        "github" => "https://api.github.com/user",
        "discord" => "https://discord.com/api/users/@me",
        _ => return Redirect::to("/login?error=unsupported_provider").into_response(),
    };

    let req_client = reqwest::Client::new();
    let req = req_client
        .get(user_info_url)
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "Wallr");

    let res = match req.send().await {
        Ok(r) => r,
        Err(_) => return Redirect::to("/login?error=user_info_failed").into_response(),
    };

    let user_info: UserInfo = match res.json().await {
        Ok(info) => info,
        Err(_) => return Redirect::to("/login?error=parse_user_info_failed").into_response(),
    };

    let email = match user_info.email {
        Some(e) => e,
        None => {
            if provider == "github" {
                let email_res = req_client
                    .get("https://api.github.com/user/emails")
                    .bearer_auth(token.access_token().secret())
                    .header("User-Agent", "Wallr")
                    .send()
                    .await;

                if let Ok(email_res) = email_res {
                    #[derive(Deserialize)]
                    struct GithubEmail {
                        email: String,
                        primary: bool,
                        verified: bool,
                    }
                    if let Ok(emails) = email_res.json::<Vec<GithubEmail>>().await {
                        if let Some(primary) = emails.into_iter().find(|e| e.primary && e.verified)
                        {
                            primary.email
                        } else {
                            return Redirect::to("/login?error=no_email").into_response();
                        }
                    } else {
                        return Redirect::to("/login?error=no_email").into_response();
                    }
                } else {
                    return Redirect::to("/login?error=no_email").into_response();
                }
            } else {
                return Redirect::to("/login?error=no_email").into_response();
            }
        }
    }
    .to_lowercase();

    let provider_id = user_info.id.or(user_info.sub).unwrap_or_default();
    if provider_id.is_empty() {
        return Redirect::to("/login?error=no_provider_id").into_response();
    }

    let name = user_info
        .name
        .or(user_info.login)
        .or(user_info.username)
        .unwrap_or_else(|| "User".to_string());

    let pfp_url = user_info
        .picture
        .or(user_info.avatar_url)
        .unwrap_or_else(|| {
            if provider == "discord" {
                if let Some(avatar) = user_info.avatar {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}",
                        provider_id, avatar
                    )
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        });

    let pfp_url = if pfp_url.is_empty() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(email.as_bytes());
        format!(
            "https://www.gravatar.com/avatar/{:x}?s=256&d=retro",
            hasher.finalize()
        )
    } else {
        pfp_url
    };

    let user_record = crate::storage::get_user_by_email(&email)
        .await
        .unwrap_or(None);

    let user = if let Some(existing_user) = user_record {
        crate::storage::link_oauth_account(&existing_user.user.id, &provider, &provider_id)
            .await
            .ok();
        existing_user.user
    } else {
        use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
        let salt = SaltString::generate(&mut OsRng);
        let random_pass = Uuid::new_v4().to_string();
        let password_hash = argon2::Argon2::default()
            .hash_password(random_pass.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .unwrap_or_default();

        let new_user = crate::User {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            pfp_url,
            banner_url: None,
            bio: None,
            social_links: None,
            role: "user".to_string(),
            is_banned: false,
            active_playlist_id: None,
            playlist_interval_secs: 3600,
        };

        let record = crate::UserRecord {
            user: new_user.clone(),
            password_hash,
            token_version: 1,
        };

        if crate::storage::create_user(&record).await.is_err() {
            return Redirect::to("/login?error=create_user_failed").into_response();
        }

        crate::storage::link_oauth_account(&new_user.id, &provider, &provider_id)
            .await
            .ok();

        new_user
    };

    let record = match crate::storage::get_user_by_email(&user.email).await.unwrap_or(None) {
        Some(r) => r,
        None => return Redirect::to("/login?error=user_not_found").into_response(),
    };
    let token = match crate::storage::generate_token(&record.user, record.token_version) {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=token_generation_failed").into_response(),
    };

    let cookie = format!(
        "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000",
        token
    );

    let mut response = Redirect::to("/").into_response();
    response
        .headers_mut()
        .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());

    response
}

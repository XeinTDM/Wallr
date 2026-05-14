use dioxus::prelude::*;

#[cfg(feature = "server")]
fn extract_client_ip(
    headers: &http::HeaderMap,
    connect_info: Result<
        axum::extract::ConnectInfo<std::net::SocketAddr>,
        dioxus::prelude::ServerFnError,
    >,
) -> String {
    let direct_ip = connect_info.map(|info| info.0.ip());

    let allow_headers = match &direct_ip {
        Ok(std::net::IpAddr::V4(ipv4)) => {
            let octets = ipv4.octets();
            let is_private = octets[0] == 10
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168);
            is_private || ipv4.is_loopback() || ipv4.is_link_local()
        }
        Ok(std::net::IpAddr::V6(ipv6)) => ipv6.is_loopback(),
        Err(_) => true,
    };

    if allow_headers {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
            && let Some(first_ip) = xff.split(',').next() {
                let trimmed = first_ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        if let Some(xrip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            let trimmed = xrip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    match direct_ip {
        Ok(ip) => ip.to_string(),
        Err(_) => "unknown_ip".to_string(),
    }
}

#[cfg(feature = "server")]
pub async fn require_auth() -> Result<User, dioxus::prelude::ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    FullstackContext::extract::<User, _>()
        .await
        .map_err(|_| dioxus::prelude::ServerFnError::new("api_err_unauthorized"))
}

#[cfg(feature = "server")]
pub async fn require_admin() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "admin" && user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Admins only",
        ));
    }
    Ok(user)
}

#[cfg(feature = "server")]
pub async fn require_verified_auth() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if !user.is_verified {
        return Err(dioxus::prelude::ServerFnError::new("api_err_unverified"));
    }
    Ok(user)
}

#[cfg(feature = "server")]
impl<S> axum::extract::FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::extract::cookie::CookieJar;
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get("session_token").map(|c| c.value());

        let token = match token {
            Some(t) => t,
            None => return Err((http::StatusCode::UNAUTHORIZED, "Missing session token")),
        };

        match crate::storage::verify_token(token).await {
            Ok(user) => Ok(user),
            Err(_) => Err((
                http::StatusCode::UNAUTHORIZED,
                "Invalid or expired session token",
            )),
        }
    }
}

#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use dioxus_fullstack::FullstackContext;

    if password.len() < 8 || password.len() > 128 {
        return Err(ServerFnError::new("api_err_invalid_login"));
    }

    let email = email.trim().to_lowercase();

    let headers = FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("api_err_unauthorized"))?;

    let connect_info =
        FullstackContext::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>, _>().await;
    let ip = extract_client_ip(&headers, connect_info);

    crate::storage::check_login_rate_limit(&ip, &email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let user_record_opt = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let is_valid = if let Some(user_record) = &user_record_opt {
        let hash_clone = user_record.password_hash.clone();
        let pass_clone = password.clone();
        crate::get_heavy_runtime().spawn_blocking(move || {
            let parsed_hash =
                PasswordHash::new(&hash_clone).map_err(|e| format!("Hash error: {}", e))?;
            Ok::<bool, String>(
                Argon2::default()
                    .verify_password(pass_clone.as_bytes(), &parsed_hash)
                    .is_ok(),
            )
        })
        .await
        .map_err(|_| ServerFnError::new("api_err_task"))?
        .map_err(ServerFnError::new)?
    } else {
        let pass_clone = password.clone();
        let _ = crate::get_heavy_runtime().spawn_blocking(move || {
            use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
            let salt = SaltString::generate(&mut OsRng);
            let _ = Argon2::default().hash_password(pass_clone.as_bytes(), &salt);
        })
        .await;
        false
    };

    if is_valid
        && let Some(user_record) = user_record_opt {
            if user_record.user.is_banned {
                return Err(ServerFnError::new("api_err_account_banned"));
            }

            let token =
                crate::storage::generate_token(&user_record.user, user_record.token_version)
                    .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

            if let Some(ctx) = FullstackContext::current() {
                let cookie = format!(
                    "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000",
                    token
                );
                ctx.add_response_header(
                    http::header::SET_COOKIE,
                    cookie.parse::<http::header::HeaderValue>().unwrap(),
                );
            }

            crate::storage::reset_login_rate_limit(&ip, &email).await;
            return Ok(());
        }

    Err(ServerFnError::new("api_err_invalid_login"))
}

#[server]
pub async fn register(name: String, email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use dioxus_fullstack::FullstackContext;
    use sha2::{Digest, Sha256};
    

    if password.len() < 8 || password.len() > 128 {
        return Err(ServerFnError::new(
            "Password must be between 8 and 128 characters long",
        ));
    }

    let email = email.trim().to_lowercase();

    let headers = FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("api_err_unauthorized"))?;

    let connect_info =
        FullstackContext::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>, _>().await;
    let ip = extract_client_ip(&headers, connect_info);

    crate::storage::check_register_rate_limit(&ip)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let existing_user = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if existing_user.is_some() {
        return Err(ServerFnError::new("api_err_email_exists"));
    }

    let existing_name = crate::storage::get_user_by_name(&name)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if existing_name.is_some() {
        return Err(ServerFnError::new("api_err_username_exists"));
    }

    let password_hash = crate::get_heavy_runtime().spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Hashing error: {}", e))
    })
    .await
    .map_err(|_| ServerFnError::new("api_err_task"))?
    .map_err(ServerFnError::new)?;

    let mut hasher = Sha256::new();
    hasher.update(email.as_bytes());
    let _email_hash = format!("{:x}", hasher.finalize());

    let new_user = User {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        email,
        pfp_url: format!(
            "https://api.dicebear.com/7.x/avataaars/svg?seed={}",
            uuid::Uuid::new_v4()
        ),
        banner_url: None,
        bio: None,
        social_links: None,
        role: "user".to_string(),
        is_banned: false,
        active_playlist_id: None,
        playlist_interval_secs: 300,
        email_notifs: true,
        push_notifs: false,
        download_quality: "Original (4K+)".to_string(),
        auto_download_avif: true,
        safe_search: true,
        };

    let record = UserRecord {
        user: new_user.clone(),
        password_hash,
        token_version: 1,
    };

    crate::storage::create_user(&record)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    let token = crate::storage::generate_token(&new_user, record.token_version)
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(ctx) = FullstackContext::current() {
        let cookie = format!(
            "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=2592000",
            token
        );
        ctx.add_response_header(
            http::header::SET_COOKIE,
            cookie.parse::<http::header::HeaderValue>().unwrap(),
        );
    }
    
    // Add to user_verifications and send email
    let token_str = format!("{}", uuid::Uuid::new_v4());
    if let Ok(pool) = crate::storage::get_pool() {
        let _ = sqlx::query("INSERT INTO user_verifications (user_id, is_verified, verification_token, token_expires_at) VALUES ($1, false, $2, NOW() + INTERVAL '24 hours')")
            .bind(uuid::Uuid::parse_str(&new_user.id).unwrap_or_default())
            .bind(&token_str)
            .execute(pool)
            .await;
            
        send_verification_email(&new_user.email, &token_str).await;
    }

    Ok(())
}

#[cfg(feature = "server")]
pub async fn send_verification_email(email: &str, token: &str) {
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let verify_link = format!("{}/verify-email/{}", app_url, token);

    let smtp_username = std::env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_password = std::env::var("SMTP_PASSWORD").unwrap_or_default();
    let smtp_server = std::env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@wallr.example.com".to_string());

    if !smtp_server.is_empty() && !smtp_username.is_empty() && !smtp_password.is_empty() {
        use lettre::{AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
        use lettre::message::Message;
        use lettre::transport::smtp::authentication::Credentials;
        
        let creds = Credentials::new(smtp_username, smtp_password);
        
        if let Ok(mailer) = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server) {
            let mailer = mailer.credentials(creds).build();
                
            if let Ok(email_msg) = Message::builder()
                .from(smtp_from.parse().unwrap())
                .to(email.parse().unwrap())
                .subject("Wallr - Verify Your Email")
                .body(format!(
                    "Welcome to Wallr!\n\nPlease click the link below to verify your email address:\n\n{}\n\nIf you did not create this account, please ignore this email.",
                    verify_link
                )) 
            {
                let _ = mailer.send(email_msg).await;
            }
        }
    } else {
        println!("----------------------------------------");
        println!("EMAIL VERIFICATION REQUESTED FOR: {}", email);
        println!("Verification Link: {}", verify_link);
        println!("----------------------------------------");
    }
}

#[server]
pub async fn resend_verification() -> Result<(), ServerFnError> {
    let user = require_auth().await?;
    if user.is_verified {
        return Err(ServerFnError::new("api_err_already_verified"));
    }

    let pool = crate::storage::get_pool().map_err(|e| ServerFnError::new(e.to_string()))?;
        
    let token_str = format!("{}", uuid::Uuid::new_v4());
    sqlx::query("UPDATE user_verifications SET verification_token = $1, token_expires_at = NOW() + INTERVAL '24 hours', last_resent_at = NOW() WHERE user_id = $2")
        .bind(&token_str)
        .bind(uuid::Uuid::parse_str(&user.id).unwrap_or_default())
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        
    send_verification_email(&user.email, &token_str).await;
    Ok(())
}

#[server]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    let pool = crate::storage::get_pool().map_err(|e| ServerFnError::new(e.to_string()))?;
    
    let row = sqlx::query("UPDATE user_verifications SET is_verified = true, verification_token = NULL WHERE verification_token = $1 AND token_expires_at > NOW() RETURNING user_id")
        .bind(&token)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        
    if let Some(r) = row {
        use sqlx::Row;
        if let Ok(uid) = r.try_get::<uuid::Uuid, _>("user_id") {
            crate::storage::cache::get_user_cache().remove(&uid.to_string()).await;
        }
        Ok(())
    } else {
        Err(ServerFnError::new("api_err_invalid_verification_token"))
    }
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;

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
pub async fn change_password(
    current_password: String,
    new_password: String,
) -> Result<(), ServerFnError> {
    use argon2::{
        Argon2, PasswordHash, PasswordVerifier,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use dioxus_fullstack::FullstackContext;

    if new_password.len() < 8 || new_password.len() > 128 {
        return Err(ServerFnError::new(
            "New password must be between 8 and 128 characters long",
        ));
    }

    let user = require_auth().await?;
    let user_record = crate::storage::get_user_by_id(&user.id)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?
        .ok_or_else(|| ServerFnError::new("api_err_user_not_found"))?;

    let hash_clone = user_record.password_hash.clone();
    let is_valid = crate::get_heavy_runtime().spawn_blocking(move || {
        let parsed_hash =
            PasswordHash::new(&hash_clone).map_err(|e| format!("Hash error: {}", e))?;
        Ok::<bool, String>(
            Argon2::default()
                .verify_password(current_password.as_bytes(), &parsed_hash)
                .is_ok(),
        )
    })
    .await
    .map_err(|_| ServerFnError::new("api_err_task"))?
    .map_err(ServerFnError::new)?;

    if !is_valid {
        return Err(ServerFnError::new("api_err_wrong_pwd"));
    }

    let new_password_hash = crate::get_heavy_runtime().spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Hashing error: {}", e))
    })
    .await
    .map_err(|_| ServerFnError::new("api_err_task"))?
    .map_err(ServerFnError::new)?;

    crate::storage::update_password(&user.id, &new_password_hash)
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
pub async fn revoke_sessions() -> Result<(), ServerFnError> {
    use dioxus_fullstack::FullstackContext;
    let user = require_auth().await?;
    crate::storage::revoke_all_sessions(&user.id)
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

#[cfg(feature = "server")]
pub async fn require_super_admin() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Super Admins only",
        ));
    }
    Ok(user)
}

#[cfg(feature = "server")]
pub async fn require_moderator() -> Result<User, dioxus::prelude::ServerFnError> {
    let user = require_auth().await?;
    if user.role != "moderator" && user.role != "admin" && user.role != "super_admin" {
        return Err(dioxus::prelude::ServerFnError::new(
            "Forbidden: Moderators only",
        ));
    }
    Ok(user)
}

#[server]
pub async fn request_password_reset(email: String) -> Result<(), ServerFnError> {
    let user_opt = crate::storage::get_user_by_email(&email)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

    if let Some(user_record) = user_opt {
        let token = crate::storage::create_password_reset_token_db(&user_record.user.id)
            .await
            .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())?;

        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let reset_link = format!("{}/reset-password/{}", app_url, token);

        let smtp_username = std::env::var("SMTP_USERNAME").unwrap_or_default();
        let smtp_password = std::env::var("SMTP_PASSWORD").unwrap_or_default();
        let smtp_server = std::env::var("SMTP_SERVER").unwrap_or_default();
        let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@wallr.example.com".to_string());

        if !smtp_server.is_empty() && !smtp_username.is_empty() && !smtp_password.is_empty() {
            use lettre::{AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
            use lettre::message::Message;
            use lettre::transport::smtp::authentication::Credentials;
            
            let creds = Credentials::new(smtp_username, smtp_password);
            
            let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
                .unwrap()
                .credentials(creds)
                .build();
                
            let email_msg = Message::builder()
                .from(smtp_from.parse().unwrap())
                .to(email.parse().unwrap())
                .subject("Wallr - Password Reset Request")
                .body(format!(
                    "A password reset was requested for your Wallr account.\n\nPlease click the link below to reset your password:\n\n{}\n\nIf you did not request this, you can safely ignore this email.",
                    reset_link
                ))
                .unwrap();
                
            let _ = mailer.send(email_msg).await;
        } else {
            // Fallback for development if SMTP is not configured
            println!("----------------------------------------");
            println!("PASSWORD RESET REQUESTED FOR: {}", email);
            println!("Reset Link: {}", reset_link);
            println!("----------------------------------------");
        }
    }

    Ok(())
}

#[server]
pub async fn reset_password_with_token(
    token: String,
    new_password: String,
) -> Result<(), ServerFnError> {
    let new_password_hash = crate::get_heavy_runtime().spawn_blocking(move || {
        use argon2::PasswordHasher;
        let salt = argon2::password_hash::SaltString::generate(
            &mut argon2::password_hash::rand_core::OsRng,
        );
        argon2::Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Hashing error: {}", e))
    })
    .await
    .map_err(|_| ServerFnError::new("api_err_task"))?
    .map_err(ServerFnError::new)?;

    crate::storage::consume_password_reset_token_db(&token, &new_password_hash)
        .await
        .map_err(|e| crate::error::ApiError::from(e).into_server_fn_err())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use http::HeaderMap;

    #[test]
    fn test_extract_client_ip_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.195, 198.51.100.1".parse().unwrap());
        
        let connect_info = Ok(axum::extract::ConnectInfo(
            "127.0.0.1:8080".parse().unwrap() // Loopback ip means headers are allowed
        ));

        let ip = extract_client_ip(&headers, connect_info);
        assert_eq!(ip, "203.0.113.195");
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "198.51.100.1".parse().unwrap());
        
        let connect_info = Ok(axum::extract::ConnectInfo(
            "10.0.0.5:8080".parse().unwrap() // Private ip means headers are allowed
        ));

        let ip = extract_client_ip(&headers, connect_info);
        assert_eq!(ip, "198.51.100.1");
    }

    #[test]
    fn test_extract_client_ip_direct_public() {
        let headers = HeaderMap::new();
        // If connecting from a public IP, headers should NOT be trusted to prevent spoofing
        let connect_info = Ok(axum::extract::ConnectInfo(
            "203.0.113.50:8080".parse().unwrap() 
        ));

        let ip = extract_client_ip(&headers, connect_info);
        assert_eq!(ip, "203.0.113.50");
    }

    #[test]
    fn test_extract_client_ip_spoofed() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.1.1.1".parse().unwrap());
        
        let connect_info = Ok(axum::extract::ConnectInfo(
            "203.0.113.50:8080".parse().unwrap() // Public IP, do not trust headers
        ));

        let ip = extract_client_ip(&headers, connect_info);
        assert_eq!(ip, "203.0.113.50"); // Should ignore 1.1.1.1
    }
}


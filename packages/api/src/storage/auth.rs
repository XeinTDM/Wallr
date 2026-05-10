use super::cache::{get_login_rate_limit_cache, get_register_rate_limit_cache};
use super::users::get_user_by_id;
use crate::User;

pub async fn check_login_rate_limit(ip: &str, email: &str) -> anyhow::Result<()> {
    let cache = get_login_rate_limit_cache();

    let ip_key = format!("ip:{}", ip);
    let ip_count = cache.get(&ip_key).await.unwrap_or(0);

    if ip_count >= 20 {
        anyhow::bail!("api_err_rate_limit_ip_login");
    }

    let email_key = format!("email:{}", email);
    let email_count = cache.get(&email_key).await.unwrap_or(0);

    if email_count >= 5 {
        anyhow::bail!("api_err_rate_limit_acct_login");
    }

    cache.insert(ip_key, ip_count + 1).await;
    cache.insert(email_key, email_count + 1).await;
    Ok(())
}

pub async fn check_register_rate_limit(ip: &str) -> anyhow::Result<()> {
    let cache = get_register_rate_limit_cache();
    let mut count = cache.get(ip).await.unwrap_or(0);

    if count >= 3 {
        anyhow::bail!("api_err_rate_limit_ip_accts");
    }

    count += 1;
    cache.insert(ip.to_string(), count).await;
    Ok(())
}

pub async fn reset_login_rate_limit(ip: &str, email: &str) {
    let cache = get_login_rate_limit_cache();
    cache.remove(&format!("ip:{}", ip)).await;
    cache.remove(&format!("email:{}", email)).await;
}

fn get_paseto_key() -> &'static [u8; 32] {
    static KEY: std::sync::OnceLock<[u8; 32]> = std::sync::OnceLock::new();
    KEY.get_or_init(|| {
        if let Ok(key_str) = std::env::var("PASETO_SECRET_KEY") {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(key_str.as_bytes());
            let result = hasher.finalize();
            let mut key = [0u8; 32];
            key.copy_from_slice(&result);
            key
        } else {
            #[cfg(not(debug_assertions))]
            {
                panic!("CRITICAL ERROR: PASETO_SECRET_KEY environment variable is not set. This is required in production (release mode)!");
            }
            #[cfg(debug_assertions)]
            {
                use argon2::password_hash::rand_core::{OsRng, RngCore};
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                eprintln!("WARNING: PASETO_SECRET_KEY not set. Using a randomly generated key for this session.");
                key
            }
        }
    })
}

pub fn generate_token(user: &User, token_version: i32) -> anyhow::Result<String> {
    use chrono::{Duration, Utc};
    use pasetors::{claims::Claims, keys::SymmetricKey, version4::V4};
    let key = SymmetricKey::<V4>::from(get_paseto_key())?;
    let mut claims = Claims::new()?;

    let exp = Utc::now() + Duration::try_days(30).unwrap_or_default();
    claims.expiration(&exp.to_rfc3339())?;

    claims.add_additional("user_id", user.id.clone())?;
    claims.add_additional("token_version", token_version)?;

    Ok(pasetors::local::encrypt(&key, &claims, None, None)?)
}

pub async fn verify_token(token: &str) -> anyhow::Result<User> {
    use pasetors::{
        claims::ClaimsValidationRules, keys::SymmetricKey, token::UntrustedToken, version4::V4,
    };
    let key = SymmetricKey::<V4>::from(get_paseto_key())?;
    let validation_rules = ClaimsValidationRules::new();

    let untrusted_token = UntrustedToken::<pasetors::Local, V4>::try_from(token)?;
    let trusted_token =
        pasetors::local::decrypt(&key, &untrusted_token, &validation_rules, None, None)?;
    let claims = trusted_token.payload_claims().unwrap();

    let id = claims
        .get_claim("user_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("api_err_missing_claim"))?;

    let token_version = claims
        .get_claim("token_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    let user_record = get_user_by_id(id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("api_err_user_not_found"))?;

    if user_record.token_version != token_version {
        anyhow::bail!("api_err_token_revoked");
    }

    if user_record.user.is_banned {
        anyhow::bail!("api_err_account_banned");
    }

    Ok(user_record.user)
}

pub async fn link_oauth_account(
    user_id: &str,
    provider: &str,
    provider_user_id: &str,
) -> anyhow::Result<()> {
    let pool = crate::storage::get_pool()?;
    sqlx::query!(
        "INSERT INTO user_oauth_accounts (user_id, provider, provider_user_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        user_id,
        provider,
        provider_user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

use crate::{UserRecord, Wallpaper};
use std::sync::OnceLock;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "server")]
use redis::AsyncCommands;

pub struct RedisCache<V> {
    prefix: &'static str,
    ttl_secs: u64,
    _marker: std::marker::PhantomData<V>,
}

impl<V> Clone for RedisCache<V> {
    fn clone(&self) -> Self {
        RedisCache {
            prefix: self.prefix,
            ttl_secs: self.ttl_secs,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V: Serialize + DeserializeOwned + Send + Sync + 'static> RedisCache<V> {
    pub const fn new(prefix: &'static str, ttl_secs: u64) -> Self {
        Self {
            prefix,
            ttl_secs,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "server")]
    pub async fn get(&self, key: &str) -> Option<V> {
        let pool = crate::storage::get_redis_pool().ok()?;
        let mut conn = pool.get().await.ok()?;
        let redis_key = format!("{}:{}", self.prefix, key);
        let data: String = conn.get(&redis_key).await.ok()?;
        serde_json::from_str(&data).ok()
    }
    #[cfg(not(feature = "server"))]
    pub async fn get(&self, _key: &str) -> Option<V> { None }

    #[cfg(feature = "server")]
    pub async fn insert(&self, key: String, value: V) {
        if let Ok(pool) = crate::storage::get_redis_pool() {
            if let Ok(mut conn) = pool.get().await {
                let redis_key = format!("{}:{}", self.prefix, key);
                if let Ok(data) = serde_json::to_string(&value) {
                    let _: redis::RedisResult<()> = conn.set_ex(&redis_key, data, self.ttl_secs).await;
                }
            }
        }
    }
    #[cfg(not(feature = "server"))]
    pub async fn insert(&self, _key: String, _value: V) {}

    #[cfg(feature = "server")]
    pub async fn remove(&self, key: &str) {
        if let Ok(pool) = crate::storage::get_redis_pool() {
            if let Ok(mut conn) = pool.get().await {
                let redis_key = format!("{}:{}", self.prefix, key);
                let _: redis::RedisResult<()> = conn.del(&redis_key).await;
            }
        }
    }
    #[cfg(not(feature = "server"))]
    pub async fn remove(&self, _key: &str) {}

    #[cfg(feature = "server")]
    pub fn invalidate_all(&self) {
        let prefix = self.prefix;
        tokio::spawn(async move {
            if let Ok(pool) = crate::storage::get_redis_pool() {
                if let Ok(mut conn) = pool.get().await {
                    let pattern = format!("{}:*", prefix);
                    let mut keys_to_delete = Vec::new();
                    {
                        let mut iter: redis::AsyncIter<String> = match conn.scan_match(&pattern).await {
                            Ok(it) => it,
                            Err(_) => return,
                        };
                        while let Some(key) = iter.next_item().await {
                            keys_to_delete.push(key);
                        }
                    }
                    for chunk in keys_to_delete.chunks(1000) {
                        let _: redis::RedisResult<()> = conn.del(chunk).await;
                    }
                }
            }
        });
    }
    #[cfg(not(feature = "server"))]
    pub fn invalidate_all(&self) {}
}

pub fn get_wallpaper_cache() -> RedisCache<Option<Wallpaper>> {
    static CACHE: OnceLock<RedisCache<Option<Wallpaper>>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("wp", 5 * 60)).clone()
}

pub fn get_wallpaper_list_cache() -> RedisCache<Vec<String>> {
    static CACHE: OnceLock<RedisCache<Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("wp_list", 5 * 60)).clone()
}

pub fn get_collection_cache() -> RedisCache<Vec<crate::Collection>> {
    static CACHE: OnceLock<RedisCache<Vec<crate::Collection>>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("coll", 15 * 60)).clone()
}

pub fn get_user_cache() -> RedisCache<Option<UserRecord>> {
    static CACHE: OnceLock<RedisCache<Option<UserRecord>>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("user", 15 * 60)).clone()
}

pub fn get_login_rate_limit_cache() -> RedisCache<u32> {
    static CACHE: OnceLock<RedisCache<u32>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("rl_login", 15 * 60)).clone()
}

pub fn get_register_rate_limit_cache() -> RedisCache<u32> {
    static CACHE: OnceLock<RedisCache<u32>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("rl_reg", 60 * 60)).clone()
}

pub fn get_download_rate_limit_cache() -> RedisCache<bool> {
    static CACHE: OnceLock<RedisCache<bool>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("rl_dl", 60 * 60)).clone()
}

pub fn get_trending_tags_cache() -> RedisCache<Vec<String>> {
    static CACHE: OnceLock<RedisCache<Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("trending", 15 * 60)).clone()
}

pub fn get_comment_rate_limit_cache() -> RedisCache<u32> {
    static CACHE: OnceLock<RedisCache<u32>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("rl_comment", 60)).clone()
}

pub fn get_cursor_cache() -> RedisCache<String> {
    static CACHE: OnceLock<RedisCache<String>> = OnceLock::new();
    CACHE.get_or_init(|| RedisCache::new("cursor", 15 * 60)).clone()
}

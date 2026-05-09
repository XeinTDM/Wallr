use crate::{UserRecord, Wallpaper};
use moka::future::Cache;
use std::sync::OnceLock;
use std::time::Duration;

pub fn get_wallpaper_cache() -> Cache<String, Option<Wallpaper>> {
    static CACHE: OnceLock<Cache<String, Option<Wallpaper>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(5 * 60))
                .build()
        })
        .clone()
}

pub fn get_wallpaper_list_cache() -> Cache<String, std::sync::Arc<Vec<Wallpaper>>> {
    static CACHE: OnceLock<Cache<String, std::sync::Arc<Vec<Wallpaper>>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(5 * 60))
                .build()
        })
        .clone()
}

pub fn get_collection_cache() -> Cache<String, Vec<crate::Collection>> {
    static CACHE: OnceLock<Cache<String, Vec<crate::Collection>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(10)
                .time_to_live(Duration::from_secs(15 * 60))
                .build()
        })
        .clone()
}

pub fn get_user_cache() -> Cache<String, Option<UserRecord>> {
    static CACHE: OnceLock<Cache<String, Option<UserRecord>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(15 * 60))
                .build()
        })
        .clone()
}

pub fn get_login_rate_limit_cache() -> Cache<String, u32> {
    static CACHE: OnceLock<Cache<String, u32>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(10000)
                .time_to_live(Duration::from_secs(15 * 60))
                .build()
        })
        .clone()
}

pub fn get_register_rate_limit_cache() -> Cache<String, u32> {
    static CACHE: OnceLock<Cache<String, u32>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(10000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build()
        })
        .clone()
}

pub fn get_download_rate_limit_cache() -> Cache<String, bool> {
    static CACHE: OnceLock<Cache<String, bool>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(50000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build()
        })
        .clone()
}

pub fn get_trending_tags_cache() -> Cache<u32, Vec<String>> {
    static CACHE: OnceLock<Cache<u32, Vec<String>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(10)
                .time_to_live(Duration::from_secs(15 * 60))
                .build()
        })
        .clone()
}

pub fn get_comment_rate_limit_cache() -> Cache<String, u32> {
    static CACHE: OnceLock<Cache<String, u32>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Cache::builder()
                .max_capacity(10000)
                .time_to_live(Duration::from_secs(60)) // 1 minute window
                .build()
        })
        .clone()
}

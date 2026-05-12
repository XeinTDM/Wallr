#![allow(clippy::redundant_pattern_matching, clippy::redundant_locals)]
use dioxus::prelude::*;
use std::collections::HashSet;

pub static FAVORITED_IDS: GlobalSignal<HashSet<String>> = GlobalSignal::new(HashSet::new);
pub static CHECKED_FAVORITES_IDS: GlobalSignal<HashSet<String>> = GlobalSignal::new(HashSet::new);

mod footer;
pub use footer::Footer;

mod toast;
pub use toast::*;

mod theme;
pub use theme::Theme;

mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::*;

mod wallpaper_card;
pub use wallpaper_card::WallpaperCard;

mod wallpaper_grid;
pub use wallpaper_grid::WallpaperGrid;

mod category_hero;
pub use category_hero::CategoryHero;

mod loading_screen;
pub use loading_screen::LoadingScreen;

pub mod i18n;
pub use i18n::*;

pub mod app;
pub mod views;

pub fn resolve_asset_url(url: &str) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if url.starts_with("/assets/uploads/") {
            return url.replace("/assets/uploads/", "/upload/");
        }
    }
    url.to_string()
}

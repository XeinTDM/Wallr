use dioxus::prelude::*;
use std::collections::HashSet;

pub static FAVORITED_IDS: GlobalSignal<HashSet<String>> = GlobalSignal::new(|| HashSet::new());

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

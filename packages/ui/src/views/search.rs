use crate::{CategoryHero, WallpaperGrid};
use api::search_wallpapers;
use dioxus::prelude::*;

#[component]
pub fn Search(query: String) -> Element {
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "rating".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    let mut page = use_signal(|| 0_u32);
    let mut all_wallpapers = use_signal(Vec::new);
    let mut has_more = use_signal(|| true);

    let query_clone = query.clone();
    let _fetch = use_resource(move || {
        let query = query_clone.clone();
        let filters = api::FilterOptions {
            resolution: resolution(),
            sort: sort(),
            aspect_ratio: aspect_ratio(),
            color: color(),
            ai_filter: ai_filter(),
            timeframe: timeframe(),
        };
        async move {
            if !has_more() {
                return;
            }
            let p = page();
            if let Ok(new_wps) = search_wallpapers(query, p, 20, filters).await {
                if new_wps.is_empty() {
                    has_more.set(false);
                } else {
                    all_wallpapers.with_mut(|w| w.extend_from_slice(new_wps.as_ref()));
                }
            }
        }
    });

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: "{query} wallpapers",
            breadcrumb: "Search",
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            WallpaperGrid {
                wallpapers: all_wallpapers,
                is_loading: _fetch().is_none(),
                on_end_reached: move |_| { if has_more() { page += 1 } },
                empty_message: "No wallpapers match your search.".to_string(),
                empty_submessage: "Try different keywords or explore our collections.".to_string(),
            }
        }
    }
}

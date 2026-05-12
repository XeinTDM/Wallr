use crate::{CategoryHero, WallpaperGrid};
use api::get_wallpapers_by_tag;
use dioxus::prelude::*;

#[component]
pub fn Explore(tag: String) -> Element {
    let category = use_signal(|| tag.clone());
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "rating".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    let mut cursor = use_signal(|| None::<String>);
    let mut all_wallpapers = use_signal(Vec::new);
    let mut has_more = use_signal(|| true);

    let _fetch = use_resource(move || {
        let current_cat = category();
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
            let c = cursor();
            if let Ok(new_wps) = get_wallpapers_by_tag(current_cat, c, 20, filters).await {
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
            title: "{tag} wallpapers",
            breadcrumb: "Explore",
            category,
            resolution,
            sort: sort.clone(),
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            WallpaperGrid {
                wallpapers: all_wallpapers,
                is_loading: _fetch().is_none(),
                on_end_reached: move |_| {
                    if has_more() {
                        if let Some(last) = all_wallpapers().last() {
                            let val = match sort().as_str() {
                                "downloads" => last.downloads.to_string(),
                                "rating" => last.likes.to_string(),
                                _ => last.created_at.to_rfc3339(),
                            };
                            cursor.set(Some(format!("{},{}", val, last.id)));
                        }
                    }
                },
                empty_message: "No wallpapers found in this category yet.".to_string(),
                empty_submessage: "Try exploring other categories or check back later!".to_string(),
            }
        }
    }
}

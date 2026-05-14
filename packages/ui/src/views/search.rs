use crate::{CategoryHero, WallpaperGrid};
use api::search_wallpapers;
use dioxus::prelude::*;

#[component]
pub fn Search(query: Option<String>) -> Element {
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "rating".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    let query_for_initial = query.clone();
    let initial_res = use_server_future(move || {
        let q = query_for_initial.clone().unwrap_or_default();
        async move {
            let filters = api::FilterOptions::default();
            api::search_wallpapers(q, None, 20, filters).await.unwrap_or_default()
        }
    })?;

    let mut cursor = use_signal(|| None::<String>);
    let mut all_wallpapers = use_signal(|| initial_res().map(|arc| arc.as_ref().clone()).unwrap_or_default());
    let mut has_more = use_signal(|| !initial_res().map(|arc| arc.as_ref().clone()).unwrap_or_default().is_empty());

    let query_clone = query.clone();
    let _fetch = use_resource(move || {
        let query_str = query_clone.clone().unwrap_or_default();
        let filters = api::FilterOptions {
            resolution: resolution(),
            sort: sort(),
            aspect_ratio: aspect_ratio(),
            color: color(),
            ai_filter: ai_filter(),
            timeframe: timeframe(),
            safe_search: true,
            ..Default::default()
        };
        async move {
            let c = cursor();
            let default_filters = api::FilterOptions { sort: "rating".to_string(), ..Default::default() };
            
            if c.is_none() && filters == default_filters {
                return;
            }
            if let Ok(new_wps) = search_wallpapers(query_str, c.clone(), 20, filters).await {
                if new_wps.is_empty() {
                    has_more.set(false);
                } else {
                    all_wallpapers.with_mut(|w| {
                        if c.is_none() {
                            w.clear();
                        }
                        for new_wp in new_wps.iter() {
                            if !w.iter().any(|existing: &api::Wallpaper| existing.id == new_wp.id) {
                                w.push(new_wp.clone());
                            }
                        }
                    });
                }
            }
        }
    });

    let mut is_first_mount = use_signal(|| true);
    use_effect(move || {
        let _ = category();
        let _ = resolution();
        let _ = sort();
        let _ = aspect_ratio();
        let _ = color();
        let _ = ai_filter();
        let _ = timeframe();
        
        if is_first_mount() {
            is_first_mount.set(false);
            return;
        }

        all_wallpapers.write().clear();
        cursor.set(None);
        has_more.set(true);
    });

    let title_text = match &query {
        Some(q) if !q.is_empty() => format!("{q} wallpapers"),
        _ => "Search wallpapers".to_string(),
    };

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: "{title_text}",
            breadcrumb: "Search",
            category,
            resolution,
            sort: sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            WallpaperGrid {
                wallpapers: all_wallpapers,
                is_loading: _fetch().is_none() && (cursor().is_some() || all_wallpapers().is_empty()),
                on_end_reached: move |_| {
                    if has_more()
                        && let Some(last) = all_wallpapers().last() {
                            let val = match sort().as_str() {
                                "downloads" => last.downloads.to_string(),
                                "rating" => last.likes.to_string(),
                                _ => last.created_at.to_rfc3339(),
                            };
                            cursor.set(Some(format!("{},{}", val, last.id)));
                        }
                },
                empty_message: "No wallpapers match your search.".to_string(),
                empty_submessage: "Try different keywords or explore our collections.".to_string(),
            }
        }
    }
}
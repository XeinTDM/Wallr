use crate::{CategoryHero, WallpaperGrid};
use dioxus::prelude::*;

#[component]
pub fn Latest() -> Element {
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    let safe_search_enabled = match user_ctx() { crate::app::AuthState::Authenticated(u) => u.safe_search, _ => true };
    let i18n = crate::i18n::use_i18n();
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "date".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    let initial_res = use_server_future(move || async move {
        let filters = api::FilterOptions { safe_search: safe_search_enabled, ..Default::default() };
        api::get_wallpapers(None, 20, filters)
            .await
            .unwrap_or_default()
    })?;

    let mut cursor = use_signal(|| None::<String>);
    let mut all_wallpapers = use_signal(|| {
        initial_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
    });
    let mut has_more = use_signal(|| {
        !initial_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
            .is_empty()
    });

    let _fetch = use_resource(move || {
        let current_cat = category();
        let filters = api::FilterOptions {
            resolution: resolution(),
            sort: sort(),
            aspect_ratio: aspect_ratio(),
            color: color(),
            ai_filter: ai_filter(),
            timeframe: timeframe(),
            safe_search: safe_search_enabled,
            ..Default::default()
        };
        async move {
            let c = cursor();
            let default_filters = api::FilterOptions {
                sort: "date".to_string(),
                ..Default::default()
            };

            if c.is_none() && current_cat.is_empty() && filters == default_filters {
                return;
            }

            let res = if current_cat.is_empty() {
                api::get_wallpapers(c.clone(), 20, filters).await
            } else {
                api::get_wallpapers_by_tag(current_cat, c.clone(), 20, filters).await
            };

            if let Ok(new_wps) = res {
                if new_wps.is_empty() {
                    has_more.set(false);
                } else {
                    all_wallpapers.with_mut(|w| {
                        if c.is_none() {
                            w.clear();
                        }
                        for new_wp in new_wps.iter() {
                            if !w
                                .iter()
                                .any(|existing: &api::Wallpaper| existing.id == new_wp.id)
                            {
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

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: i18n.t("latest_title"),
            breadcrumb: i18n.t("latest_breadcrumb"),
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            WallpaperGrid {
                wallpapers: all_wallpapers,
                is_loading: _fetch().is_none() && (cursor().is_some() || all_wallpapers().is_empty()),
                on_end_reached: move |_| {
                    if has_more() && let Some(last) = all_wallpapers().last() {
                        let val = match sort().as_str() {
                            "downloads" => last.downloads.to_string(),
                            "rating" => last.likes.to_string(),
                            _ => last.created_at.to_rfc3339(),
                        };
                        cursor.set(Some(format!("{},{}", val, last.id)));
                    }
                },
            }
        }
    }
}

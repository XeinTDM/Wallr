use crate::{CategoryHero, WallpaperGrid};
use api::get_wallpapers;
use dioxus::prelude::*;

#[component]
pub fn PopularSelection() -> Element {
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    let _safe_search_enabled = match user_ctx() { crate::app::AuthState::Authenticated(u) => u.safe_search, _ => true };
    let i18n = crate::i18n::use_i18n();
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "downloads".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: i18n.t("popular_title"),
            breadcrumb: i18n.t("popular_breadcrumb"),
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            div {
                class: "popular-sections",
                style: "display: flex; flex-direction: column; gap: 60px; margin-top: 20px;",

                PopularSection { title: i18n.t("popular_daily").to_string(), period: "daily", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: i18n.t("popular_weekly").to_string(), period: "weekly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: i18n.t("popular_monthly").to_string(), period: "monthly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: i18n.t("popular_yearly").to_string(), period: "yearly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: i18n.t("popular_all_time").to_string(), period: "all-time", category, resolution, sort, aspect_ratio, color, ai_filter }
            }
        }
    }
}

#[component]
fn PopularSection(
    title: String,
    period: String,
    category: Signal<String>,
    resolution: Signal<String>,
    sort: Signal<String>,
    aspect_ratio: Signal<String>,
    color: Signal<String>,
    ai_filter: Signal<String>,
) -> Element {
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    let safe_search_enabled = match user_ctx() { crate::app::AuthState::Authenticated(u) => u.safe_search, _ => true };
    let i18n = crate::i18n::use_i18n();
    let period_for_resource = period.clone();
    let wallpapers = use_resource(move || {
        let current_cat = category();
        let period_clone = period_for_resource.clone();
        let filters = api::FilterOptions {
            resolution: resolution(),
            sort: sort(),
            aspect_ratio: aspect_ratio(),
            color: color(),
            ai_filter: ai_filter(),
            timeframe: period_clone.clone(),
            safe_search: safe_search_enabled,
            ..Default::default()
        };
        async move {
            if current_cat == "all" {
                get_wallpapers(None::<String>, 4, filters).await
            } else {
                api::get_wallpapers_by_tag(current_cat, None::<String>, 4, filters).await
            }
        }
    });

    rsx! {
        div {
            class: "popular-section",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;",
                h2 { style: "font-size: 24px; font-weight: 800;", "{title}" }
                Link {
                    to: crate::app::Route::PopularGrid { period: period.clone() },
                    style: "color: var(--accent-primary); font-weight: 600; text-decoration: none;",
                    "{i18n.t(\"popular_view_all\")}"
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px;",
                match wallpapers() {
                    Some(Ok(list)) => rsx! {
                        for wp in list.iter().take(4) {
                            div {
                                key: "{wp.id}",
                                class: "preview-card glass glow-hover",
                                style: "aspect-ratio: 16/9; border-radius: 12px; overflow: hidden; position: relative;",
                                img {
                                    src: "{crate::resolve_asset_url(&wp.thumbnail_url)}",
                                    style: "width: 100%; height: 100%; object-fit: cover;"
                                }
                                div {
                                    style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 12px; background: linear-gradient(transparent, rgba(0,0,0,0.8));",
                                    span { style: "font-size: 12px; font-weight: 600; color: white;", "{wp.title}" }
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        for i in 0..4 {
                            div { key: "popular-skeleton-{period}-{i}", class: "skeleton glass", style: "aspect-ratio: 16/9; border-radius: 12px;" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PopularGrid(period: String) -> Element {
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    let safe_search_enabled = match user_ctx() { crate::app::AuthState::Authenticated(u) => u.safe_search, _ => true };
    let i18n = crate::i18n::use_i18n();
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "downloads".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(|| period.clone());
    let title_key = match period.as_str() {
        "daily" => "popular_daily",
        "weekly" => "popular_weekly",
        "monthly" => "popular_monthly",
        "yearly" => "popular_yearly",
        _ => "popular_all_time",
    };

    let mut cursor = use_signal(|| None::<String>);
    let mut all_wallpapers = use_signal(Vec::new);
    let mut has_more = use_signal(|| true);

    let _period_clone = period.clone();
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

        let period_val = _period_clone.clone();
        async move {
            let c = cursor();
            let default_filters = api::FilterOptions { sort: "downloads".to_string(), timeframe: period_val.clone(), ..Default::default() };

            let res = if current_cat == "all" || current_cat.is_empty() {
                if c.is_none() && filters == default_filters {
                    return;
                }
                get_wallpapers(c.clone(), 20, filters).await
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

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::PopularSelection {},
            title: i18n.t(title_key),
            breadcrumb: i18n.t("popular_breadcrumb"),
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
                }
            }
        }
    }
}

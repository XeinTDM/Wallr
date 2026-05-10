use crate::{CategoryHero, WallpaperGrid};
use api::get_wallpapers;
use dioxus::prelude::*;

#[component]
pub fn PopularSelection() -> Element {
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
            title: "Popular wallpapers",
            breadcrumb: "Popular",
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

                PopularSection { title: "Daily Popular", period: "daily", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: "Weekly Popular", period: "weekly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: "Monthly Popular", period: "monthly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: "Yearly Popular", period: "yearly", category, resolution, sort, aspect_ratio, color, ai_filter }
                PopularSection { title: "All Time Popular", period: "all-time", category, resolution, sort, aspect_ratio, color, ai_filter }
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
            timeframe: period_clone,
        };
        async move {
            if current_cat.is_empty() {
                get_wallpapers(0, 4, filters).await
            } else {
                api::get_wallpapers_by_tag(current_cat, 0, 4, filters).await
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
                    to: crate::app::Route::PopularGrid { period },
                    style: "color: var(--accent-primary); font-weight: 600; text-decoration: none;",
                    "View all →"
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
                            div { key: "skeleton-{i}", class: "skeleton glass", style: "aspect-ratio: 16/9; border-radius: 12px;" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PopularGrid(period: String) -> Element {
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "downloads".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(|| period.clone());

    let mut page = use_signal(|| 0_u32);
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
        };
        async move {
            if !has_more() {
                return;
            }
            let p = page();

            let res = if current_cat.is_empty() {
                get_wallpapers(p, 20, filters).await
            } else {
                api::get_wallpapers_by_tag(current_cat, p, 20, filters).await
            };

            if let Ok(new_wps) = res {
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
            home_route: crate::app::Route::PopularSelection {},
            title: "{period} Popular",
            breadcrumb: "Popular",
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
                on_end_reached: move |_| { if has_more() { page += 1 } }
            }
        }
    }
}

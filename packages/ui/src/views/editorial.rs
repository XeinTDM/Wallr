use crate::{CategoryHero, LoadingScreen, WallpaperGrid};
use dioxus::prelude::*;
use api::{get_published_editorial_collections, get_editorial_collection_wallpapers};

#[component]
pub fn Editorial() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "rating".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    let cat_id = category();
    let cat_id_for_resource = cat_id.clone();

    let collections = use_resource(move || async move { 
        get_published_editorial_collections().await 
    });

    let wallpapers = use_resource(move || {
        let cid = cat_id_for_resource.clone();
        async move {
            if cid.is_empty() {
                Ok(vec![])
            } else {
                get_editorial_collection_wallpapers(cid, 1).await
            }
        }
    });

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: i18n.t("editorial_title"),
            breadcrumb: i18n.t("editorial_breadcrumb"),
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            
            if cat_id.is_empty() {
                div {
                    style: "max-width: 1200px; margin: 0 auto; padding: 40px 20px;",
                    match collections() {
                        Some(Ok(list)) => {
                            if list.is_empty() {
                                rsx! {
                                    div {
                                        style: "text-align: center; padding: 100px 0; color: var(--text-secondary);",
                                        h2 { "{i18n.t(\"editorial_curated\")}" }
                                        p { "{i18n.t(\"editorial_coming_soon\")}" }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px;",
                                        for col in list {
                                            div {
                                                key: "{col.id}",
                                                class: "glass collection-card glow-hover",
                                                style: "border-radius: 20px; overflow: hidden; cursor: pointer; transition: all 0.3s ease;",
                                                onclick: {
                                                    let id = col.id.clone();
                                                    move |_| category.set(id.clone())
                                                },
                                                div {
                                                    style: "height: 200px; position: relative;",
                                                    if let Some(cover) = &col.cover_url {
                                                        img {
                                                            src: "{crate::resolve_asset_url(cover)}",
                                                            style: "width: 100%; height: 100%; object-fit: cover;"
                                                        }
                                                    } else {
                                                        div {
                                                            style: "width: 100%; height: 100%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center;",
                                                            span { style: "font-size: 48px; opacity: 0.1;", "🖼️" }
                                                        }
                                                    }
                                                    div {
                                                        style: "position: absolute; bottom: 12px; right: 12px; background: rgba(0,0,0,0.6); backdrop-filter: blur(8px); padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 700;",
                                                        "{col.item_count} items"
                                                    }
                                                }
                                                div {
                                                    style: "padding: 16px;",
                                                    h3 { style: "font-size: 18px; font-weight: 800; margin-bottom: 4px;", "{col.title}" }
                                                    p { style: "color: var(--text-muted); font-size: 14px; margin: 0; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;", "{col.description}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { "Error: {e}" } },
                        None => rsx! { LoadingScreen {} }
                    }
                }
            } else {
                div {
                    style: "max-width: 1600px; margin: 0 auto; padding: 40px 20px;",
                    div {
                        style: "margin-bottom: 24px;",
                        button {
                            class: "btn-secondary",
                            style: "padding: 8px 16px; border-radius: 8px; background: rgba(255,255,255,0.1); border: none; color: white; cursor: pointer;",
                            onclick: move |_| category.set(String::new()),
                            "← Back to Collections"
                        }
                    }
                    match wallpapers() {
                        Some(Ok(list)) => {
                            if list.is_empty() {
                                rsx! {
                                    div {
                                        style: "text-align: center; padding: 80px 0; color: var(--text-secondary);",
                                        "This collection is currently empty."
                                    }
                                }
                            } else {
                                let sig = use_signal(|| list.clone());
                                rsx! {
                                    WallpaperGrid {
                                        wallpapers: sig,
                                        is_loading: false
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { "Error: {e}" } },
                        None => rsx! { LoadingScreen {} }
                    }
                }
            }
        }
    }
}
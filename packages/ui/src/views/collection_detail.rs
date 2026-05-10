use crate::app::Route;
use crate::{LoadingScreen, WallpaperCard};
use api::{add_wallpaper_to_collection, get_collection_wallpapers, get_user_uploads};
use dioxus::prelude::*;

#[component]
pub fn CollectionDetail(id: String) -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut page = use_signal(|| 0_u32);
    let mut current_id = use_signal(|| id.clone());
    let mut is_add_modal_open = use_signal(|| false);

    if *current_id.peek() != id {
        current_id.set(id.clone());
        page.set(0);
    }

    let mut wallpapers = use_resource(move || {
        let cid = current_id();
        let p = page();
        async move { get_collection_wallpapers(cid, p, 50).await }
    });

    let existing_ids = match wallpapers() {
        Some(Ok(list)) => list.iter().map(|w| w.id.clone()).collect::<Vec<_>>(),
        _ => vec![],
    };

    rsx! {
        div {
            div {
                class: "container fade-in",
                style: "padding-top: var(--nav-height, 68px); padding-bottom: 80px;",

                div {
                    style: "margin-bottom: 40px; margin-top: 40px;",
                    Link {
                        to: Route::Profile {},
                        style: "color: var(--text-muted); text-decoration: none; display: inline-block; margin-bottom: 16px; font-weight: 600;",
                        "{i18n.t(\"col_detail_back\")}"
                    }
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        h1 { style: "font-size: 32px; font-weight: 900; margin: 0;", "{i18n.t(\"col_detail_title\")}" }
                        button {
                            class: "glow-hover",
                            style: "padding: 10px 24px; background: var(--accent-primary); border-radius: 12px; color: white; font-weight: 600; border: none; cursor: pointer;",
                            onclick: move |_| is_add_modal_open.set(true),
                            "{i18n.t(\"col_detail_add_btn\")}"
                        }
                    }
                }

                match wallpapers() {
                    Some(Ok(list)) => {
                        if list.is_empty() {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 80px 20px; text-align: center;",
                                    lucide_dioxus::Image { size: 48, color: "rgba(255,255,255,0.2)", class: "mb-4" }
                                    h3 { style: "font-size: 20px; font-weight: 700; margin-bottom: 8px;", "{i18n.t(\"col_detail_empty_title\")}" }
                                    p { style: "color: var(--text-muted); margin-bottom: 24px;", "{i18n.t(\"col_detail_empty_desc\")}" }
                                    Link {
                                        to: Route::Explore { tag: "all".to_string() },
                                        class: "glow-hover",
                                        style: "padding: 10px 24px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; font-weight: 600; text-decoration: none;",
                                        "{i18n.t(\"col_detail_discover\")}"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    class: "wallpaper-grid",
                                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 32px;",
                                    for wp in list.iter() {
                                        WallpaperCard {
                                            key: "{wp.id}",
                                            wallpaper: wp.clone(),
                                            on_remove: {
                                                let w_id = wp.id.clone();
                                                let c_id = current_id();
                                                move |_| {
                                                    let w_id = w_id.clone();
                                                    let c_id = c_id.clone();
                                                    spawn(async move {
                                                        let _ = api::remove_wallpaper_from_collection(c_id, w_id).await;
                                                        wallpapers.restart();
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! { div { class: "error", "{i18n.t(\"col_detail_error\")}{e}" } },
                    None => rsx! { LoadingScreen {} }
                }
            }
            AddFromUploadsModal {
                is_open: is_add_modal_open,
                collection_id: current_id(),
                existing_ids,
                on_added: move |_| {
                    wallpapers.restart();
                }
            }
        }
    }
}

#[component]
fn AddFromUploadsModal(
    is_open: Signal<bool>,
    collection_id: String,
    existing_ids: Vec<String>,
    on_added: EventHandler<()>,
) -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut just_added = use_signal(|| std::collections::HashSet::<String>::new());

    use_effect(move || {
        if !is_open() {
            just_added.set(std::collections::HashSet::new());
        }
    });
    let uploads = use_resource(move || {
        let open = is_open();
        async move {
            if open {
                get_user_uploads(0, 100).await
            } else {
                Err(dioxus::prelude::ServerFnError::new("Not open"))
            }
        }
    });

    use_effect(move || {
        #[allow(unused_variables)]
        let current_is_open = is_open();
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    if current_is_open {
                        let _ = body.set_attribute("style", "overflow: hidden;");
                    } else {
                        let _ = body.remove_attribute("style");
                    }
                }
            }
        }
    });

    if !is_open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay fade-in",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.7); backdrop-filter: blur(8px); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |e| {
                e.stop_propagation();
                is_open.set(false);
            },

            div {
                class: "glass slide-up",
                style: "width: 90%; max-width: 800px; max-height: 80vh; border-radius: 24px; padding: 32px; display: flex; flex-direction: column; gap: 24px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    h2 {
                        style: "font-size: 24px; font-weight: 800; margin: 0; color: white;",
                        "{i18n.t(\"col_detail_modal_title\")}"
                    }
                    button {
                        style: "background: none; border: none; cursor: pointer; padding: 8px;",
                        onclick: move |_| is_open.set(false),
                        lucide_dioxus::X { size: 24, color: "var(--text-muted)" }
                    }
                }

                div {
                    style: "overflow-y: auto; flex-grow: 1; padding-right: 8px;",
                    match uploads() {
                        Some(Ok(list)) => {
                            let filtered_list: Vec<_> = list.iter()
                                .filter(|wp| !existing_ids.contains(&wp.id) && !just_added().contains(&wp.id))
                                .collect();

                            if filtered_list.is_empty() {
                                rsx! {
                                    div {
                                        style: "text-align: center; padding: 40px; color: var(--text-muted);",
                                        "{i18n.t(\"col_detail_modal_empty\")}"
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 16px;",
                                        for wp in filtered_list {
                                            div {
                                                key: "{wp.id}",
                                                style: "position: relative; border-radius: 12px; overflow: hidden; cursor: pointer; aspect-ratio: 16/9;",
                                                class: "glow-hover",
                                                onclick: {
                                                    let w_id = wp.id.clone();
                                                    let c_id = collection_id.clone();
                                                    let on_added = on_added.clone();
                                                    let mut just_added_sig = just_added;
                                                    move |_| {
                                                        let w_id = w_id.clone();
                                                        let c_id = c_id.clone();
                                                        let on_added = on_added.clone();
                                                        just_added_sig.write().insert(w_id.clone());
                                                        spawn(async move {
                                                            if let Ok(_) = add_wallpaper_to_collection(c_id, w_id).await {
                                                                on_added.call(());
                                                            }
                                                        });
                                                    }
                                                },
                                                img {
                                                    src: "{crate::resolve_asset_url(&wp.thumbnail_url)}",
                                                    style: "width: 100%; height: 100%; object-fit: cover;"
                                                }
                                                div {
                                                    style: "position: absolute; inset: 0; background: linear-gradient(transparent, rgba(0,0,0,0.8)); display: flex; align-items: flex-end; padding: 12px;",
                                                    span {
                                                        style: "color: white; font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                                        "{wp.title}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        _ => rsx! {
                            div {
                                style: "display: flex; justify-content: center; padding: 40px;",
                                crate::LoadingScreen {}
                            }
                        }
                    }
                }
            }
        }
    }
}

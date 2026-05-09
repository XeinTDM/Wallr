use crate::app::Route;
use api::get_collection_wallpapers;
use dioxus::prelude::*;
use crate::{LoadingScreen, WallpaperCard};


#[component]
pub fn CollectionDetail(id: String) -> Element {
    let mut page = use_signal(|| 0_u32);
    let mut current_id = use_signal(|| id.clone());
    
    if *current_id.peek() != id {
        current_id.set(id.clone());
        page.set(0);
    }

    let wallpapers = use_resource(move || {
        let cid = current_id();
        let p = page();
        async move { get_collection_wallpapers(cid, p, 50).await }
    });

    rsx! {
        div {
            class: "container fade-in",
            style: "padding-top: var(--nav-height, 68px); padding-bottom: 80px;",
            
            div {
                style: "margin-bottom: 40px; margin-top: 40px;",
                Link {
                    to: Route::Profile {},
                    style: "color: var(--text-muted); text-decoration: none; display: inline-block; margin-bottom: 16px; font-weight: 600;",
                    "← Back to Profile"
                }
                h1 { style: "font-size: 32px; font-weight: 900;", "Collection Wallpapers" }
            }

            match wallpapers() {
                Some(Ok(list)) => {
                    if list.is_empty() {
                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 80px 20px; text-align: center;",
                                lucide_dioxus::Image { size: 48, color: "rgba(255,255,255,0.2)", class: "mb-4" }
                                h3 { style: "font-size: 20px; font-weight: 700; margin-bottom: 8px;", "Collection is empty" }
                                p { style: "color: var(--text-muted);", "Add wallpapers from the explore page." }
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
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! { div { class: "error", "Error loading collection: {e}" } },
                None => rsx! { LoadingScreen {} }
            }
        }
    }
}

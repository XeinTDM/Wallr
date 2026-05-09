use dioxus::prelude::*;
use lucide_dioxus::{Download, Heart};
use api::Wallpaper;

const CARD_CSS: Asset = asset!("/assets/styling/wallpaper_card.css");

#[derive(Props, Clone, PartialEq)]
pub struct WallpaperCardProps {
    pub wallpaper: Wallpaper,
}

#[component]
pub fn WallpaperCard(props: WallpaperCardProps) -> Element {
    let mut likes_count = use_signal(|| props.wallpaper.likes);
    let mut downloads_count = use_signal(|| props.wallpaper.downloads);
    let mut has_downloaded = use_signal(|| false);

    let is_liked = crate::FAVORITED_IDS.read().contains(&props.wallpaper.id);
    
    let toggle_id = props.wallpaper.id.clone();

    rsx! {
        document::Stylesheet { href: CARD_CSS }

        div {
            class: "wallpaper-card-wrapper",

            div {
                class: "wallpaper-card glass glow-hover",
                style: "position: relative; overflow: hidden; border-radius: 20px; background: var(--bg-secondary); transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); cursor: pointer; aspect-ratio: 16 / 10;",

                img {
                    src: "{crate::resolve_asset_url(&props.wallpaper.thumbnail_url)}",
                    alt: "{props.wallpaper.title}",
                    loading: "lazy",
                    style: "width: 100%; height: 100%; object-fit: cover; transition: transform 0.4s cubic-bezier(0.4, 0, 0.2, 1);"
                }

                div {
                    class: "card-overlay",
                    style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: flex; flex-direction: column; justify-content: space-between; padding: 20px; background: linear-gradient(to bottom, rgba(0,0,0,0.7) 0%, transparent 40%, transparent 60%, rgba(0,0,0,0.85) 100%); transition: opacity 0.15s ease-out; z-index: 2;",

                    div {
                        class: "card-top-actions",
                        style: "display: flex; justify-content: flex-end; gap: 8px; width: 100%; z-index: 5; pointer-events: auto;",
                        button {
                            class: "action-btn like-btn",
                            class: if is_liked { "liked" },
                            style: "display: flex; align-items: center; gap: 8px; backdrop-filter: blur(12px); padding: 6px 12px; border-radius: 10px; color: white; font-weight: 700; font-size: 12px; cursor: pointer; transition: all 0.2s;",
                            onclick: move |e| {
                                e.stop_propagation();
                                let mut ids = crate::FAVORITED_IDS.write();
                                if ids.contains(&toggle_id) {
                                    *likes_count.write() -= 1;
                                    ids.remove(&toggle_id);
                                } else {
                                    *likes_count.write() += 1;
                                    ids.insert(toggle_id.clone());
                                }
                                let current_id = toggle_id.clone();
                                spawn(async move {
                                    let _ = api::toggle_favorite(current_id).await;
                                });
                            },
                            Heart { size: 16, fill: if is_liked { "currentColor" } else { "none" } }
                            span { "{likes_count}" }
                        }

                        a {
                            class: "action-btn download-btn",
                            style: "display: flex; align-items: center; gap: 8px; backdrop-filter: blur(12px); padding: 6px 12px; border-radius: 10px; color: white; font-weight: 700; font-size: 12px; cursor: pointer; transition: all 0.2s;",
                            href: "/wallpaper/{props.wallpaper.id}/download",
                            target: "_blank",
                            download: "{props.wallpaper.title}",
                            onclick: move |e| {
                                e.stop_propagation();
                                if !has_downloaded() {
                                    *downloads_count.write() += 1;
                                    has_downloaded.set(true);
                                }
                            },
                            Download { size: 16 }
                            span { "{downloads_count}" }
                        }
                    }

                    div {
                        class: "card-info-bottom",
                        style: "display: flex; flex-direction: column;",
                        h3 { style: "font-size: 18px; font-weight: 800; color: white; margin-bottom: 2px; letter-spacing: -0.02em; text-shadow: 0 2px 4px rgba(0,0,0,0.3);", "{props.wallpaper.title}" }
                        p { 
                            style: "display: flex; align-items: center; gap: 4px; opacity: 0.9; font-size: 13px; color: rgba(255,255,255,0.8); font-weight: 600; pointer-events: auto; z-index: 5;",
                            lucide_dioxus::User { size: 12 }
                            a { href: "/user/{props.wallpaper.author.replace(\" \", \"-\")}", style: "color: inherit; text-decoration: none;", "{props.wallpaper.author}" }
                        }
                    }

                    Link {
                        to: "/wallpaper/{props.wallpaper.id}",
                        class: "card-click-overlay",
                        style: "position: absolute; top: 0; left: 0; right: 0; bottom: 0; z-index: 1; pointer-events: auto;",
                    }
                }
            }
        }
    }
}

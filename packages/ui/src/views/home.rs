use crate::Hero;
use api::{get_wallpapers, get_user_feed};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum FeedTab {
    Trending,
    Following,
}

#[component]
pub fn Home() -> Element {
    let mut tab = use_signal(|| FeedTab::Trending);

    // Trending State
    let mut trending_cursor = use_signal(|| None::<String>);
    let mut trending_wallpapers = use_signal(Vec::new);
    let mut trending_has_more = use_signal(|| true);

    // Following State
    let mut following_cursor = use_signal(|| None::<String>);
    let mut following_wallpapers = use_signal(Vec::new);
    let mut following_has_more = use_signal(|| true);

    let mut set_tab = move |new_tab| {
        if tab() != new_tab {
            tab.set(new_tab);
        }
    };

    let _fetch_trending = use_resource(move || async move {
        if !trending_has_more() {
            return;
        }
        let c = trending_cursor();
        if let Ok(new_wps) = get_wallpapers(c, 20, api::FilterOptions::default()).await {
            if new_wps.is_empty() {
                trending_has_more.set(false);
            } else {
                trending_wallpapers.with_mut(|w| w.extend_from_slice(new_wps.as_ref()));
            }
        }
    });

    let _fetch_following = use_resource(move || async move {
        if !following_has_more() {
            return;
        }
        let c = following_cursor();
        match get_user_feed(c, 20).await {
            Ok(new_wps) => {
                if new_wps.is_empty() {
                    following_has_more.set(false);
                } else {
                    following_wallpapers.with_mut(|w| w.extend_from_slice(new_wps.as_ref()));
                }
            }
            Err(_) => {
                // If fetching user feed fails (e.g., not logged in), stop fetching
                following_has_more.set(false);
            }
        }
    });

    let trending_color = if tab() == FeedTab::Trending { "#ffc76f" } else { "#888" };
    let trending_border = if tab() == FeedTab::Trending { "2px solid #ffc76f" } else { "2px solid transparent" };
    
    let following_color = if tab() == FeedTab::Following { "#ffc76f" } else { "#888" };
    let following_border = if tab() == FeedTab::Following { "2px solid #ffc76f" } else { "2px solid transparent" };

    rsx! {
        Hero {}

        div {
            style: "padding-top: 80px; padding-bottom: 80px;",

            div {
                class: "section-header",
                style: "margin-bottom: 2rem; text-align: left; padding: 0 2rem;",
                
                div {
                    style: "display: flex; gap: 1.5rem; border-bottom: 1px solid rgba(255,255,255,0.1);",
                    button {
                        style: "background: none; border: none; padding: 0.5rem 0; font-size: clamp(1.4rem, 1.8vw, 1.8rem); font-weight: 500; cursor: pointer; border-bottom: {trending_border}; color: {trending_color}; transition: all 0.2s;",
                        onclick: move |_| set_tab(FeedTab::Trending),
                        "Trending Now"
                    }
                    button {
                        style: "background: none; border: none; padding: 0.5rem 0; font-size: clamp(1.4rem, 1.8vw, 1.8rem); font-weight: 500; cursor: pointer; border-bottom: {following_border}; color: {following_color}; transition: all 0.2s;",
                        onclick: move |_| set_tab(FeedTab::Following),
                        "Following"
                    }
                }
            }

            if tab() == FeedTab::Trending {
                crate::WallpaperGrid {
                    wallpapers: trending_wallpapers,
                    is_loading: _fetch_trending().is_none(),
                    on_end_reached: move |_| { 
                        if trending_has_more() { 
                            if let Some(last) = trending_wallpapers().last() {
                                // Default sort for home is created_at
                                trending_cursor.set(Some(format!("{},{}", last.created_at.to_rfc3339(), last.id)));
                            }
                        } 
                    }
                }
            } else {
                if following_wallpapers().is_empty() && !_fetch_following().is_none() {
                    div {
                        style: "text-align: center; padding: 4rem 2rem; color: #888;",
                        p { "You aren't following anyone yet, or they haven't posted any wallpapers." }
                    }
                } else {
                    crate::WallpaperGrid {
                        wallpapers: following_wallpapers,
                        is_loading: _fetch_following().is_none(),
                        on_end_reached: move |_| { 
                            if following_has_more() { 
                                if let Some(last) = following_wallpapers().last() {
                                    following_cursor.set(Some(format!("{},{}", last.created_at.to_rfc3339(), last.id)));
                                }
                            } 
                        }
                    }
                }
            }
        }
    }
}

use crate::Hero;
use api::{get_user_feed, get_wallpapers};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum FeedTab {
    Trending,
    Following,
}

#[component]
pub fn Home() -> Element {
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    let safe_search_enabled = match user_ctx() { crate::app::AuthState::Authenticated(u) => u.safe_search, _ => true };
    let mut tab = use_signal(|| FeedTab::Trending);

    let initial_trending_res = use_server_future(move || async move {
        get_wallpapers(None, 20, api::FilterOptions { safe_search: safe_search_enabled, ..Default::default() })
            .await
            .unwrap_or_default()
    })?;

    let initial_following_res =
        use_server_future(
            move || async move { get_user_feed(None, 20).await.unwrap_or_default() },
        )?;

    let mut trending_cursor = use_signal(|| None::<String>);
    let mut trending_wallpapers = use_signal(|| {
        initial_trending_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
    });
    let mut trending_has_more = use_signal(|| {
        !initial_trending_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
            .is_empty()
    });

    let mut following_cursor = use_signal(|| None::<String>);
    let mut following_wallpapers = use_signal(|| {
        initial_following_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
    });
    let mut following_has_more = use_signal(|| {
        !initial_following_res()
            .map(|arc| arc.as_ref().clone())
            .unwrap_or_default()
            .is_empty()
    });

    let mut set_tab = move |new_tab| {
        if tab() != new_tab {
            tab.set(new_tab);
        }
    };

    let suggested_users =
        use_resource(move || async move { api::get_suggested_users(5).await.unwrap_or_default() });

    let _fetch_more_trending = use_resource(move || async move {
        let c = trending_cursor();
        if c.is_none() {
            return;
        }
        if !trending_has_more() {
            return;
        }

        if let Ok(new_wps) = get_wallpapers(c, 20, api::FilterOptions { safe_search: safe_search_enabled, ..Default::default() }).await {
            if new_wps.is_empty() {
                trending_has_more.set(false);
            } else {
                trending_wallpapers.with_mut(|w| {
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
    });

    let _fetch_more_following = use_resource(move || async move {
        let c = following_cursor();
        if c.is_none() {
            return;
        }
        if !following_has_more() {
            return;
        }

        match get_user_feed(c, 20).await {
            Ok(new_wps) => {
                if new_wps.is_empty() {
                    following_has_more.set(false);
                } else {
                    following_wallpapers.with_mut(|w| {
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
            Err(_) => following_has_more.set(false),
        }
    });

    let trending_color = if tab() == FeedTab::Trending {
        "#ffc76f"
    } else {
        "#888"
    };
    let trending_border = if tab() == FeedTab::Trending {
        "2px solid #ffc76f"
    } else {
        "2px solid transparent"
    };

    let following_color = if tab() == FeedTab::Following {
        "#ffc76f"
    } else {
        "#888"
    };
    let following_border = if tab() == FeedTab::Following {
        "2px solid #ffc76f"
    } else {
        "2px solid transparent"
    };

    rsx! {
        Hero {}

        div { style: "padding-top: 80px; padding-bottom: 80px;",

            div {
                class: "section-header",
                style: "margin-bottom: 2rem; text-align: left; padding: 0 2rem;",

                div { style: "display: flex; gap: 1.5rem; border-bottom: 1px solid rgba(255,255,255,0.1);",
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
                    key: "{\"trending-grid\"}",
                    wallpapers: trending_wallpapers,
                    is_loading: _fetch_more_trending().is_none() && trending_cursor().is_some(),
                    on_end_reached: move |_| {
                        if trending_has_more() && let Some(last) = trending_wallpapers().last() {
                            trending_cursor
                                .set(Some(format!("{},{}", last.created_at.to_rfc3339(), last.id)));
                        }
                    },
                }
            } else {
                if following_wallpapers().is_empty() && _fetch_more_following().is_some() {
                    div { style: "text-align: center; padding: 4rem 2rem; color: #888;",
                        p { "You aren't following anyone yet, or they haven't posted any wallpapers." }

                        if let Some(users) = suggested_users() {
                            if !users.is_empty() {
                                div { style: "margin-top: 3rem;",
                                    h3 { style: "color: white; margin-bottom: 1.5rem; font-weight: 600;",
                                        "Suggested Creators to Follow"
                                    }
                                    div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 1.5rem; max-width: 1000px; margin: 0 auto;",
                                        for user in users {
                                            Link {
                                                key: "{user.id}",
                                                to: crate::app::Route::PublicProfile {
                                                    username: user.name.replace(" ", "-"),
                                                },
                                                class: "glass glow-hover",
                                                style: "display: flex; flex-direction: column; align-items: center; padding: 2rem 1.5rem; border-radius: 20px; text-decoration: none; border: 1px solid rgba(255,255,255,0.05); transition: transform 0.2s;",
                                                img {
                                                    src: "{crate::resolve_asset_url(&user.pfp_url)}",
                                                    style: "width: 80px; height: 80px; border-radius: 50%; object-fit: cover; margin-bottom: 1rem; border: 3px solid rgba(255,255,255,0.1);",
                                                    referrerpolicy: "no-referrer",
                                                }
                                                span { style: "color: white; font-weight: 700; font-size: 1.1rem; margin-bottom: 0.5rem;",
                                                    "{user.name}"
                                                }
                                                if let Some(bio) = user.bio {
                                                    span { style: "color: var(--text-muted); font-size: 0.85rem; text-align: center; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
                                                        "{bio}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    crate::WallpaperGrid {
                        key: "{\"following-grid\"}",
                        wallpapers: following_wallpapers,
                        is_loading: _fetch_more_following().is_none() && following_cursor().is_some(),
                        on_end_reached: move |_| {
                            if following_has_more() && let Some(last) = following_wallpapers().last() {
                                following_cursor
                                    .set(Some(format!("{},{}", last.created_at.to_rfc3339(), last.id)));
                            }
                        },
                    }
                }
            }
        }
    }
}

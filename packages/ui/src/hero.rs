use dioxus::prelude::*;
use lucide_dioxus::{Search, X};

const HERO_CSS: Asset = asset!("/assets/styling/hero.css");

#[component]
pub fn Hero() -> Element {
    let mut _query = use_signal(String::new);
    let mut current_slide = use_signal(|| 0);
    let nav = use_navigator();

    let suggestions = use_resource(move || {
        let q = _query();
        async move {
            if q.is_empty() {
                return vec![];
            }
            api::search_users_endpoint(q, 5).await.unwrap_or_default()
        }
    });

    let trending_tags = use_server_future(move || async move { api::get_trending_tags(10).await })?;

    let perform_search = move |q: String| {
        if !q.trim().is_empty() {
            nav.push(format!("/search/{}", q.trim()));
        }
    };

    let trending_wallpapers = use_server_future(move || async move {
        api::get_wallpapers(None::<String>, 5, api::FilterOptions::default())
            .await
            .unwrap_or_default()
    })?;

    use_hook(|| {
        spawn(async move {
            loop {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(5000).await;
                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                current_slide.with_mut(|s| *s += 1);
            }
        });
    });

    let wallpapers_ref = trending_wallpapers.read();
    let wallpapers = wallpapers_ref.as_deref().map_or(&[][..], |v| v);
    let len = wallpapers.len();
    let current_idx = if len > 0 { current_slide() % len } else { 0 };
    let current = wallpapers.get(current_idx);

    rsx! {
        document::Stylesheet { href: HERO_CSS }

        section {
            class: "hero-intro",
            style: "padding: 1rem; width: 100%; margin-top: var(--nav-height);",
            div {
                class: "hero",
                style: "position: relative; height: 460px; border-radius: 18px; overflow: hidden; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; background: #111;",

                div {
                    class: "slides",
                    style: "position: absolute; inset: 0; z-index: 0;",
                    if len > 0 {
                        for (i, wallpaper) in wallpapers.iter().enumerate() {
                            div {
                                key: "{wallpaper.id}",
                                class: if current_idx == i { "slide active" } else { "slide" },
                                style: "position: absolute; inset: 0; background-size: cover; background-position: center; transition: opacity 0.8s ease-in-out; background-image: url({crate::resolve_asset_url(&wallpaper.image_url)});",
                                "data-index": "{i}",
                                "data-title": "{wallpaper.title}",
                            }
                        }
                    } else {
                        div {
                            class: "slide active skeleton-pulse",
                            style: "background-color: rgba(255, 255, 255, 0.1);",
                        }
                    }
                }

                div {
                    class: "center",
                    style: "position: relative; z-index: 10; text-align: center; width: 100%; max-width: 900px; padding: 0 2rem;",
                    h1 {
                        style: "font-size: clamp(1.5rem, 4vw, 2.8rem); font-weight: 600; margin-bottom: 2.5rem; line-height: 1.2; text-wrap: balance;",
                        "Discover beautiful wallpapers for every screen and mood"
                    }

                    form {
                        class: "search",
                        style: "position: relative; width: 100%; max-width: 720px; margin: 0 auto 1.5rem; display: flex; align-items: center;",
                        onsubmit: move |e| {
                            e.stop_propagation();
                            perform_search(_query());
                        },
                        span {
                            style: "position: absolute; left: 16px; opacity: 0.5; display: flex; align-items: center; pointer-events: none;",
                            Search { size: 20 }
                        }
                        input {
                            r#type: "text",
                            class: "hero-search-input",
                            style: "width: 100%; height: 53px; padding: 0 48px; background: rgba(255, 255, 255, 0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); border-radius: 12px; color: white; font-size: 16px; outline: none; transition: all 0.3s var(--transition-smooth);",
                            placeholder: "Search wallpapers...",
                            value: "{_query}",
                            oninput: move |e| _query.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    perform_search(_query());
                                }
                            }
                        }
                        if !_query().is_empty() {
                            button {
                                r#type: "button",
                                style: "position: absolute; right: 16px; opacity: 0.6; display: flex; align-items: center; background: none; border: none; color: white; cursor: pointer; padding: 4px; border-radius: 50%; transition: opacity 0.2s, background 0.2s;",
                                class: "search-clear-btn glow-hover",
                                onclick: move |_| {
                                    _query.set(String::new());
                                },
                                X { size: 20 }
                            }
                        }

                        if !_query().is_empty() && suggestions.read().as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                            div {
                                class: "glass",
                                style: "position: absolute; top: calc(100% + 8px); left: 0; right: 0; border-radius: 12px; padding: 8px; display: flex; flex-direction: column; gap: 4px; z-index: 1000; border: 1px solid rgba(255,255,255,0.1);",
                                for user in suggestions.read().as_ref().unwrap_or(&vec![]).iter() {
                                    a {
                                        key: "{user.id}",
                                        href: "/user/{user.name.replace(\" \", \"-\")}",
                                        style: "display: flex; align-items: center; gap: 12px; padding: 8px 12px; border-radius: 8px; text-decoration: none; color: white; transition: background 0.2s; text-align: left;",
                                        class: "menu-item-hover",
                                        onclick: move |_| _query.set(String::new()),
                                        img {
                                            referrerpolicy: "no-referrer",
                                            src: "{crate::resolve_asset_url(&user.pfp_url)}",
                                            style: "width: 28px; height: 28px; border-radius: 50%; object-fit: cover; border: 1px solid rgba(255,255,255,0.1);"
                                        }
                                        span { style: "font-size: 14px; font-weight: 600;", "{user.name}" }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "tags",
                        style: "display: flex; justify-content: center; flex-wrap: wrap; gap: 8px; opacity: 0.8;",
                        match trending_tags.read().as_ref() {
                            Some(Ok(tags)) if !tags.is_empty() => rsx! {
                                for tag in tags.iter().take(9) {
                                    span {
                                        key: "{tag}",
                                        class: "glass tag",
                                        onclick: {
                                            let tag = tag.clone();
                                            move |_| {
                                                nav.push(format!("/search/{}", tag));
                                            }
                                        },
                                        "{tag}"
                                    }
                                }
                            },
                            None => rsx! {
                                for i in 0..5 {
                                    span {
                                        key: "hero-skeleton-{i}",
                                        class: "glass tag skeleton-pulse",
                                        style: "width: 80px; height: 32px; color: transparent;",
                                        "..."
                                    }
                                }
                            },
                            _ => rsx! {},
                        }
                    }
                }

                div {
                    class: "footer",
                    style: "position: absolute; bottom: 2rem; left: 2rem; z-index: 10; text-align: left;",
                    if let Some(w) = current {
                        p { style: "margin: 0; font-size: 14px; opacity: 0.9;", strong { style: "font-weight: 700;", "{w.author_name}" } " - Featured wallpaper" }
                        p { style: "margin: 0; font-size: 13px; opacity: 0.6; margin-top: 2px;", "{w.title}" }
                    }
                }

                div {
                    class: "dots",
                    style: "position: absolute; bottom: 2rem; left: 50%; transform: translateX(-50%); display: flex; gap: 10px; z-index: 10;",
                    for i in 0..len {
                        button {
                            key: "{i}",
                            class: if current_idx == i { "active" },
                            style: "width: 8px; height: 8px; border-radius: 50%; border: none; cursor: pointer; padding: 0; transition: all 0.3s;",
                            "data-index": "{i}",
                            onclick: move |_| current_slide.set(i),
                        }
                    }
                }
            }
        }
    }
}

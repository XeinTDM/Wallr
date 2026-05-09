use crate::app::Route;
use api::{UserCollection, get_my_collections, create_user_collection};
use dioxus::prelude::*;
use crate::{LoadingScreen, WallpaperCard, use_toaster};


#[component]
pub fn Profile() -> Element {
    let mut active_tab = use_signal(|| "favorites".to_string());
    let user = use_context::<Signal<crate::app::AuthState>>();
    let nav = use_navigator();
    let toaster = use_toaster();

    let user_data = match user() {
        crate::app::AuthState::Loading => return rsx! { LoadingScreen {} },
        crate::app::AuthState::Unauthenticated => {
            nav.push(Route::Login {});
            return rsx! {};
        }
        crate::app::AuthState::Authenticated(u) => u,
    };

    let uploads = use_resource(move || async move { api::get_user_uploads(0, 100).await });
    let collections = use_resource(move || async move { get_my_collections().await });
    let favorites = use_resource(move || async move { api::get_user_favorites(0, 100).await });
    let analytics = use_resource(move || async move { api::get_creator_analytics().await });

    let uploads_count = match uploads() {
        Some(Ok(list)) => list.len() as u32,
        _ => 0,
    };

    let collections_count = match collections() {
        Some(Ok(list)) => list.len() as u32,
        _ => 0,
    };

    let favorites_count = match favorites() {
        Some(Ok(list)) => list.len() as u32,
        _ => 0,
    };

    let latest_upload_url = match uploads() {
        Some(Ok(list)) => list.first().map(|w| w.thumbnail_url.clone()),
        _ => None,
    };

    rsx! {
        style {
            ".edit-overlay-container {{ position: relative; cursor: pointer; }}"
            ".edit-overlay {{ position: absolute; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; opacity: 0; transition: all 0.2s ease; backdrop-filter: blur(2px); }}"
            ".edit-overlay-container:hover .edit-overlay {{ opacity: 1; }}"
            ".pfp-overlay {{ border-radius: 50%; }}"
        }
        div {
            class: "fade-in",
            style: "padding-top: var(--nav-height, 68px);",

            ProfileHeader {
                user: user_data.clone(),
                is_owner: true,
                latest_upload_url,
            }

            div {
                class: "container",
                style: "padding-bottom: 80px;",

            div {
                style: "display: flex; gap: 32px; margin-bottom: 48px; border-bottom: 1px solid rgba(255,255,255,0.1);",
                ProfileTab {
                    label: "Favorites",
                    count: favorites_count,
                    active: active_tab() == "favorites",
                    onclick: move |_| active_tab.set("favorites".into())
                }
                ProfileTab {
                    label: "Uploads",
                    count: uploads_count,
                    active: active_tab() == "uploads",
                    onclick: move |_| active_tab.set("uploads".into())
                }
                ProfileTab {
                    label: "Collections",
                    count: collections_count,
                    active: active_tab() == "collections",
                    onclick: move |_| active_tab.set("collections".into())
                }
                ProfileTab {
                    label: "Analytics",
                    count: 0,
                    active: active_tab() == "analytics",
                    onclick: move |_| active_tab.set("analytics".into())
                }
            }

            div {
                match active_tab().as_str() {
                    "favorites" | "uploads" => rsx! {
                        div {
                            class: "wallpaper-grid",
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 32px;",

                            {
                                let current_tab = active_tab();
                                let render_list = if current_tab == "uploads" {
                                    match uploads() {
                                        Some(Ok(list)) => list.as_ref().clone(),
                                        _ => vec![]
                                    }
                                } else if current_tab == "favorites" {
                                    match favorites() {
                                        Some(Ok(list)) => list.as_ref().clone(),
                                        _ => vec![]
                                    }
                                } else { vec![] };

                                let is_loading = if current_tab == "uploads" {
                                    uploads().is_none()
                                } else if current_tab == "collections" {
                                    collections().is_none()
                                } else {
                                    favorites().is_none()
                                };

                                if is_loading {
                                    rsx! {
                                        div {
                                            style: "grid-column: 1 / -1; display: flex; justify-content: center; padding: 80px 20px;",
                                            crate::LoadingScreen {}
                                        }
                                    }
                                } else if render_list.is_empty() {
                                    rsx! {
                                        div {
                                            style: "grid-column: 1 / -1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 100px 20px; text-align: center;",
                                            lucide_dioxus::Image { size: 48, color: "rgba(255,255,255,0.2)", class: "mb-4" }
                                            h3 { style: "font-size: 20px; font-weight: 700; color: white; margin-bottom: 8px;", "Nothing to see here" }
                                            p { style: "color: var(--text-muted); margin-bottom: 24px; max-width: 300px;", "You haven't added any wallpapers to this section yet." }
                                            if current_tab == "uploads" {
                                                Link {
                                                    to: Route::Upload {},
                                                    class: "glow-hover",
                                                    style: "padding: 10px 24px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; font-weight: 600; text-decoration: none;",
                                                    "Upload your first"
                                                }
                                            } else {
                                                Link {
                                                    to: Route::Explore { tag: "all".to_string() },
                                                    class: "glow-hover",
                                                    style: "padding: 10px 24px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; font-weight: 600; text-decoration: none;",
                                                    "Discover wallpapers"
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        for wp in render_list {
                                            WallpaperCard {
                                                key: "{wp.id}",
                                                wallpaper: wp.clone(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "collections" => rsx! {
                        div {
                            class: "collections-grid",
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 32px;",
                            match collections() {
                                Some(Ok(list)) if !list.is_empty() => rsx! {
                                    for col in list {
                                        CollectionCard { key: "{col.id}", collection: col }
                                    }
                                },
                                Some(Ok(_)) => rsx! {
                                    div {
                                        style: "grid-column: 1 / -1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 80px 20px; text-align: center; background: rgba(255,255,255,0.02); border-radius: 24px; border: 1px dashed rgba(255,255,255,0.1);",
                                        lucide_dioxus::FolderPlus { size: 48, color: "var(--text-muted)", style: "margin-bottom: 16px;" }
                                        h3 { style: "font-size: 20px; font-weight: 700; margin-bottom: 8px;", "No Collections Yet" }
                                        p { style: "color: var(--text-secondary); margin-bottom: 24px;", "Organize your favorite wallpapers into beautiful collections." }
                                        button {
                                            class: "glow-hover",
                                            style: "padding: 12px 24px; border-radius: 12px; background: var(--accent-primary); color: white; font-weight: 600; border: none; cursor: pointer;",
                                            onclick: move |_| {
                                                #[cfg(target_arch = "wasm32")]
                                                if let Some(win) = web_sys::window() {
                                                    if let Ok(Some(name)) = win.prompt_with_message("Enter collection name:") {
                                                        if !name.trim().is_empty() {
                                                            let mut toaster = toaster;
                                                            let mut cols = collections;
                                                            spawn(async move {
                                                                if let Ok(_) = create_user_collection(name, None, false).await {
                                                                    toaster.success("Collection created!");
                                                                    cols.restart();
                                                                } else {
                                                                    toaster.error("Failed to create collection");
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                                #[cfg(not(target_arch = "wasm32"))]
                                                {
                                                    let mut toaster = toaster;
                                                    toaster.error("Collection creation prompt not supported on desktop yet.");
                                                }
                                            },
                                            "Create Collection"
                                        }
                                    }
                                },
                                _ => rsx! { div { "Loading collections..." } }
                            }
                        }
                    },
                    "analytics" => rsx! {
                        div {
                            class: "analytics-dashboard fade-in",
                            match analytics() {
                                Some(Ok(stats)) => rsx! {
                                    div {
                                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 24px;",
                                        div {
                                            class: "glass glow-hover",
                                            style: "padding: 32px; border-radius: 24px; text-align: center;",
                                            lucide_dioxus::Image { size: 32, color: "var(--text-muted)", style: "margin-bottom: 16px; margin-inline: auto;" }
                                            h4 { style: "font-size: 16px; color: var(--text-secondary); margin-bottom: 8px;", "Total Uploads" }
                                            p { style: "font-size: 48px; font-weight: 900; color: white;", "{stats.total_uploads}" }
                                        }
                                        div {
                                            class: "glass glow-hover",
                                            style: "padding: 32px; border-radius: 24px; text-align: center;",
                                            lucide_dioxus::Heart { size: 32, color: "#ef4444", style: "margin-bottom: 16px; margin-inline: auto;" }
                                            h4 { style: "font-size: 16px; color: var(--text-secondary); margin-bottom: 8px;", "Total Likes" }
                                            p { style: "font-size: 48px; font-weight: 900; color: white;", "{stats.total_likes}" }
                                        }
                                        div {
                                            class: "glass glow-hover",
                                            style: "padding: 32px; border-radius: 24px; text-align: center;",
                                            lucide_dioxus::Download { size: 32, color: "#10b981", style: "margin-bottom: 16px; margin-inline: auto;" }
                                            h4 { style: "font-size: 16px; color: var(--text-secondary); margin-bottom: 8px;", "Total Downloads" }
                                            p { style: "font-size: 48px; font-weight: 900; color: white;", "{stats.total_downloads}" }
                                        }
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    div {
                                        style: "padding: 80px 20px; text-align: center; color: #ef4444;",
                                        "Error loading analytics: {e}"
                                    }
                                },
                                None => rsx! {
                                    div {
                                        style: "display: flex; justify-content: center; padding: 80px 20px;",
                                        crate::LoadingScreen {}
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! { div { "Tab not found" } }
                }
            }
        }
        }
    }
}

#[component]
fn CollectionCard(collection: UserCollection) -> Element {
    rsx! {
        Link {
            to: Route::CollectionDetail { id: collection.id.clone() },
            class: "glass glow-hover",
            style: "border-radius: 20px; overflow: hidden; height: 260px; position: relative; cursor: pointer; display: block; text-decoration: none;",
            if let Some(url) = &collection.cover_url {
                img {
                    src: "{crate::resolve_asset_url(url)}",
                    style: "width: 100%; height: 100%; object-fit: cover;"
                }
            } else {
                div {
                    style: "width: 100%; height: 100%; background: linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(5, 5, 5, 1) 100%); display: flex; align-items: center; justify-content: center;",
                    lucide_dioxus::Folder { size: 48, color: "rgba(255,255,255,0.2)" }
                }
            }
            div {
                style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 24px; background: linear-gradient(transparent, rgba(0,0,0,0.9));",
                h4 { style: "font-size: 20px; font-weight: 800; color: white; margin-bottom: 4px;", "{collection.name}" }
                p { style: "font-size: 14px; color: var(--text-muted);", "{collection.item_count} wallpapers" }
            }
        }
    }
}

#[component]
fn ProfileTab(
    label: String,
    count: u32,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let border_color = if active {
        "var(--accent-primary)"
    } else {
        "transparent"
    };
    let text_color = if active {
        "white"
    } else {
        "var(--text-secondary)"
    };

    rsx! {
        div {
            style: "padding: 16px 0; font-weight: 700; cursor: pointer; border-bottom: 2px solid {border_color}; color: {text_color}; transition: all 0.2s ease;",
            onclick: move |e| onclick.call(e),
            "{label} "
            span {
                style: "font-size: 14px; opacity: 0.6; margin-left: 4px;",
                "{count}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ProfileHeaderProps {
    pub user: api::User,
    pub is_owner: bool,
    pub latest_upload_url: Option<String>,
}

#[component]
pub fn ProfileHeader(props: ProfileHeaderProps) -> Element {
    let is_auth = use_context::<Signal<crate::app::AuthState>>();

    let uid_for_counts = props.user.id.clone();
    let counts = use_resource(move || {
        let uid = uid_for_counts.clone();
        async move { api::get_follow_counts(uid).await }
    });

    let mut is_following = use_signal(|| false);
    let uid_for_following = props.user.id.clone();
    let is_following_res = use_resource(move || {
        let uid = uid_for_following.clone();
        async move { api::check_is_following(uid).await }
    });

    let target_id_click = props.user.id.clone();

    use_effect(move || {
        if let Some(Ok(val)) = is_following_res() {
            is_following.set(val);
        }
    });

    let banner_bg = match &props.user.banner_url {
        Some(url) => format!("url('{}')", crate::resolve_asset_url(url)),
        None => match &props.latest_upload_url {
            Some(url) => format!("url('{}')", crate::resolve_asset_url(url)),
            None => "linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(5, 5, 5, 1) 100%)"
                .to_string(),
        },
    };

    let (followers, following) = match counts() {
        Some(Ok((f1, f2))) => (f1, f2),
        _ => (0, 0),
    };

    rsx! {
        if props.is_owner {
            Link {
                to: Route::Settings {},
                class: "edit-overlay-container",
                style: "display: block; width: 100%; height: 320px; background: {banner_bg} center/cover no-repeat; position: relative;",
                div { style: "position: absolute; inset: 0; background: linear-gradient(to bottom, transparent, var(--bg-primary)); z-index: 1;" }
                div {
                    class: "edit-overlay",
                    style: "z-index: 2;",
                    lucide_dioxus::Pen { size: 36, color: "white" }
                }
            }
        } else {
            div {
                style: "display: block; width: 100%; height: 320px; background: {banner_bg} center/cover no-repeat; position: relative;",
                div { style: "position: absolute; inset: 0; background: linear-gradient(to bottom, transparent, var(--bg-primary)); z-index: 1;" }
            }
        }

        div {
            class: "container",
            style: "margin-top: -80px; position: relative; z-index: 10;",
            div {
                style: "display: flex; flex-direction: column; gap: 24px; margin-bottom: 48px;",

                div {
                    style: "display: flex; justify-content: space-between; align-items: flex-end;",

                    if props.is_owner {
                        Link {
                            to: Route::Settings {},
                            class: "glass edit-overlay-container",
                            style: "display: block; width: 160px; height: 160px; border-radius: 50%; overflow: hidden; border: 4px solid var(--bg-primary); box-shadow: 0 10px 30px rgba(0,0,0,0.5);",
                            img { src: "{crate::resolve_asset_url(&props.user.pfp_url)}", style: "width: 100%; height: 100%; object-fit: cover;" }
                            div {
                                class: "edit-overlay pfp-overlay",
                                lucide_dioxus::Pen { size: 28, color: "white" }
                            }
                        }
                    } else {
                        div {
                            class: "glass",
                            style: "display: block; width: 160px; height: 160px; border-radius: 50%; overflow: hidden; border: 4px solid var(--bg-primary); box-shadow: 0 10px 30px rgba(0,0,0,0.5);",
                            img { src: "{crate::resolve_asset_url(&props.user.pfp_url)}", style: "width: 100%; height: 100%; object-fit: cover;" }
                        }
                    }

                    if props.is_owner {
                        div {
                            style: "display: flex; gap: 12px; margin-bottom: 16px;",
                            Link {
                                to: Route::Settings {},
                                class: "glass glow-hover",
                                style: "padding: 10px 20px; border-radius: 12px; color: white; font-weight: 600; font-size: 14px; border: 1px solid rgba(255,255,255,0.1); text-decoration: none;",
                                "Edit Profile"
                            }
                        }
                    } else {
                        if let crate::app::AuthState::Authenticated(_) = is_auth() {
                            div {
                                style: "display: flex; gap: 12px; margin-bottom: 16px;",
                                button {
                                    class: "glass glow-hover",
                                    style: format!("padding: 10px 24px; border-radius: 12px; color: {}; font-weight: 600; font-size: 14px; border: 1px solid {}; background: {}; cursor: pointer; transition: all 0.2s;", if is_following() { "var(--text-primary)" } else { "white" }, if is_following() { "rgba(255,255,255,0.2)" } else { "var(--accent-primary)" }, if is_following() { "rgba(255,255,255,0.1)" } else { "var(--accent-primary)" }),
                                    onclick: move |_| {
                                        let target_id = target_id_click.clone();
                                        let current_following = is_following();
                                        is_following.toggle();
                                        spawn(async move {
                                            if current_following {
                                                let _ = api::unfollow_user(target_id).await;
                                            } else {
                                                let _ = api::follow_user(target_id).await;
                                            }
                                        });
                                    },
                                    if is_following() { "Following" } else { "Follow" }
                                }
                            }
                        }
                    }
                }

                div {
                    h1 { style: "font-size: 40px; font-weight: 900; margin-bottom: 8px; letter-spacing: -0.02em;", "{props.user.name}" }
                    div {
                        style: "color: var(--text-secondary); font-size: 15px; display: flex; align-items: center; gap: 12px;",
                        span { "{props.user.email}" }
                        span { style: "opacity: 0.5;", "•" }
                        span { style: "font-weight: 600; color: white;", "{followers}" } span { "Followers" }
                        span { style: "opacity: 0.5;", "•" }
                        span { style: "font-weight: 600; color: white;", "{following}" } span { "Following" }
                    }
                    if let Some(bio) = &props.user.bio {
                        p { style: "color: var(--text-muted); font-size: 15px; margin-top: 16px; line-height: 1.5; max-width: 600px;", "{bio}" }
                    }
                    if let Some(socials) = &props.user.social_links {
                        div {
                            style: "display: flex; gap: 16px; margin-top: 16px;",
                            for (platform, url) in socials.iter() {
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    style: "color: var(--text-secondary); text-decoration: none; display: flex; align-items: center; gap: 6px; font-size: 14px; transition: color 0.2s;",
                                    class: "glow-hover-text",
                                    "{platform}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

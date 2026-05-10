use crate::views::FollowsModal;
use crate::views::profile::{ProfileHeader, render_profile_tab};
use crate::{LoadingScreen, WallpaperCard};
use api::{get_public_profile, get_public_uploads};
use dioxus::prelude::*;

#[component]
pub fn PublicProfile(username: String) -> Element {
    let mut current_username = use_signal(|| username.clone());
    if *current_username.peek() != username {
        current_username.set(username.clone());
    }

    let profile = use_resource(move || {
        let uname = current_username();
        async move { get_public_profile(uname).await }
    });

    let uploads = use_resource(move || {
        let uname = current_username();
        async move { get_public_uploads(uname, 0, 100).await }
    });

    let mut is_follows_modal_open = use_signal(|| false);
    let mut follows_modal_type = use_signal(|| String::from("followers"));

    rsx! {
        div {
            div {
                class: "fade-in",
            style: "padding-top: var(--nav-height, 68px);",

            match profile() {
                Some(Ok(Some(user_data))) => {
                    let auth_state = use_context::<Signal<crate::app::AuthState>>();
                    let is_owner = match auth_state() {
                        crate::app::AuthState::Authenticated(u) => u.id == user_data.id,
                        _ => false,
                    };

                    let latest_upload_url = match uploads() {
                        Some(Ok(list)) => list.first().map(|w| w.thumbnail_url.clone()),
                        _ => None,
                    };
                    let uploads_count = uploads().and_then(|res| res.ok()).map(|list| list.len() as u32);

                    rsx! {
                        ProfileHeader {
                            user: user_data.clone(),
                            is_owner,
                            latest_upload_url,
                            on_followers_click: move |_| {
                                follows_modal_type.set(String::from("followers"));
                                is_follows_modal_open.set(true);
                            },
                            on_following_click: move |_| {
                                follows_modal_type.set(String::from("following"));
                                is_follows_modal_open.set(true);
                            },
                        }

                        div {
                            class: "container",
                            style: "padding-bottom: 80px;",

                            div {
                                style: "display: flex; gap: 32px; margin-bottom: 48px; border-bottom: 1px solid rgba(255,255,255,0.1);",
                                {render_profile_tab("Uploads", uploads_count, true, move |_| {})}
                            }

                            div {
                                class: "wallpaper-grid",
                                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 32px;",
                                match uploads() {
                                    Some(Ok(list)) => {
                                        if list.is_empty() {
                                            rsx! {
                                                div {
                                                    style: "grid-column: 1 / -1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 100px 20px; text-align: center;",
                                                    lucide_dioxus::Image { size: 48, color: "rgba(255,255,255,0.2)", class: "mb-4" }
                                                    h3 { style: "font-size: 20px; font-weight: 700; color: white; margin-bottom: 8px;", "Nothing to see here" }
                                                    p { style: "color: var(--text-muted); margin-bottom: 24px; max-width: 300px;", "This user hasn't uploaded any public wallpapers yet." }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                for wp in list.iter() {
                                                    WallpaperCard {
                                                        key: "{wp.id}",
                                                        wallpaper: wp.clone(),
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        div {
                                            style: "grid-column: 1 / -1; display: flex; justify-content: center; padding: 80px 20px;",
                                            crate::LoadingScreen {}
                                        }
                                    }
                                }
                            }
                        }
                        FollowsModal {
                            is_open: is_follows_modal_open,
                            modal_type: follows_modal_type(),
                            username: user_data.name.clone(),
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    div {
                        style: "padding: 120px 32px; text-align: center;",
                        h1 { "User not found" }
                    }
                },
                Some(Err(e)) => rsx! {
                    div {
                        style: "padding: 120px 32px; text-align: center; color: #ef4444;",
                        "Error loading profile: {e}"
                    }
                },
                None => rsx! { LoadingScreen {} }
            }
            }
        }
    }
}

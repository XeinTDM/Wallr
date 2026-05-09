use crate::views::profile::ProfileHeader;
use api::{get_public_profile, get_public_uploads};
use dioxus::prelude::*;
use crate::{LoadingScreen, WallpaperCard};


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

    rsx! {
        div {
            class: "fade-in",
            style: "padding-top: var(--nav-height, 68px);",

            match profile() {
                Some(Ok(Some(user_data))) => {
                    let latest_upload_url = match uploads() {
                        Some(Ok(list)) => list.first().map(|w| w.thumbnail_url.clone()),
                        _ => None,
                    };
                    let uploads_count = match uploads() {
                        Some(Ok(list)) => list.len() as u32,
                        _ => 0,
                    };

                    rsx! {
                        ProfileHeader {
                            user: user_data.clone(),
                            is_owner: false,
                            latest_upload_url,
                        }

                        div {
                            class: "container",
                            style: "padding-bottom: 80px;",

                            div {
                                style: "display: flex; gap: 32px; margin-bottom: 48px; border-bottom: 1px solid rgba(255,255,255,0.1);",
                                div {
                                    style: "padding: 16px 0; font-weight: 700; border-bottom: 2px solid var(--accent-primary); color: white; transition: all 0.2s ease;",
                                    "Uploads "
                                    span {
                                        style: "font-size: 14px; opacity: 0.6; margin-left: 4px;",
                                        "{uploads_count}"
                                    }
                                }
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

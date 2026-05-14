use crate::LoadingScreen;
use crate::app::Route;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Users};

#[component]
pub fn UserFollowers(username: String) -> Element {
    rsx! {
        FollowsPage {
            username: username.clone(),
            modal_type: "followers".to_string(),
        }
    }
}

#[component]
pub fn UserFollowing(username: String) -> Element {
    rsx! {
        FollowsPage {
            username: username.clone(),
            modal_type: "following".to_string(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FollowsPageProps {
    username: String,
    modal_type: String, // "followers" or "following"
}

#[component]
fn FollowsPage(props: FollowsPageProps) -> Element {
    let page = use_signal(|| 0u32);
    let username = props.username.clone();
    let m_type = props.modal_type.clone();

    let follows_res = use_resource(move || {
        let uname = username.clone();
        let p = page();
        let mt = m_type.clone();
        async move {
            if mt == "followers" {
                api::get_followers(uname, p, 50).await
            } else {
                api::get_following(uname, p, 50).await
            }
        }
    });

    rsx! {
        div {
            class: "container",
            style: "padding: 120px 0 80px; display: flex; justify-content: center;",

            div {
                class: "glass",
                style: "width: 100%; max-width: 600px; border-radius: 24px; display: flex; flex-direction: column; overflow: hidden; border: 1px solid rgba(255,255,255,0.1);",

                div {
                    style: "display: flex; align-items: center; gap: 16px; padding: 24px; border-bottom: 1px solid rgba(255,255,255,0.05);",
                    Link {
                        to: Route::PublicProfile { username: props.username.clone() },
                        class: "glow-hover",
                        style: "display: flex; align-items: center; justify-content: center; width: 40px; height: 40px; border-radius: 50%; background: rgba(255,255,255,0.05); color: white; text-decoration: none;",
                        ArrowLeft { size: 20 }
                    }
                    h1 {
                        style: "font-size: 24px; font-weight: 800; margin: 0; text-transform: capitalize; display: flex; align-items: center; gap: 12px;",
                        Users { size: 24, color: "var(--accent-primary)" }
                        "{props.username}'s {props.modal_type}"
                    }
                }

                div {
                    style: "flex: 1; overflow-y: auto; padding: 24px; min-height: 400px;",

                    match follows_res() {
                        Some(Ok(list)) => {
                            if list.is_empty() {
                                rsx! {
                                    div {
                                        style: "padding: 40px 0; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; align-items: center; gap: 16px;",
                                        Users { size: 48, color: "var(--text-muted)" }
                                        if props.modal_type == "followers" { "No followers yet." } else { "Not following anyone yet." }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 12px;",
                                        for user in list {
                                            UserListItem {
                                                key: "{user.id}",
                                                user: user,
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { style: "color: #ef4444;", "Error: {e}" } },
                        None => rsx! {
                            div {
                                style: "display: flex; justify-content: center; padding: 40px;",
                                LoadingScreen {}
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct UserListItemProps {
    user: api::PublicUser,
}

#[component]
fn UserListItem(props: UserListItemProps) -> Element {
    let user = props.user;
    rsx! {
        Link {
            to: Route::PublicProfile { username: user.name.clone() },
            class: "menu-item-hover",
            style: "display: flex; align-items: center; padding: 16px; border-radius: 16px; text-decoration: none; border: 1px solid rgba(255,255,255,0.05); background: rgba(0,0,0,0.2); transition: all 0.2s;",
            img {
                referrerpolicy: "no-referrer",
                src: "{crate::resolve_asset_url(&user.pfp_url)}",
                style: "width: 56px; height: 56px; border-radius: 50%; object-fit: cover; border: 2px solid rgba(255,255,255,0.1); margin-right: 20px;"
            }
            div {
                style: "display: flex; flex-direction: column; flex: 1; min-width: 0;",
                span { style: "font-size: 16px; font-weight: 700; color: white; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{user.name}" }
                if let Some(bio) = user.bio {
                    span { style: "font-size: 14px; color: var(--text-secondary); margin-top: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{bio}" }
                }
            }
        }
    }
}

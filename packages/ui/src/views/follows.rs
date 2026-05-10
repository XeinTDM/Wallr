use crate::LoadingScreen;
use crate::app::Route;
use api::User;
use dioxus::prelude::*;
use lucide_dioxus::X;

#[derive(Props, Clone, PartialEq)]
pub struct FollowsModalProps {
    pub username: String,
    pub modal_type: String, // "followers" or "following"
    pub is_open: Signal<bool>,
}

#[component]
pub fn FollowsModal(props: FollowsModalProps) -> Element {
    let mut page = use_signal(|| 0u32);
    let mut is_open = props.is_open;

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

    use_effect(move || {
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
                style: "width: 90%; max-width: 500px; max-height: 80vh; border-radius: 24px; display: flex; flex-direction: column; overflow: hidden; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                div {
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 24px; border-bottom: 1px solid rgba(255,255,255,0.05);",
                    h2 {
                        style: "font-size: 20px; font-weight: 800; margin: 0; text-transform: capitalize;",
                        "{props.modal_type}"
                    }
                    button {
                        class: "glow-hover",
                        style: "background: none; border: none; color: var(--text-muted); cursor: pointer; display: flex; align-items: center; justify-content: center; padding: 8px; border-radius: 50%;",
                        onclick: move |_| is_open.set(false),
                        X { size: 20 }
                    }
                }

                div {
                    style: "flex: 1; overflow-y: auto; padding: 24px;",

                    match follows_res() {
                        Some(Ok(list)) => {
                            if list.is_empty() {
                                rsx! {
                                    div {
                                        style: "padding: 40px 0; text-align: center; color: var(--text-muted);",
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
                                                on_click: move |_| {
                                                    is_open.set(false);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { "Error: {e}" } },
                        None => rsx! { LoadingScreen {} },
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct UserListItemProps {
    user: User,
    on_click: EventHandler<MouseEvent>,
}

#[component]
fn UserListItem(props: UserListItemProps) -> Element {
    let user = props.user;
    rsx! {
        Link {
            to: Route::PublicProfile { username: user.name.clone() },
            class: "menu-item-hover",
            style: "display: flex; align-items: center; padding: 12px; border-radius: 12px; text-decoration: none; border: 1px solid transparent; transition: all 0.2s;",
            onclick: move |e| props.on_click.call(e),
            img {
                src: "{crate::resolve_asset_url(&user.pfp_url)}",
                style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover; border: 2px solid rgba(255,255,255,0.05); margin-right: 16px;"
            }
            div {
                style: "display: flex; flex-direction: column; flex: 1; min-width: 0;",
                span { style: "font-size: 15px; font-weight: 700; color: white; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{user.name}" }
                if let Some(bio) = user.bio {
                    span { style: "font-size: 13px; color: var(--text-secondary); margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{bio}" }
                }
            }
        }
    }
}

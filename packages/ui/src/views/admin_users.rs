use crate::app::{AuthState, Route};
use api::{admin_ban_user, admin_bulk_delete_users, get_recent_users};
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Gavel, Trash2, Undo2, Users};

#[component]
pub fn AdminUsers() -> Element {
    let i18n = crate::i18n::use_i18n();
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = use_navigator();

    let mut bulk_hours = use_signal(|| 24_u32);
    let mut bulk_pattern = use_signal(String::new);
    let mut status_msg = use_signal(String::new);

    let (is_allowed, user_role, current_user_id) = match auth_state() {
        AuthState::Authenticated(u) => (
            u.role == "admin" || u.role == "super_admin" || u.role == "moderator",
            u.role.clone(),
            u.id.clone(),
        ),
        AuthState::Loading => return rsx! { crate::LoadingScreen {} },
        AuthState::Unauthenticated => (false, String::new(), String::new()),
    };

    if !is_allowed {
        nav.push(Route::Home {});
        return rsx! { div {} };
    }

    let mut users_res = use_resource(move || async move { get_recent_users(50).await });

    use_effect(move || {
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            loop {
                gloo_timers::future::TimeoutFuture::new(30_000).await;
                users_res.restart();
            }
        });
    });

    rsx! {
        div {
            class: "container fade-in",
            style: "padding: 120px 0 80px;",

            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 40px;",
                div {
                    style: "display: flex; align-items: center; gap: 16px;",
                    Link {
                        to: Route::Admin {},
                        class: "glass glow-hover",
                        style: "padding: 10px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center; color: white; text-decoration: none;",
                        ArrowLeft { size: 24 }
                    }
                    h1 {
                        style: "font-size: 36px; font-weight: 900; margin: 0; display: flex; align-items: center; gap: 16px;",
                        Users { size: 36, color: "var(--accent-primary)" }
                        "{i18n.t(\"admin_user_mgmt\")}"
                    }
                }
                div {
                    style: "display: flex; gap: 12px;",
                    div {
                        style: "padding: 8px 16px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.05); background: rgba(52, 211, 153, 0.1); color: #6ee7b7; display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 13px;",
                        div {
                            style: "width: 8px; height: 8px; border-radius: 50%; background: #34d399; box-shadow: 0 0 8px #34d399; animation: pulse 2s infinite;"
                        }
                        "{i18n.t(\"admin_live_updates\")}"
                    }
                }
            }

            div {
                style: "display: grid; grid-template-columns: 2fr 1fr; gap: 24px; margin-bottom: 48px;",

                div {
                    class: "glass",
                    style: "padding: 32px; border-radius: 24px;",
                    h2 {
                        style: "margin-bottom: 24px; display: flex; align-items: center; gap: 12px; font-size: 20px;",
                        Users { size: 24, color: "#60a5fa" }
                        "{i18n.t(\"admin_recent_users_list\")}"
                    }
                    match users_res() {
                        Some(Ok(users)) => {
                            rsx! {
                                table {
                                    style: "width: 100%; border-collapse: collapse; text-align: left;",
                                    thead {
                                        tr {
                                            style: "border-bottom: 1px solid rgba(255,255,255,0.1);",
                                            th { style: "padding: 12px; color: var(--text-secondary); font-weight: 600; font-size: 13px;", "User" }
                                            th { style: "padding: 12px; color: var(--text-secondary); font-weight: 600; font-size: 13px;", "ID" }
                                            th { style: "padding: 12px; color: var(--text-secondary); font-weight: 600; font-size: 13px;", "Role" }
                                            th { style: "padding: 12px; color: var(--text-secondary); font-weight: 600; font-size: 13px;", "Status" }
                                            th { style: "padding: 12px; color: var(--text-secondary); font-weight: 600; font-size: 13px; text-align: right;", "Actions" }
                                        }
                                    }
                                    tbody {
                                        for user in users {
                                            UserRow {
                                                user: user.clone(),
                                                current_user_id: current_user_id.clone(),
                                                on_action: {
                                                    let uid = user.id.clone();
                                                    let is_banned = user.is_banned;
                                                    move |_| {
                                                        let uid2 = uid.clone();
                                                        spawn(async move {
                                                            if admin_ban_user(uid2, !is_banned).await.is_ok() {
                                                                users_res.restart();
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { "Error: {e}" } },
                        None => rsx! { div { "{i18n.t(\"loading\")}" } }
                    }
                }

                if user_role == "super_admin" {
                    div {
                        class: "glass",
                        style: "padding: 32px; border-radius: 24px; height: fit-content;",
                    h2 {
                        style: "margin-bottom: 24px; display: flex; align-items: center; gap: 12px; font-size: 20px;",
                        Trash2 { size: 24, color: "#ef4444" }
                        "{i18n.t(\"admin_advanced_moderation\")}"
                    }
                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",
                        p { style: "color: var(--text-secondary); font-size: 14px;", "{i18n.t(\"admin_bulk_delete_desc\")}" }

                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label { style: "font-size: 14px; font-weight: 600; color: var(--text-secondary);", "Created in the last (hours):" }
                            input {
                                type: "number",
                                class: "glass",
                                style: "padding: 12px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white;",
                                value: "{bulk_hours}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        bulk_hours.set(v);
                                    }
                                }
                            }
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label { style: "font-size: 14px; font-weight: 600; color: var(--text-secondary);", "Username Regex (Optional):" }
                            input {
                                type: "text",
                                class: "glass",
                                placeholder: "e.g., ^spam_.*",
                                style: "padding: 12px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white;",
                                value: "{bulk_pattern}",
                                oninput: move |e| bulk_pattern.set(e.value().clone())
                            }
                        }

                        button {
                            class: "glow-hover",
                            style: "margin-top: 8px; padding: 12px; border-radius: 12px; background: rgba(239, 68, 68, 0.2); border: 1px solid rgba(239, 68, 68, 0.5); color: #fca5a5; font-weight: 700; cursor: pointer;",
                            onclick: move |_| async move {
                                let pat = bulk_pattern();
                                let pat_opt = if pat.trim().is_empty() { None } else { Some(pat) };
                                match admin_bulk_delete_users(bulk_hours(), pat_opt).await {
                                    Ok(count) => {
                                        status_msg.set(format!("Successfully deleted {} user(s).", count));
                                        users_res.restart();
                                    },
                                    Err(e) => status_msg.set(format!("Error: {}", e)),
                                }
                            },
                            "{i18n.t(\"admin_bulk_delete\")}"
                        }

                        if !status_msg().is_empty() {
                            div {
                                style: "margin-top: 8px; font-size: 14px; color: var(--accent-primary); font-weight: 600;",
                                "{status_msg}"
                            }
                        }
                    }
                    }
                }
            }
        }
    }
}

#[component]
fn UserRow(user: api::User, current_user_id: String, on_action: EventHandler<String>) -> Element {
    let i18n = crate::i18n::use_i18n();
    let is_banned = user.is_banned;

    rsx! {
        tr {
            style: "border-bottom: 1px solid rgba(255,255,255,0.05); transition: background 0.2s;",
            class: "table-row-hover",
            td {
                style: "padding: 12px;",
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    img {
                        referrerpolicy: "no-referrer",
                        src: "{crate::resolve_asset_url(&user.pfp_url)}",
                        style: "width: 36px; height: 36px; border-radius: 50%; object-fit: cover;"
                    }
                    div {
                        style: "display: flex; flex-direction: column;",
                        span { style: "font-weight: 600; color: white;", "{user.name}" }
                        span { style: "font-size: 12px; color: var(--text-muted);", "{user.email}" }
                    }
                }
            }
            td {
                style: "padding: 12px; font-family: monospace; font-size: 12px; color: var(--text-muted);",
                "{user.id}"
            }
            td {
                style: "padding: 12px;",
                span {
                    style: format!("font-size: 11px; padding: 4px 8px; border-radius: 6px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; {}", match user.role.as_str() {
                        "super_admin" => "background: rgba(139, 92, 246, 0.2); color: #c4b5fd; border: 1px solid rgba(139, 92, 246, 0.3);",
                        "admin" => "background: rgba(245, 158, 11, 0.2); color: #fcd34d; border: 1px solid rgba(245, 158, 11, 0.3);",
                        "moderator" => "background: rgba(16, 185, 129, 0.2); color: #6ee7b7; border: 1px solid rgba(16, 185, 129, 0.3);",
                        _ => "background: rgba(255,255,255,0.05); color: var(--text-secondary); border: 1px solid rgba(255,255,255,0.1);",
                    }),
                    "{user.role}"
                }
            }
            td {
                style: "padding: 12px;",
                if is_banned {
                    span { style: "font-size: 11px; padding: 4px 8px; border-radius: 6px; background: rgba(239, 68, 68, 0.2); color: #fca5a5; font-weight: 700; border: 1px solid rgba(239, 68, 68, 0.3);", "{i18n.t(\"admin_status_banned\")}" }
                } else {
                    span { style: "font-size: 11px; padding: 4px 8px; border-radius: 6px; background: rgba(52, 211, 153, 0.1); color: #6ee7b7; font-weight: 700; border: 1px solid rgba(52, 211, 153, 0.2);", "{i18n.t(\"admin_status_active\")}" }
                }
            }
            td {
                style: "padding: 12px; text-align: right;",
                if current_user_id != user.id {
                    button {
                        class: "glow-hover",
                        style: format!("padding: 6px 12px; border-radius: 8px; font-weight: 600; font-size: 12px; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; {}", if is_banned { "background: rgba(52, 211, 153, 0.2); border: 1px solid rgba(52, 211, 153, 0.5); color: #6ee7b7;" } else { "background: rgba(239, 68, 68, 0.2); border: 1px solid rgba(239, 68, 68, 0.5); color: #fca5a5;" }),
                        onclick: move |e| {
                            e.stop_propagation();
                            on_action.call(String::new());
                        },
                        if is_banned {
                            Undo2 { size: 14 }
                            "{i18n.t(\"admin_action_unban\")}"
                        } else {
                            Gavel { size: 14 }
                            "{i18n.t(\"admin_action_ban\")}"
                        }
                    }
                }
            }
        }
    }
}

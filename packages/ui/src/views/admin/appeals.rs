use crate::app::{AuthState, Route};
use api::{get_moderation_appeals, resolve_moderation_appeal};
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Check, ShieldAlert, X};

#[component]
pub fn AdminAppeals() -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = use_navigator();
    let mut toaster = crate::toast::use_toaster();

    let is_allowed = match auth_state() {
        AuthState::Authenticated(u) => {
            u.role == "admin" || u.role == "super_admin" || u.role == "moderator"
        }
        AuthState::Loading => return rsx! { crate::LoadingScreen {} },
        AuthState::Unauthenticated => false,
    };

    if !is_allowed {
        nav.push(Route::Home {});
        return rsx! { div {} };
    }

    let mut status_filter = use_signal(|| "pending".to_string());

    #[allow(unused_mut)]
    let mut appeals_res = use_server_future(move || {
        let f = status_filter();
        async move {
            let status = if f == "all" { None } else { Some(f) };
            get_moderation_appeals(status, 50, 0).await
        }
    })?;

    let resolve_action = move |appeal_id: String, status: String| {
        spawn(async move {
            if let Ok(_) = resolve_moderation_appeal(appeal_id, status.clone(), None).await {
                appeals_res.restart();
                toaster.success("Appeal resolved");
            } else {
                toaster.error("Failed to resolve appeal");
            }
        });
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 120px 0 80px;",

            div {
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 32px;",
                Link {
                    to: Route::AdminDashboard {},
                    style: "color: var(--text-secondary); display: flex; align-items: center; gap: 8px; text-decoration: none;",
                    ArrowLeft { size: 20 }
                    "Back to Dashboard"
                }
            }

            div {
                style: "display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 32px;",
                div {
                    h1 {
                        style: "font-size: 32px; font-weight: 900; margin: 0 0 8px 0; display: flex; align-items: center; gap: 12px;",
                        ShieldAlert { size: 32, color: "#8b5cf6" }
                        "Moderation Appeals"
                    }
                    p { style: "color: var(--text-secondary); margin: 0;", "Review and resolve user appeals for moderation actions." }
                }

                div {
                    style: "display: flex; gap: 8px;",
                    button {
                        class: if status_filter() == "pending" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("pending".to_string()),
                        "Pending"
                    }
                    button {
                        class: if status_filter() == "approved" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("approved".to_string()),
                        "Approved"
                    }
                    button {
                        class: if status_filter() == "rejected" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("rejected".to_string()),
                        "Rejected"
                    }
                    button {
                        class: if status_filter() == "all" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("all".to_string()),
                        "All"
                    }
                }
            }

            div {
                class: "glass",
                style: "border-radius: 24px; padding: 24px; min-height: 400px;",

                match appeals_res() {
                    Some(Ok(appeals)) => {
                        if appeals.is_empty() {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 300px; color: var(--text-muted); gap: 16px;",
                                    ShieldAlert { size: 48, color: "var(--text-muted)" }
                                    p { "No appeals found." }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; gap: 16px;",
                                    for appeal in appeals {
                                        div {
                                            key: "{appeal.id}",
                                            style: "display: flex; gap: 24px; padding: 20px; border-radius: 16px; background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.05);",

                                            div {
                                                style: "flex: 1; display: flex; flex-direction: column; justify-content: center;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; margin-bottom: 8px;",
                                                    span {
                                                        style: "font-weight: 700; font-size: 16px;",
                                                        "Target: {appeal.target_type} - {appeal.target_id}"
                                                    }
                                                    {
                                                        let date_str = appeal.created_at.format("%Y-%m-%d %H:%M").to_string();
                                                        rsx! {
                                                            span {
                                                                style: "font-size: 13px; color: var(--text-muted);",
                                                                "{date_str}"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    style: "color: var(--text-secondary); font-size: 14px; margin-bottom: 12px; background: rgba(255,255,255,0.03); padding: 12px; border-radius: 8px; border-left: 2px solid #8b5cf6;",
                                                    "{appeal.reason}"
                                                }
                                                div {
                                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                                    div {
                                                        span {
                                                            style: "font-size: 12px; padding: 4px 8px; border-radius: 4px; background: rgba(255,255,255,0.05); color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600;",
                                                            "{appeal.status}"
                                                        }
                                                    }

                                                    if appeal.status == "pending" {
                                                        div {
                                                            style: "display: flex; gap: 8px;",
                                                            button {
                                                                style: "padding: 8px 16px; border-radius: 8px; border: none; background: rgba(16, 185, 129, 0.2); color: #10b981; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px; transition: background 0.2s;",
                                                                class: "glow-hover",
                                                                onclick: {
                                                                    let id = appeal.id.clone();
                                                                    move |_| resolve_action(id.clone(), "approved".to_string())
                                                                },
                                                                Check { size: 16 }
                                                                "Approve"
                                                            }
                                                            button {
                                                                style: "padding: 8px 16px; border-radius: 8px; border: none; background: rgba(239, 68, 68, 0.2); color: #f87171; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px; transition: background 0.2s;",
                                                                class: "glow-hover",
                                                                onclick: {
                                                                    let id = appeal.id.clone();
                                                                    move |_| resolve_action(id.clone(), "rejected".to_string())
                                                                },
                                                                X { size: 16 }
                                                                "Reject"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => {
                        rsx! {
                            div {
                                style: "color: #ef4444; padding: 24px; background: rgba(239, 68, 68, 0.1); border-radius: 12px;",
                                "Error loading appeals: {e}"
                            }
                        }
                    },
                    None => {
                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 16px;",
                                for _ in 0..3 {
                                    div {
                                        style: "height: 120px; background: rgba(255,255,255,0.02); border-radius: 16px; animation: pulse 2s infinite;"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use crate::app::{AuthState, Route};
use api::{get_reported_wallpapers, resolve_report};
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Check, ShieldAlert, Trash2};

#[component]
pub fn AdminReports() -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = use_navigator();
    let mut toaster = crate::toast::use_toaster();
    let i18n = crate::i18n::use_i18n();

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
    let mut reports_res = use_resource(move || {
        let f = status_filter();
        async move {
            let status = if f == "all" { None } else { Some(f) };
            get_reported_wallpapers(status).await
        }
    });

    let resolve_action = move |report_id: String, action: String| {
        spawn(async move {
            if let Ok(_) = resolve_report(report_id, action).await {
                reports_res.restart();
                toaster.success(i18n.t("success_report_resolved"));
            } else {
                toaster.error(i18n.t("err_resolve_report"));
            }
        });
    };

    rsx! {
        div {
            class: "container fade-in",
            style: "padding: 120px 0 80px;",

            div {
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 32px;",
                Link {
                    to: Route::Admin {},
                    style: "color: var(--text-secondary); display: flex; align-items: center; gap: 8px; text-decoration: none;",
                    ArrowLeft { size: 20 }
                    "{i18n.t(\"admin_back\")}"
                }
            }

            div {
                style: "display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 32px;",
                div {
                    h1 {
                        style: "font-size: 32px; font-weight: 900; margin: 0 0 8px 0; display: flex; align-items: center; gap: 12px;",
                        ShieldAlert { size: 32, color: "#f59e0b" }
                        "{i18n.t(\"admin_moderation_reports\")}"
                    }
                    p { style: "color: var(--text-secondary); margin: 0;", "{i18n.t(\"admin_reports_desc\")}" }
                }

                div {
                    style: "display: flex; gap: 8px;",
                    button {
                        class: if status_filter() == "pending" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("pending".to_string()),
                        "{i18n.t(\"admin_pending\")}"
                    }
                    button {
                        class: if status_filter() == "dismissed" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("dismissed".to_string()),
                        "{i18n.t(\"admin_dismissed\")}"
                    }
                    button {
                        class: if status_filter() == "all" { "tag active" } else { "tag" },
                        style: "padding: 8px 16px; border-radius: 8px; font-weight: 600;",
                        onclick: move |_| status_filter.set("all".to_string()),
                        "{i18n.t(\"admin_all\")}"
                    }
                }
            }

            div {
                class: "glass",
                style: "border-radius: 24px; padding: 24px; min-height: 400px;",

                match reports_res() {
                    Some(Ok(reports)) => {
                        if reports.is_empty() {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 300px; color: var(--text-muted); gap: 16px;",
                                    ShieldAlert { size: 48, color: "var(--text-muted)" }
                                    p { "{i18n.t(\"admin_no_reports\")}" }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; gap: 16px;",
                                    for report in reports {
                                        div {
                                            key: "{report.id}",
                                            style: "display: flex; gap: 24px; padding: 20px; border-radius: 16px; background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.05);",

                                            if let Some(thumb) = report.wallpaper_thumbnail.clone() {
                                                Link {
                                                    to: Route::WallpaperDetail { id: report.wallpaper_id.clone() },
                                                    div {
                                                        style: format!("width: 120px; height: 80px; border-radius: 8px; background-image: url('{}'); background-size: cover; background-position: center; border: 1px solid rgba(255,255,255,0.1);", thumb)
                                                    }
                                                }
                                            } else {
                                                div {
                                                    style: "width: 120px; height: 80px; border-radius: 8px; background: rgba(255,255,255,0.05); display: flex; align-items: center; justify-content: center; color: var(--text-muted); font-size: 12px;",
                                                    "{i18n.t(\"admin_deleted\")}"
                                                }
                                            }

                                            div {
                                                style: "flex: 1; display: flex; flex-direction: column; justify-content: center;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; margin-bottom: 8px;",
                                                    span {
                                                        style: "font-weight: 700; font-size: 16px;",
                                                        "{i18n.t(\"admin_reported_by\")}"
                                                        Link {
                                                            to: Route::PublicProfile { username: report.reporter_name.clone() },
                                                            style: "color: var(--accent-primary); text-decoration: none;",
                                                            "{report.reporter_name}"
                                                        }
                                                    }
                                                    {
                                                        let date_str = report.created_at.format("%Y-%m-%d %H:%M").to_string();
                                                        rsx! {
                                                            span {
                                                                style: "font-size: 13px; color: var(--text-muted);",
                                                                "{date_str}"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    style: "color: var(--text-secondary); font-size: 14px; margin-bottom: 12px; background: rgba(255,255,255,0.03); padding: 12px; border-radius: 8px; border-left: 2px solid #f59e0b;",
                                                    "{report.reason}"
                                                }
                                                div {
                                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                                    div {
                                                        span {
                                                            style: "font-size: 12px; padding: 4px 8px; border-radius: 4px; background: rgba(255,255,255,0.05); color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600;",
                                                            "{report.status}"
                                                        }
                                                    }

                                                    if report.status == "pending" {
                                                        div {
                                                            style: "display: flex; gap: 8px;",
                                                            button {
                                                                style: "padding: 8px 16px; border-radius: 8px; border: none; background: rgba(255,255,255,0.05); color: white; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px; transition: background 0.2s;",
                                                                class: "glow-hover",
                                                                onclick: {
                                                                    let id = report.id.clone();
                                                                    move |_| resolve_action(id.clone(), "dismiss".to_string())
                                                                },
                                                                Check { size: 16 }
                                                                "{i18n.t(\"admin_dismiss\")}"
                                                            }
                                                            button {
                                                                style: "padding: 8px 16px; border-radius: 8px; border: none; background: rgba(239, 68, 68, 0.2); color: #f87171; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px; transition: background 0.2s;",
                                                                class: "glow-hover",
                                                                onclick: {
                                                                    let id = report.id.clone();
                                                                    move |_| resolve_action(id.clone(), "delete_wallpaper".to_string())
                                                                },
                                                                Trash2 { size: 16 }
                                                                "{i18n.t(\"admin_delete\")}"
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
                                "{i18n.t(\"err_loading_reports\")}{e}"
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

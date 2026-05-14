use crate::app::Route;
use api::{get_admin_stats, get_audit_logs};
use dioxus::prelude::*;
use lucide_dioxus::{
    Activity, ArrowRight, ClipboardList, Download, Heart, Image as ImageIcon, ShieldAlert, Users,
};

#[component]
pub fn AdminDashboard() -> Element {
    let i18n = crate::i18n::use_i18n();

    #[allow(unused_mut)]
    let mut stats_res = use_server_future(move || async move { get_admin_stats().await })?;

    #[allow(unused_mut)]
    let mut audit_res = use_server_future(move || async move { get_audit_logs(10).await })?;

    use_effect(move || {
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            loop {
                gloo_timers::future::TimeoutFuture::new(30_000).await;
                stats_res.restart();
                audit_res.restart();
            }
        });
    });

    rsx! {
        div {
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 40px;",
                h1 {
                    style: "font-size: 36px; font-weight: 900; margin: 0; display: flex; align-items: center; gap: 16px;",
                    Activity { size: 36, color: "var(--accent-primary)" }
                    "{i18n.t(\"admin_dashboard_title\")}"
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

            match stats_res() {
                Some(Ok(stats)) => {
                    rsx! {
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 24px; margin-bottom: 48px;",
                            StatCard { title: "{i18n.t(\"admin_stat_users\")}", value: stats.total_users.to_string(), icon: rsx!{ Users { size: 24, color: "#60a5fa" } }, color: "#60a5fa" }
                            StatCard { title: "{i18n.t(\"admin_stat_wallpapers\")}", value: stats.total_wallpapers.to_string(), icon: rsx!{ ImageIcon { size: 24, color: "#a78bfa" } }, color: "#a78bfa" }
                            StatCard { title: "{i18n.t(\"admin_stat_downloads\")}", value: stats.total_downloads.to_string(), icon: rsx!{ Download { size: 24, color: "#34d399" } }, color: "#34d399" }
                            StatCard { title: "{i18n.t(\"admin_stat_likes\")}", value: stats.total_likes.to_string(), icon: rsx!{ Heart { size: 24, color: "#f43f5e" } }, color: "#f43f5e" }
                        }
                    }
                },
                Some(Err(e)) => rsx! { div { class: "glass", style: "padding: 24px; color: #ef4444;", "Error loading stats: {e}" } },
                None => rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 24px; margin-bottom: 48px;",
                        for _ in 0..4 {
                            div {
                                class: "glass",
                                style: "padding: 24px; border-radius: 24px; height: 138px; display: flex; flex-direction: column; position: relative; overflow: hidden; opacity: 0.7;",
                                div {
                                    style: "display: flex; align-items: center; gap: 16px; margin-bottom: 16px;",
                                    div { style: "width: 48px; height: 48px; border-radius: 16px; background: rgba(255,255,255,0.05);" }
                                    div { style: "width: 100px; height: 14px; border-radius: 8px; background: rgba(255,255,255,0.05);" }
                                }
                                div { style: "width: 60px; height: 36px; border-radius: 12px; background: rgba(255,255,255,0.05);" }
                            }
                        }
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
                        ClipboardList { size: 24, color: "var(--accent-primary)" }
                        "{i18n.t(\"admin_recent_audit_logs\")}"
                    }
                    match audit_res() {
                        Some(Ok(logs)) => {
                            if logs.is_empty() {
                                rsx! { p { style: "color: var(--text-muted);", "{i18n.t(\"admin_no_audit_logs\")}" } }
                            } else {
                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 12px;",
                                        for log in logs {
                                            div {
                                                key: "{log.id}",
                                                style: "padding: 16px; background: rgba(0,0,0,0.2); border-radius: 12px; border: 1px solid rgba(255,255,255,0.05); display: flex; flex-direction: column; gap: 8px;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                                    span {
                                                        style: "font-weight: 700; color: white;",
                                                        "{log.admin_name} "
                                                        span { style: "color: #ef4444;", "{log.action} " }
                                                        span { style: "color: var(--text-secondary);", "{log.target_type}" }
                                                    }
                                                    {
                                                        let date_str = log.created_at.format("%Y-%m-%d %H:%M").to_string();
                                                        rsx! { span { style: "font-size: 12px; color: var(--text-muted);", "{date_str}" } }
                                                    }
                                                }
                                                div {
                                                    style: "font-size: 13px; color: var(--text-muted); font-family: monospace; background: rgba(255,255,255,0.05); padding: 4px 8px; border-radius: 4px; width: fit-content;",
                                                    "Target ID: {log.target_id}"
                                                }
                                                if let Some(reason) = log.reason {
                                                    div {
                                                        style: "font-size: 14px; color: var(--text-secondary); margin-top: 4px; border-left: 2px solid var(--accent-primary); padding-left: 8px;",
                                                        "Reason: \"{reason}\""
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { p { style: "color: #ef4444;", "Error loading logs: {e}" } },
                        None => rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 12px;",
                                for _ in 0..4 {
                                    div {
                                        style: "padding: 16px; background: rgba(0,0,0,0.1); border-radius: 12px; border: 1px solid rgba(255,255,255,0.02); height: 90px; display: flex; flex-direction: column; justify-content: space-between; opacity: 0.7;",
                                        div {
                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                            div { style: "width: 180px; height: 16px; border-radius: 8px; background: rgba(255,255,255,0.05);" }
                                            div { style: "width: 100px; height: 12px; border-radius: 6px; background: rgba(255,255,255,0.03);" }
                                        }
                                        div { style: "width: 220px; height: 24px; border-radius: 6px; background: rgba(255,255,255,0.05);" }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "glass",
                    style: "padding: 32px; border-radius: 24px; height: fit-content;",
                    h2 {
                        style: "margin-bottom: 24px; display: flex; align-items: center; gap: 12px; font-size: 20px;",
                        Users { size: 24, color: "#60a5fa" }
                        "{i18n.t(\"admin_user_mgmt\")}"
                    }
                    p { style: "color: var(--text-secondary); line-height: 1.6; margin-bottom: 24px;", "{i18n.t(\"admin_user_mgmt_desc\")}" }
                    div {
                        style: "display: flex; gap: 12px; margin-top: 24px;",
                        Link {
                            to: Route::AdminUsers {},
                            class: "glow-hover",
                            style: "padding: 12px 24px; border-radius: 12px; background: rgba(96, 165, 250, 0.2); border: 1px solid rgba(96, 165, 250, 0.5); color: #93c5fd; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px;",
                            "{i18n.t(\"admin_open_user_mgmt\")}"
                            ArrowRight { size: 18 }
                        }
                        Link {
                            to: Route::AdminReports {},
                            class: "glow-hover",
                            style: "padding: 12px 24px; border-radius: 12px; background: rgba(245, 158, 11, 0.2); border: 1px solid rgba(245, 158, 11, 0.5); color: #fcd34d; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px;",
                            "{i18n.t(\"admin_review_reports\")}"
                            ArrowRight { size: 18 }
                        }
                        Link {
                            to: Route::AdminDmca {},
                            class: "glow-hover",
                            style: "padding: 12px 24px; border-radius: 12px; background: rgba(239, 68, 68, 0.2); border: 1px solid rgba(239, 68, 68, 0.5); color: #f87171; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px;",
                            "DMCA Claims"
                            ArrowRight { size: 18 }
                        }
                    }
                }
            }

            div {
                style: "display: grid; grid-template-columns: 1fr; gap: 24px;",
                div {
                    class: "glass",
                    style: "padding: 32px; border-radius: 24px;",
                    h2 {
                        style: "margin-bottom: 24px; display: flex; align-items: center; gap: 12px; font-size: 20px;",
                        ShieldAlert { size: 24, color: "var(--accent-primary)" }
                        "{i18n.t(\"admin_quick_moderation\")}"
                    }
                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",
                        p { style: "color: var(--text-secondary); line-height: 1.6;", "To moderate a specific wallpaper, browse the platform normally. As an admin, you have a direct deletion button on every wallpaper's detail page." }

                        div {
                            style: "margin-top: 16px;",
                            Link {
                                to: Route::Latest {},
                                class: "glow-hover",
                                style: "padding: 12px 24px; border-radius: 12px; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); color: white; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px;",
                                "{i18n.t(\"admin_review_latest\")}"
                                ArrowRight { size: 18 }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatCard(title: String, value: String, icon: Element, color: String) -> Element {
    rsx! {
        div {
            class: "glass glow-hover",
            style: "padding: 24px; border-radius: 24px; border-top: 1px solid rgba(255,255,255,0.1); display: flex; flex-direction: column; position: relative; overflow: hidden;",
            div {
                style: format!("position: absolute; top: -20px; right: -20px; width: 100px; height: 100px; background: {}; filter: blur(40px); opacity: 0.15; border-radius: 50%;", color)
            }
            div {
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 16px;",
                div {
                    style: format!("width: 48px; height: 48px; border-radius: 16px; background: {}15; display: flex; align-items: center; justify-content: center; border: 1px solid {}30;", color, color),
                    {icon}
                }
                span { style: "font-size: 14px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em;", "{title}" }
            }
            div {
                style: "font-size: 36px; font-weight: 900; font-variant-numeric: tabular-nums;",
                "{value}"
            }
        }
    }
}
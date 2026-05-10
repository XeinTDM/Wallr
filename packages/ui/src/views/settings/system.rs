#![cfg(feature = "desktop")]

use dioxus::prelude::*;
use lucide_dioxus::{Keyboard, Monitor};

fn get_auto_launch() -> Option<auto_launch::AutoLaunch> {
    let app_path = std::env::current_exe().ok()?;
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Wallr")
        .set_app_path(app_path.to_str()?)
        .set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent)
        .build()
        .ok()
    }

#[component]
pub fn SystemSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut is_enabled = use_signal(|| {
        get_auto_launch().map(|al| al.is_enabled().unwrap_or(false)).unwrap_or(false)
    });

    rsx! {
        div {
            class: "settings-card fade-in",
            h2 { Monitor { size: 20 } "{i18n.t(\"sys_integration\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_launch_startup\")}" }
                    p { "{i18n.t(\"sys_launch_desc\")}" }
                }
                div { class: "setting-control",
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: is_enabled(),
                            onchange: move |_| {
                                let current = is_enabled();
                                is_enabled.set(!current);
                                if let Some(al) = get_auto_launch() {
                                    if !current {
                                        let _ = al.enable();
                                    } else {
                                        let _ = al.disable();
                                    }
                                }
                            }
                        }
                        span { class: "toggle-slider" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn KeybindsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-card fade-in",
            h2 { Keyboard { size: 20 } "{i18n.t(\"sys_global_keybinds\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_random\")}" }
                    p { "{i18n.t(\"sys_keybind_random_desc\")}" }
                }
                div { class: "setting-control",
                    div {
                        style: "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border-radius: 8px; font-family: monospace; font-weight: 600;",
                        "Ctrl + Alt + W"
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_next\")}" }
                    p { "{i18n.t(\"sys_keybind_next_desc\")}" }
                }
                div { class: "setting-control",
                    div {
                        style: "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border-radius: 8px; font-family: monospace; font-weight: 600;",
                        "Ctrl + Alt + Right"
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_prev\")}" }
                    p { "{i18n.t(\"sys_keybind_prev_desc\")}" }
                }
                div { class: "setting-control",
                    div {
                        style: "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border-radius: 8px; font-family: monospace; font-weight: 600;",
                        "Ctrl + Alt + Left"
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_save\")}" }
                    p { "{i18n.t(\"sys_keybind_save_desc\")}" }
                }
                div { class: "setting-control",
                    div {
                        style: "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border-radius: 8px; font-family: monospace; font-weight: 600;",
                        "Ctrl + Alt + S"
                    }
                }
            }

            div { class: "setting-group", style: "border-bottom: none; padding-bottom: 0;",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_hud\")}" }
                    p { "{i18n.t(\"sys_keybind_hud_desc\")}" }
                }
                div { class: "setting-control",
                    div {
                        style: "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border-radius: 8px; font-family: monospace; font-weight: 600;",
                        "Ctrl + Shift + H"
                    }
                }
            }
        }
    }
}

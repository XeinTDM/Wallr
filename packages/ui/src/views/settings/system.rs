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
            class: "settings-card",
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub hotkey_next_wp: (Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code),
    pub hotkey_prev_wp: (Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code),
    pub hotkey_save_wp: (Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code),
    pub hotkey_toggle_ui: (Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code),
    pub wallpaper_mode: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        use global_hotkey::hotkey::{Code, Modifiers};
        Self {
            hotkey_next_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowRight),
            hotkey_prev_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft),
            hotkey_save_wp: (Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS),
            hotkey_toggle_ui: (Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH),
            wallpaper_mode: "Crop".to_string(),
        }
    }
}

impl AppConfig {
    fn get_path() -> std::path::PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let root_dir = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf();
            
        root_dir.join(".desktop_data").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&data) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn format_hotkey(hotkey: &(Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code)) -> String {
    let mut parts = Vec::new();
    if let Some(mods) = hotkey.0 {
        if mods.contains(global_hotkey::hotkey::Modifiers::CONTROL) { parts.push("Ctrl"); }
        if mods.contains(global_hotkey::hotkey::Modifiers::ALT) { parts.push("Alt"); }
        if mods.contains(global_hotkey::hotkey::Modifiers::SHIFT) { parts.push("Shift"); }
        if mods.contains(global_hotkey::hotkey::Modifiers::SUPER) { parts.push("Super"); }
    }
    
    let code_str = match hotkey.1 {
        global_hotkey::hotkey::Code::KeyW => "W",
        global_hotkey::hotkey::Code::KeyS => "S",
        global_hotkey::hotkey::Code::KeyH => "H",
        global_hotkey::hotkey::Code::ArrowRight => "Right",
        global_hotkey::hotkey::Code::ArrowLeft => "Left",
        _ => "Unknown", // Simplification for display purposes
    };
    parts.push(code_str);
    
    parts.join(" + ")
}

fn parse_keyboard_event(evt: &KeyboardEvent) -> Option<(Option<global_hotkey::hotkey::Modifiers>, global_hotkey::hotkey::Code)> {
    use global_hotkey::hotkey::{Code, Modifiers};
    
    let mut mods = Modifiers::empty();
    if evt.modifiers().contains(dioxus::html::input_data::keyboard_types::Modifiers::CONTROL) { mods.insert(Modifiers::CONTROL); }
    if evt.modifiers().contains(dioxus::html::input_data::keyboard_types::Modifiers::ALT) { mods.insert(Modifiers::ALT); }
    if evt.modifiers().contains(dioxus::html::input_data::keyboard_types::Modifiers::SHIFT) { mods.insert(Modifiers::SHIFT); }
    if evt.modifiers().contains(dioxus::html::input_data::keyboard_types::Modifiers::META) { mods.insert(Modifiers::SUPER); }

    let code = match evt.code() {
        dioxus::html::input_data::keyboard_types::Code::KeyA => Code::KeyA,
        dioxus::html::input_data::keyboard_types::Code::KeyB => Code::KeyB,
        dioxus::html::input_data::keyboard_types::Code::KeyC => Code::KeyC,
        dioxus::html::input_data::keyboard_types::Code::KeyD => Code::KeyD,
        dioxus::html::input_data::keyboard_types::Code::KeyE => Code::KeyE,
        dioxus::html::input_data::keyboard_types::Code::KeyF => Code::KeyF,
        dioxus::html::input_data::keyboard_types::Code::KeyG => Code::KeyG,
        dioxus::html::input_data::keyboard_types::Code::KeyH => Code::KeyH,
        dioxus::html::input_data::keyboard_types::Code::KeyI => Code::KeyI,
        dioxus::html::input_data::keyboard_types::Code::KeyJ => Code::KeyJ,
        dioxus::html::input_data::keyboard_types::Code::KeyK => Code::KeyK,
        dioxus::html::input_data::keyboard_types::Code::KeyL => Code::KeyL,
        dioxus::html::input_data::keyboard_types::Code::KeyM => Code::KeyM,
        dioxus::html::input_data::keyboard_types::Code::KeyN => Code::KeyN,
        dioxus::html::input_data::keyboard_types::Code::KeyO => Code::KeyO,
        dioxus::html::input_data::keyboard_types::Code::KeyP => Code::KeyP,
        dioxus::html::input_data::keyboard_types::Code::KeyQ => Code::KeyQ,
        dioxus::html::input_data::keyboard_types::Code::KeyR => Code::KeyR,
        dioxus::html::input_data::keyboard_types::Code::KeyS => Code::KeyS,
        dioxus::html::input_data::keyboard_types::Code::KeyT => Code::KeyT,
        dioxus::html::input_data::keyboard_types::Code::KeyU => Code::KeyU,
        dioxus::html::input_data::keyboard_types::Code::KeyV => Code::KeyV,
        dioxus::html::input_data::keyboard_types::Code::KeyW => Code::KeyW,
        dioxus::html::input_data::keyboard_types::Code::KeyX => Code::KeyX,
        dioxus::html::input_data::keyboard_types::Code::KeyY => Code::KeyY,
        dioxus::html::input_data::keyboard_types::Code::KeyZ => Code::KeyZ,
        dioxus::html::input_data::keyboard_types::Code::ArrowRight => Code::ArrowRight,
        dioxus::html::input_data::keyboard_types::Code::ArrowLeft => Code::ArrowLeft,
        dioxus::html::input_data::keyboard_types::Code::ArrowUp => Code::ArrowUp,
        dioxus::html::input_data::keyboard_types::Code::ArrowDown => Code::ArrowDown,
        _ => return None,
    };

    let mods_opt = if mods.is_empty() { None } else { Some(mods) };
    Some((mods_opt, code))
}

#[component]
pub fn KeybindsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut config = use_signal(|| AppConfig::load());
    let mut is_recording = use_signal(|| None::<String>);

    let mut handle_keydown = move |evt: KeyboardEvent, key: String| {
        evt.prevent_default();
        if let Some(hotkey) = parse_keyboard_event(&evt) {
            let mut current_config = config();
            match key.as_str() {
                "next" => current_config.hotkey_next_wp = hotkey,
                "prev" => current_config.hotkey_prev_wp = hotkey,
                "save" => current_config.hotkey_save_wp = hotkey,
                "hud" => current_config.hotkey_toggle_ui = hotkey,
                _ => {}
            }
            let _ = current_config.save();
            config.set(current_config);
            is_recording.set(None);
        } else if evt.key() == dioxus::html::input_data::keyboard_types::Key::Escape {
            is_recording.set(None);
        }
    };

    rsx! {
        div {
            class: "settings-card",
            h2 { Keyboard { size: 20 } "{i18n.t(\"sys_global_keybinds\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_next\")}" }
                    p { "{i18n.t(\"sys_keybind_next_desc\")}" }
                }
                div { class: "setting-control",
                    button {
                        style: if is_recording() == Some("next".to_string()) {
                            "padding: 8px 16px; background: rgba(139, 92, 246, 0.3); border: 1px solid #8b5cf6; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        } else {
                            "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border: 1px solid transparent; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        },
                        onclick: move |_| is_recording.set(Some("next".to_string())),
                        onkeydown: move |e| {
                            if is_recording() == Some("next".to_string()) {
                                handle_keydown(e, "next".to_string());
                            }
                        },
                        tabindex: 0,
                        if is_recording() == Some("next".to_string()) {
                            "Recording..."
                        } else {
                            "{format_hotkey(&config().hotkey_next_wp)}"
                        }
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_prev\")}" }
                    p { "{i18n.t(\"sys_keybind_prev_desc\")}" }
                }
                div { class: "setting-control",
                    button {
                        style: if is_recording() == Some("prev".to_string()) {
                            "padding: 8px 16px; background: rgba(139, 92, 246, 0.3); border: 1px solid #8b5cf6; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        } else {
                            "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border: 1px solid transparent; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        },
                        onclick: move |_| is_recording.set(Some("prev".to_string())),
                        onkeydown: move |e| {
                            if is_recording() == Some("prev".to_string()) {
                                handle_keydown(e, "prev".to_string());
                            }
                        },
                        tabindex: 0,
                        if is_recording() == Some("prev".to_string()) {
                            "Recording..."
                        } else {
                            "{format_hotkey(&config().hotkey_prev_wp)}"
                        }
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_save\")}" }
                    p { "{i18n.t(\"sys_keybind_save_desc\")}" }
                }
                div { class: "setting-control",
                    button {
                        style: if is_recording() == Some("save".to_string()) {
                            "padding: 8px 16px; background: rgba(139, 92, 246, 0.3); border: 1px solid #8b5cf6; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        } else {
                            "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border: 1px solid transparent; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        },
                        onclick: move |_| is_recording.set(Some("save".to_string())),
                        onkeydown: move |e| {
                            if is_recording() == Some("save".to_string()) {
                                handle_keydown(e, "save".to_string());
                            }
                        },
                        tabindex: 0,
                        if is_recording() == Some("save".to_string()) {
                            "Recording..."
                        } else {
                            "{format_hotkey(&config().hotkey_save_wp)}"
                        }
                    }
                }
            }

            div { class: "setting-group", style: "border-bottom: none; padding-bottom: 0;",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"sys_keybind_hud\")}" }
                    p { "{i18n.t(\"sys_keybind_hud_desc\")}" }
                }
                div { class: "setting-control",
                    button {
                        style: if is_recording() == Some("hud".to_string()) {
                            "padding: 8px 16px; background: rgba(139, 92, 246, 0.3); border: 1px solid #8b5cf6; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        } else {
                            "padding: 8px 16px; background: rgba(255, 255, 255, 0.1); border: 1px solid transparent; border-radius: 8px; font-family: monospace; font-weight: 600; cursor: pointer; min-width: 120px;"
                        },
                        onclick: move |_| is_recording.set(Some("hud".to_string())),
                        onkeydown: move |e| {
                            if is_recording() == Some("hud".to_string()) {
                                handle_keydown(e, "hud".to_string());
                            }
                        },
                        tabindex: 0,
                        if is_recording() == Some("hud".to_string()) {
                            "Recording..."
                        } else {
                            "{format_hotkey(&config().hotkey_toggle_ui)}"
                        }
                    }
                }
            }
            
            p {
                style: "color: var(--text-muted); font-size: 13px; margin-top: 16px;",
                "Restart the application for keybind changes to take full effect."
            }
        }
    }
}

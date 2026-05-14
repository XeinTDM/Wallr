pub mod account;
pub mod appearance;
pub mod downloads;
pub mod notifications;
#[cfg(feature = "desktop")]
pub mod system;

use crate::app::Route;
use crate::LoadingScreen;
use dioxus::prelude::*;
use lucide_dioxus::{Bell, CloudDownload, Palette, User};
#[cfg(feature = "desktop")]
use lucide_dioxus::{Keyboard, Monitor};

const SETTINGS_CSS: Asset = asset!("/assets/styling/settings.css");

#[allow(unused_variables)]
pub(crate) fn use_stored_signal<T: std::str::FromStr + std::fmt::Display + Clone + 'static>(
    key: &'static str,
    default: T,
) -> Signal<T> {
    let mut sig = use_signal(move || {
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(val)) = storage.get_item(key) {
                    if let Ok(parsed) = val.parse::<T>() {
                        return parsed;
                    }
                }
            }
        }
        default.clone()
    });

    #[cfg(not(target_arch = "wasm32"))]
    {
        use_effect(move || {
            let mut eval = dioxus::document::eval(&format!(
                "let val = localStorage.getItem('{}'); return val === null ? '' : val;",
                key
            ));
            spawn(async move {
                if let Ok(val) = eval.recv::<String>().await
                    && !val.is_empty()
                        && let Ok(parsed) = val.parse::<T>() {
                            sig.set(parsed);
                        }
            });
        });
    }

    use_effect(move || {
        let val = sig();
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(key, &val.to_string());
                if let Ok(event) = web_sys::CustomEvent::new("local-storage-update") {
                    let _ = win.dispatch_event(&event);
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let script = format!(
                "localStorage.setItem('{}', '{}'); window.dispatchEvent(new Event('local-storage-update'));",
                key,
                val
            );
            let _ = dioxus::document::eval(&script);
        }
    });

    sig
}

#[component]
pub fn SettingsLayout() -> Element {
    let i18n = crate::i18n::use_i18n();
    let route = use_route::<Route>();

    rsx! {
        document::Stylesheet { href: SETTINGS_CSS }
        div {
            class: "settings-page",
            style: "padding: 100px 32px 80px; max-width: 1200px; margin: 0 auto; display: flex; gap: 40px; min-height: calc(100vh - 80px);",

            // Sidebar
            div {
                class: "settings-sidebar",
                style: "width: 280px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px;",
                div {
                    class: "section-title",
                    "{i18n.t(\"settings_user_settings\")}"
                }
                Link {
                    to: Route::SettingsAccount {},
                    class: "settings-nav-item",
                    active_class: "active",
                    User { size: 18 }
                    "{i18n.t(\"settings_account\")}"
                }
                Link {
                    to: Route::SettingsAppearance {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Palette { size: 18 }
                    "{i18n.t(\"settings_appearance\")}"
                }
                Link {
                    to: Route::SettingsDownloads {},
                    class: "settings-nav-item",
                    active_class: "active",
                    CloudDownload { size: 18 }
                    "{i18n.t(\"settings_downloads\")}"
                }
                Link {
                    to: Route::SettingsNotifications {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Bell { size: 18 }
                    "{i18n.t(\"settings_notifications\")}"
                }

                {
                    #[cfg(feature = "desktop")]
                    {
                        rsx! {
                            div {
                                class: "section-title",
                                style: "margin-top: 24px;",
                                "{i18n.t(\"settings_desktop_settings\")}"
                            }
                            Link {
                                to: Route::SettingsSystem {},
                                class: "settings-nav-item",
                                active_class: "active",
                                Monitor { size: 18 }
                                "{i18n.t(\"settings_system\")}"
                            }
                            Link {
                                to: Route::SettingsKeybinds {},
                                class: "settings-nav-item",
                                active_class: "active",
                                Keyboard { size: 18 }
                                "{i18n.t(\"settings_keybinds\")}"
                            }
                        }
                    }
                    #[cfg(not(feature = "desktop"))]
                    {
                        rsx! {}
                    }
                }
            }

            // Main Content Area
            div {
                class: "settings-content",
                style: "flex: 1; max-width: 800px;",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
pub fn SettingsAccount() -> Element {
    let nav = use_navigator();
    let user = use_context::<Signal<crate::app::AuthState>>();
    let i18n = crate::i18n::use_i18n();

    let (real_username, real_email, real_pfp_url, real_bio, real_socials) = match user() {
        crate::app::AuthState::Loading => return rsx! { LoadingScreen {} },
        crate::app::AuthState::Unauthenticated => {
            nav.push(Route::Login {});
            return rsx! {};
        }
        crate::app::AuthState::Authenticated(u) => {
            (u.name, u.email, u.pfp_url, u.bio, u.social_links)
        }
    };

    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_account\")}" }
        }
        account::AccountSettings {
            real_username,
            real_email,
            real_pfp_url,
            real_bio,
            real_socials,
        }
    }
}

#[component]
pub fn SettingsAppearance() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_appearance\")}" }
        }
        appearance::AppearanceSettings {}
    }
}

#[component]
pub fn SettingsDownloads() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_downloads\")}" }
        }
        downloads::DownloadsSettings {}
    }
}

#[component]
pub fn SettingsNotifications() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_notifications\")}" }
        }
        notifications::NotificationsSettings {}
    }
}

#[cfg(feature = "desktop")]
#[component]
pub fn SettingsSystem() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_system\")}" }
        }
        system::SystemSettings {}
    }
}

#[cfg(feature = "desktop")]
#[component]
pub fn SettingsKeybinds() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "settings-header",
            h1 { "{i18n.t(\"settings_header\")}" }
            p { "{i18n.t(\"settings_desc_keybinds\")}" }
        }
        system::KeybindsSettings {}
    }
}
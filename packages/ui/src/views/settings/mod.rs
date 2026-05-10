pub mod account;
pub mod appearance;
pub mod downloads;
pub mod notifications;
#[cfg(feature = "desktop")]
pub mod system;

use crate::LoadingScreen;
use dioxus::prelude::*;
use lucide_dioxus::{Bell, CloudDownload, Palette, User};
#[cfg(feature = "desktop")]
use lucide_dioxus::{Keyboard, Monitor};

const SETTINGS_CSS: Asset = asset!("/assets/styling/settings.css");

#[derive(PartialEq, Clone, Copy)]
pub enum SettingsTab {
    Account,
    Appearance,
    Downloads,
    Notifications,
    #[cfg(feature = "desktop")]
    System,
    #[cfg(feature = "desktop")]
    Keybinds,
}

#[allow(unused_variables)]
pub(crate) fn use_stored_signal<T: std::str::FromStr + std::fmt::Display + Clone + 'static>(
    key: &'static str,
    default: T,
) -> Signal<T> {
    let sig = use_signal(move || {
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
    });

    sig
}

#[component]
pub fn Settings() -> Element {
    let nav = use_navigator();
    let user = use_context::<Signal<crate::app::AuthState>>();

    let (real_username, real_email, real_pfp_url, real_bio, real_socials) = match user() {
        crate::app::AuthState::Loading => return rsx! { LoadingScreen {} },
        crate::app::AuthState::Unauthenticated => {
            nav.push(crate::app::Route::Login {});
            return rsx! {};
        }
        crate::app::AuthState::Authenticated(u) => {
            (u.name, u.email, u.pfp_url, u.bio, u.social_links)
        }
    };

    let mut active_tab = use_signal(|| SettingsTab::Account);
    let i18n = crate::i18n::use_i18n();

    rsx! {
        document::Stylesheet { href: SETTINGS_CSS }
        div {
            class: "settings-page fade-in",
            style: "padding: 100px 32px 80px; max-width: 1200px; margin: 0 auto; display: flex; gap: 40px; min-height: calc(100vh - 80px);",

            // Sidebar
            div {
                class: "settings-sidebar",
                style: "width: 280px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px;",
                div {
                    class: "section-title",
                    "{i18n.t(\"settings_user_settings\")}"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Account { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Account),
                    User { size: 18 }
                    "{i18n.t(\"settings_account\")}"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Appearance { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Appearance),
                    Palette { size: 18 }
                    "{i18n.t(\"settings_appearance\")}"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Downloads { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Downloads),
                    CloudDownload { size: 18 }
                    "{i18n.t(\"settings_downloads\")}"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Notifications { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Notifications),
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
                            button {
                                class: "settings-nav-item",
                                class: if active_tab() == SettingsTab::System { "active" },
                                onclick: move |_| active_tab.set(SettingsTab::System),
                                Monitor { size: 18 }
                                "{i18n.t(\"settings_system\")}"
                            }
                            button {
                                class: "settings-nav-item",
                                class: if active_tab() == SettingsTab::Keybinds { "active" },
                                onclick: move |_| active_tab.set(SettingsTab::Keybinds),
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

                div {
                    class: "settings-header",
                    h1 { "{i18n.t(\"settings_header\")}" }
                    p {
                        match active_tab() {
                            SettingsTab::Account => i18n.t("settings_desc_account"),
                            SettingsTab::Appearance => i18n.t("settings_desc_appearance"),
                            SettingsTab::Downloads => i18n.t("settings_desc_downloads"),
                            SettingsTab::Notifications => i18n.t("settings_desc_notifications"),
                            #[cfg(feature = "desktop")]
                            SettingsTab::System => i18n.t("settings_desc_system"),
                            #[cfg(feature = "desktop")]
                            SettingsTab::Keybinds => i18n.t("settings_desc_keybinds"),
                        }
                    }
                }

                if active_tab() == SettingsTab::Account {
                    account::AccountSettings {
                        real_username,
                        real_email,
                        real_pfp_url,
                        real_bio,
                        real_socials,
                    }
                }

                if active_tab() == SettingsTab::Appearance {
                    appearance::AppearanceSettings {}
                }

                if active_tab() == SettingsTab::Downloads {
                    downloads::DownloadsSettings {}
                }

                if active_tab() == SettingsTab::Notifications {
                    notifications::NotificationsSettings {}
                }

                {
                    #[cfg(feature = "desktop")]
                    {
                        if active_tab() == SettingsTab::System {
                            rsx! { system::SystemSettings {} }
                        } else if active_tab() == SettingsTab::Keybinds {
                            rsx! { system::KeybindsSettings {} }
                        } else { rsx! {} }
                    }
                    #[cfg(not(feature = "desktop"))]
                    {
                        rsx! {}
                    }
                }
            }
        }
    }
}

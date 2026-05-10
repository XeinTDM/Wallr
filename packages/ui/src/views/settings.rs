use crate::app::Route;
use crate::{LoadingScreen, use_toaster};
use dioxus::prelude::*;
use lucide_dioxus::{Bell, Camera, CloudDownload, Eye, Palette, ShieldAlert, User};

const SETTINGS_CSS: Asset = asset!("/assets/styling/settings.css");

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    Account,
    Appearance,
    Downloads,
    Notifications,
}

#[allow(unused_variables)]
fn use_stored_signal<T: std::str::FromStr + std::fmt::Display + Clone + 'static>(
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
            }
        }
    });

    sig
}

#[component]
pub fn Settings() -> Element {
    let nav = use_navigator();
    let user = use_context::<Signal<crate::app::AuthState>>();
    let toaster = use_toaster();

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

    let mut active_tab = use_signal(|| SettingsTab::Account);

    // Account Settings
    let mut username = use_signal(move || real_username.clone());
    let mut email = use_signal(move || real_email.clone());
    let mut bio = use_signal(move || real_bio.clone().unwrap_or_default());
    let socials_1 = real_socials.clone();
    let mut x_url = use_signal(move || {
        socials_1
            .clone()
            .and_then(|s| s.get("x").cloned())
            .unwrap_or_default()
    });
    let socials_2 = real_socials.clone();
    let mut github_url = use_signal(move || {
        socials_2
            .clone()
            .and_then(|s| s.get("github").cloned())
            .unwrap_or_default()
    });
    let mut instagram_url = use_signal(move || {
        real_socials
            .clone()
            .and_then(|s| s.get("instagram").cloned())
            .unwrap_or_default()
    });

    let mut current_password = use_signal(String::new);
    let mut new_password = use_signal(String::new);

    // Appearance Settings
    let mut theme = use_stored_signal("settings_theme", "Dark Mode".to_string());
    let mut animations = use_stored_signal("settings_animations", true);

    // Download Settings
    let mut quality = use_stored_signal("settings_quality", "Original (4K+)".to_string());
    let mut auto_download_avif = use_stored_signal("settings_auto_download_avif", true);
    let mut safe_search = use_stored_signal("settings_safe_search", true);

    // Notification Settings
    let mut email_notifs = use_stored_signal("settings_email_notifs", true);
    let mut push_notifs = use_stored_signal("settings_push_notifs", false);

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
                    "User Settings"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Account { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Account),
                    User { size: 18 }
                    "Account"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Appearance { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Appearance),
                    Palette { size: 18 }
                    "Appearance"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Downloads { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Downloads),
                    CloudDownload { size: 18 }
                    "Downloads & Content"
                }
                button {
                    class: "settings-nav-item",
                    class: if active_tab() == SettingsTab::Notifications { "active" },
                    onclick: move |_| active_tab.set(SettingsTab::Notifications),
                    Bell { size: 18 }
                    "Notifications"
                }
            }

            // Main Content Area
            div {
                class: "settings-content",
                style: "flex: 1; max-width: 800px;",

                div {
                    class: "settings-header",
                    h1 { "Settings" }
                    p {
                        match active_tab() {
                            SettingsTab::Account => "Manage your personal information and security.",
                            SettingsTab::Appearance => "Customize how Wallr looks and feels.",
                            SettingsTab::Downloads => "Configure your download preferences and content filters.",
                            SettingsTab::Notifications => "Choose what updates you want to receive.",
                        }
                    }
                }

                if active_tab() == SettingsTab::Account {
                    div {
                        class: "settings-card fade-in",
                        h2 { User { size: 20 } "Profile Settings" }

                        div { class: "setting-group",
                            style: "flex-direction: column; align-items: flex-start; gap: 24px; border-bottom: none; padding-bottom: 0;",
                            label {
                                style: "display: block; width: 100%; cursor: pointer;",
                                div {
                                    style: "width: 100%; height: 160px; border-radius: 16px; background: linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(5, 5, 5, 1) 100%); position: relative; border: 1px dashed rgba(255,255,255,0.2); display: flex; align-items: center; justify-content: center; transition: all 0.2s;",
                                    class: "glow-hover",
                                    Camera { size: 32, color: "var(--text-muted)" }
                                    span { style: "position: absolute; bottom: 16px; font-size: 14px; font-weight: 600; color: var(--text-secondary);", "Change Profile Banner" }
                                }
                                input {
                                    r#type: "file",
                                    accept: "image/*",
                                    style: "display: none;",
                                    onchange: move |e| async move {
                                        let files = e.files();
                                        if !files.is_empty() {
                                            let file = &files[0];
                                            let mut toaster = toaster;
                                            toaster.success("Uploading banner...");
                                            if let Ok(bytes) = file.read_bytes().await {
                                                #[cfg(target_arch = "wasm32")]
                                                if let Ok(resp) = gloo_net::http::Request::post("/api/upload_media").header("X-Media-Type", "banner").body(bytes.to_vec()).unwrap().send().await {
                                                    if resp.ok() { toaster.success("Banner updated! Refreshing..."); web_sys::window().unwrap().location().reload().unwrap(); }
                                                    else { toaster.error("Upload failed."); }
                                                }
                                                #[cfg(not(target_arch = "wasm32"))]
                                                if let Ok(resp) = reqwest::Client::new().post("http://localhost:8080/api/upload_media").header("X-Media-Type", "banner").body(bytes).send().await {
                                                    if resp.status().is_success() { toaster.success("Banner updated! Refreshing..."); nav.push(crate::app::Route::Home {}); }
                                                    else { toaster.error("Upload failed."); }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: flex; align-items: center; gap: 24px; margin-top: -60px; margin-left: 24px; z-index: 10;",
                                label {
                                    style: "display: block; cursor: pointer;",
                                    div {
                                        style: "width: 100px; height: 100px; border-radius: 50%; border: 4px solid var(--bg-primary); background: var(--bg-secondary); overflow: hidden; position: relative; display: flex; align-items: center; justify-content: center;",
                                        class: "glow-hover",
                                        img {
                                            src: "{crate::resolve_asset_url(&real_pfp_url)}",
                                            style: "width: 100%; height: 100%; object-fit: cover; opacity: 0.5;"
                                        }
                                        div {
                                            style: "position: absolute;",
                                            Camera { size: 24, color: "white" }
                                        }
                                    }
                                    input {
                                        r#type: "file",
                                        accept: "image/*",
                                        style: "display: none;",
                                        onchange: move |e| async move {
                                            let files = e.files();
                                            if !files.is_empty() {
                                                let file = &files[0];
                                                let mut toaster = toaster;
                                                toaster.success("Uploading avatar...");
                                                if let Ok(bytes) = file.read_bytes().await {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if let Ok(resp) = gloo_net::http::Request::post("/api/upload_media").header("X-Media-Type", "pfp").body(bytes.to_vec()).unwrap().send().await {
                                                        if resp.ok() { toaster.success("Avatar updated! Refreshing..."); web_sys::window().unwrap().location().reload().unwrap(); }
                                                        else { toaster.error("Upload failed."); }
                                                    }
                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    if let Ok(resp) = reqwest::Client::new().post("http://localhost:8080/api/upload_media").header("X-Media-Type", "pfp").body(bytes).send().await {
                                                        if resp.status().is_success() { toaster.success("Avatar updated! Refreshing..."); nav.push(crate::app::Route::Home {}); }
                                                        else { toaster.error("Upload failed."); }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div {
                                    style: "margin-top: 40px;",
                                    h3 { style: "font-size: 16px; font-weight: 600; margin-bottom: 4px;", "Profile Avatar" }
                                    p { style: "font-size: 14px; color: var(--text-muted);", "Recommended: 256x256px (JPG, PNG)" }
                                }
                            }
                        }
                        form {
                            class: "setting-group", style: "flex-direction: column; align-items: flex-start; gap: 12px;",
                            onsubmit: move |e| {
                                e.prevent_default();
                                let name_val = username();
                                let email_val = email();
                                let bio_val = bio();
                                let x_val = x_url();
                                let gh_val = github_url();
                                let ig_val = instagram_url();

                                let mut socials = std::collections::HashMap::new();
                                if !x_val.trim().is_empty() { socials.insert("x".to_string(), x_val); }
                                if !gh_val.trim().is_empty() { socials.insert("github".to_string(), gh_val); }
                                if !ig_val.trim().is_empty() { socials.insert("instagram".to_string(), ig_val); }
                                let socials_opt = if socials.is_empty() { None } else { Some(socials) };
                                let bio_opt = if bio_val.trim().is_empty() { None } else { Some(bio_val) };

                                let mut toaster = toaster;
                                let mut user_ctx = user;
                                spawn(async move {
                                    match api::update_profile(name_val, email_val, bio_opt, socials_opt).await {
                                        Ok(_) => {
                                            if let Ok(Some(u)) = api::get_current_user().await {
                                                user_ctx.set(crate::app::AuthState::Authenticated(u));
                                            }
                                            toaster.success("Profile updated successfully!");
                                        }
                                        Err(e) => toaster.error(e.to_string()),
                                    }
                                });
                            },
                            div { class: "setting-info",
                                h3 { "Username" }
                                p { "Your public display name." }
                            }
                            input {
                                class: "setting-input",
                                style: "width: 100%; box-sizing: border-box;",
                                r#type: "text",
                                value: "{username}",
                                oninput: move |e| username.set(e.value()),
                                required: true,
                            }
                            div { class: "setting-info", style: "margin-top: 16px;",
                                h3 { "Email Address" }
                                p { "Used for account recovery and notifications." }
                            }
                            input {
                                class: "setting-input",
                                style: "width: 100%; box-sizing: border-box;",
                                r#type: "email",
                                value: "{email}",
                                oninput: move |e| email.set(e.value()),
                                required: true,
                            }
                            div { class: "setting-info", style: "margin-top: 16px;",
                                h3 { "Bio" }
                                p { "Tell people about yourself." }
                            }
                            textarea {
                                class: "setting-input",
                                style: "width: 100%; box-sizing: border-box; min-height: 100px; resize: vertical;",
                                value: "{bio}",
                                oninput: move |e| bio.set(e.value()),
                            }
                            div { class: "setting-info", style: "margin-top: 16px;",
                                h3 { "Social Links" }
                                p { "Connect your external profiles." }
                            }
                            div {
                                style: "display: flex; flex-direction: column; gap: 8px; width: 100%;",
                                input {
                                    class: "setting-input",
                                    style: "width: 100%; box-sizing: border-box;",
                                    r#type: "text",
                                    placeholder: "X (Twitter) URL",
                                    value: "{x_url}",
                                    oninput: move |e| x_url.set(e.value()),
                                }
                                input {
                                    class: "setting-input",
                                    style: "width: 100%; box-sizing: border-box;",
                                    r#type: "text",
                                    placeholder: "GitHub URL",
                                    value: "{github_url}",
                                    oninput: move |e| github_url.set(e.value()),
                                }
                                input {
                                    class: "setting-input",
                                    style: "width: 100%; box-sizing: border-box;",
                                    r#type: "text",
                                    placeholder: "Instagram URL",
                                    value: "{instagram_url}",
                                    oninput: move |e| instagram_url.set(e.value()),
                                }
                            }
                            button {
                                class: "btn-primary",
                                style: "margin-top: 8px;",
                                r#type: "submit",
                                "Save Profile"
                            }
                        }

                        h2 { style: "margin-top: 32px;", ShieldAlert { size: 20 } "Security Settings" }

                        form { class: "setting-group", style: "flex-direction: column; align-items: flex-start; gap: 12px;",
                            onsubmit: move |e| {
                                e.prevent_default();
                                let current = current_password();
                                let new_p = new_password();
                                if new_p.len() < 8 {
                                    let mut toaster = toaster;
                                    toaster.error("Password must be at least 8 characters");
                                    return;
                                }
                                let mut toaster = toaster;
                                let nav = nav;
                                spawn(async move {
                                    match api::change_password(current, new_p).await {
                                        Ok(_) => {
                                            toaster.success("Password changed successfully. Please log in again.");
                                            nav.push(Route::Login {});
                                        }
                                        Err(e) => toaster.error(e.to_string()),
                                    }
                                });
                            },
                            div { class: "setting-info",
                                h3 { "Change Password" }
                                p { "Update your password. This will log you out of all devices." }
                            }
                            input {
                                class: "setting-input",
                                style: "width: 100%; box-sizing: border-box;",
                                r#type: "password",
                                placeholder: "Current Password",
                                oninput: move |e| current_password.set(e.value()),
                                required: true,
                            }
                            input {
                                class: "setting-input",
                                style: "width: 100%; box-sizing: border-box;",
                                r#type: "password",
                                placeholder: "New Password",
                                oninput: move |e| new_password.set(e.value()),
                                required: true,
                                minlength: "8",
                            }
                            button {
                                class: "btn-primary",
                                style: "margin-top: 8px;",
                                r#type: "submit",
                                "Update Password"
                            }
                        }
                    }

                    div {
                        class: "settings-card fade-in",
                        h2 { CloudDownload { size: 20 } "Data & Privacy" }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Export Account Data" }
                                p { "Download a backup of your profile information, favorites, and upload history." }
                            }
                            div { class: "setting-control",
                                a {
                                    href: "/api/export_data",
                                    class: "btn-primary",
                                    style: "text-decoration: none; display: inline-flex; align-items: center; justify-content: center; gap: 8px;",
                                    CloudDownload { size: 16 }
                                    "Download Archive (.tar.gz)"
                                }
                            }
                        }
                    }

                    div {
                        class: "settings-card fade-in",
                        h2 { ShieldAlert { size: 20, color: "#ef4444" } span { style: "color: #ef4444;", "Danger Zone" } }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Revoke All Sessions" }
                                p { "Instantly log out of all devices by invalidating all active tokens." }
                            }
                            div { class: "setting-control",
                                button {
                                    class: "btn-danger",
                                    onclick: move |_| {
                                        let mut toaster = toaster;
                                        let nav = nav;
                                        spawn(async move {
                                            match api::revoke_sessions().await {
                                                Ok(_) => {
                                                    toaster.success("All sessions revoked. Please log in again.");
                                                    nav.push(Route::Login {});
                                                }
                                                Err(e) => toaster.error(e.to_string()),
                                            }
                                        });
                                    },
                                    "Revoke Sessions"
                                }
                            }
                        }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Delete Account" }
                                p { "Permanently delete your account and all associated data. This action cannot be undone." }
                            }
                            div { class: "setting-control",
                                button {
                                    class: "btn-danger",
                                    onclick: move |_| {
                                        let mut toaster = toaster;
                                        let nav = nav;
                                        spawn(async move {
                                            if let Ok(_) = api::delete_account().await {
                                                toaster.success("Account deleted.");
                                                nav.push(Route::Home {});
                                                #[cfg(target_arch = "wasm32")]
                                                let _ = web_sys::window().unwrap().location().reload();
                                            } else {
                                                toaster.error("Failed to delete account");
                                            }
                                        });
                                    },
                                    "Delete Account"
                                }
                            }
                        }
                    }
                }

                if active_tab() == SettingsTab::Appearance {
                    div {
                        class: "settings-card fade-in",
                        h2 { Palette { size: 20 } "Theme & Display" }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Theme Interface" }
                                p { "Select your preferred visual style." }
                            }
                            div { class: "setting-control",
                                select {
                                    class: "setting-select",
                                    value: "{theme}",
                                    onchange: move |e| theme.set(e.value()),
                                    option { "System Default" }
                                    option { "Dark Mode" }
                                    option { "Light Mode" }
                                    option { "OLED Black" }
                                }
                            }
                        }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "UI Animations" }
                                p { "Enable smooth transitions and micro-animations." }
                            }
                            div { class: "setting-control",
                                label { class: "toggle-switch",
                                    input {
                                        r#type: "checkbox",
                                        checked: animations(),
                                        onchange: move |_| animations.set(!animations())
                                    }
                                    span { class: "toggle-slider" }
                                }
                            }
                        }
                    }
                }

                if active_tab() == SettingsTab::Downloads {
                    div {
                        class: "settings-card fade-in",
                        h2 { CloudDownload { size: 20 } "Download Preferences" }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Default Resolution" }
                                p { "Preferred quality for manual downloads." }
                            }
                            div { class: "setting-control",
                                select {
                                    class: "setting-select",
                                    value: "{quality}",
                                    onchange: move |e| quality.set(e.value()),
                                    option { "Original (4K+)" }
                                    option { "High (1440p)" }
                                    option { "Standard (1080p)" }
                                }
                            }
                        }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Prefer AVIF Format" }
                                p { "Automatically download in AVIF for better quality and smaller file sizes when available." }
                            }
                            div { class: "setting-control",
                                label { class: "toggle-switch",
                                    input {
                                        r#type: "checkbox",
                                        checked: auto_download_avif(),
                                        onchange: move |_| auto_download_avif.set(!auto_download_avif())
                                    }
                                    span { class: "toggle-slider" }
                                }
                            }
                        }
                    }

                    div {
                        class: "settings-card fade-in",
                        h2 { Eye { size: 20 } "Content Filters" }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Safe Search" }
                                p { "Blur or hide potentially sensitive content in the feed." }
                            }
                            div { class: "setting-control",
                                label { class: "toggle-switch",
                                    input {
                                        r#type: "checkbox",
                                        checked: safe_search(),
                                        onchange: move |_| safe_search.set(!safe_search())
                                    }
                                    span { class: "toggle-slider" }
                                }
                            }
                        }
                    }
                }

                if active_tab() == SettingsTab::Notifications {
                    div {
                        class: "settings-card fade-in",
                        h2 { Bell { size: 20 } "Communication" }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Email Notifications" }
                                p { "Receive updates about your account and new features." }
                            }
                            div { class: "setting-control",
                                label { class: "toggle-switch",
                                    input {
                                        r#type: "checkbox",
                                        checked: email_notifs(),
                                        onchange: move |_| email_notifs.set(!email_notifs())
                                    }
                                    span { class: "toggle-slider" }
                                }
                            }
                        }

                        div { class: "setting-group",
                            div { class: "setting-info",
                                h3 { "Push Notifications" }
                                p { "Get notified when someone interacts with your uploads." }
                            }
                            div { class: "setting-control",
                                label { class: "toggle-switch",
                                    input {
                                        r#type: "checkbox",
                                        checked: push_notifs(),
                                        onchange: move |_| push_notifs.set(!push_notifs())
                                    }
                                    span { class: "toggle-slider" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

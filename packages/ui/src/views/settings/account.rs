use crate::app::Route;
use crate::use_toaster;
use dioxus::prelude::*;
use lucide_dioxus::{Camera, CloudDownload, ShieldAlert, User};

#[component]
pub fn AccountSettings(
    real_username: String,
    real_email: String,
    real_pfp_url: String,
    real_bio: Option<String>,
    real_socials: Option<std::collections::HashMap<String, String>>,
) -> Element {
    let nav = use_navigator();
    let user = use_context::<Signal<crate::app::AuthState>>();
    let toaster = use_toaster();
    let i18n = crate::i18n::use_i18n();

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

    rsx! {
        div { class: "settings-card",
            h2 {
                User { size: 20 }
                "{i18n.t(\"acc_profile_settings\")}"
            }

            div {
                class: "setting-group",
                style: "flex-direction: column; align-items: flex-start; gap: 24px; border-bottom: none; padding-bottom: 0;",
                label { style: "display: block; width: 100%; cursor: pointer;",
                    div {
                        style: "width: 100%; height: 160px; border-radius: 16px; background: linear-gradient(135deg, rgba(37, 99, 235, 0.2) 0%, rgba(5, 5, 5, 1) 100%); position: relative; border: 1px dashed rgba(255,255,255,0.2); display: flex; align-items: center; justify-content: center; transition: all 0.2s;",
                        class: "glow-hover",
                        Camera { size: 32, color: "var(--text-muted)" }
                        span { style: "position: absolute; bottom: 16px; font-size: 14px; font-weight: 600; color: var(--text-secondary);",
                            "{i18n.t(\"acc_change_banner\")}"
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
                                toaster.success(i18n.t("success_uploading_banner"));
                                if let Ok(bytes) = file.read_bytes().await {
                                    #[cfg(target_arch = "wasm32")]
                                    if let Ok(resp) = gloo_net::http::Request::post("/api/upload_media")
                                        .header("X-Media-Type", "banner")
                                        .body(bytes.to_vec())
                                        .unwrap()
                                        .send()
                                        .await
                                    {
                                        if resp.ok() {
                                            toaster.success(i18n.t("success_banner_updated"));
                                            web_sys::window().unwrap().location().reload().unwrap();
                                        } else {
                                            toaster.error(i18n.t("err_upload_failed_generic"));
                                        }
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if let Ok(resp) = reqwest::Client::new()
                                        .post("http://localhost:8080/api/upload_media")
                                        .header("X-Media-Type", "banner")
                                        .body(bytes)
                                        .send()
                                        .await
                                    {
                                        if resp.status().is_success() {
                                            toaster.success(i18n.t("success_banner_updated"));
                                            nav.push(crate::app::Route::Home {});
                                        } else {
                                            toaster.error(i18n.t("err_upload_failed_generic"));
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                div { style: "display: flex; align-items: center; gap: 24px; margin-top: -60px; margin-left: 24px; z-index: 10;",
                    label { style: "display: block; cursor: pointer;",
                        div {
                            style: "width: 100px; height: 100px; border-radius: 50%; border: 4px solid var(--bg-primary); background: var(--bg-secondary); overflow: hidden; position: relative; display: flex; align-items: center; justify-content: center;",
                            class: "glow-hover",
                            img {
                                referrerpolicy: "no-referrer",
                                src: "{crate::resolve_asset_url(&real_pfp_url)}",
                                style: "width: 100%; height: 100%; object-fit: cover; opacity: 0.5;",
                            }
                            div { style: "position: absolute;",
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
                                    toaster.success(i18n.t("success_uploading_avatar"));
                                    if let Ok(bytes) = file.read_bytes().await {
                                        #[cfg(target_arch = "wasm32")]
                                        if let Ok(resp) = gloo_net::http::Request::post("/api/upload_media")
                                            .header("X-Media-Type", "pfp")
                                            .body(bytes.to_vec())
                                            .unwrap()
                                            .send()
                                            .await
                                        {
                                            if resp.ok() {
                                                toaster.success(i18n.t("success_avatar_updated"));
                                                web_sys::window().unwrap().location().reload().unwrap();
                                            } else {
                                                toaster.error(i18n.t("err_upload_failed_generic"));
                                            }
                                        }
                                        #[cfg(not(target_arch = "wasm32"))]
                                        if let Ok(resp) = reqwest::Client::new()
                                            .post("http://localhost:8080/api/upload_media")
                                            .header("X-Media-Type", "pfp")
                                            .body(bytes)
                                            .send()
                                            .await
                                        {
                                            if resp.status().is_success() {
                                                toaster.success(i18n.t("success_avatar_updated"));
                                                nav.push(crate::app::Route::Home {});
                                            } else {
                                                toaster.error(i18n.t("err_upload_failed_generic"));
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                    div { style: "margin-top: 40px;",
                        h3 { style: "font-size: 16px; font-weight: 600; margin-bottom: 4px;",
                            "{i18n.t(\"acc_profile_avatar\")}"
                        }
                        p { style: "font-size: 14px; color: var(--text-muted);",
                            "{i18n.t(\"acc_avatar_rec\")}"
                        }
                    }
                }
            }
            form {
                class: "setting-group",
                style: "flex-direction: column; align-items: flex-start; gap: 12px;",
                onsubmit: move |e| {
                    e.prevent_default();
                    let name_val = username();
                    let email_val = email();
                    let bio_val = bio();
                    let x_val = x_url();
                    let gh_val = github_url();
                    let ig_val = instagram_url();

                    let mut socials = std::collections::HashMap::new();
                    if !x_val.trim().is_empty() {

                        socials.insert("x".to_string(), x_val);
                    }
                    if !gh_val.trim().is_empty() {
                        socials.insert("github".to_string(), gh_val);
                    }
                    if !ig_val.trim().is_empty() {
                        socials.insert("instagram".to_string(), ig_val);
                    }
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
                                toaster.success(i18n.t("success_profile_updated"));
                            }
                            Err(e) => toaster.error(e.to_string()),
                        }
                    });
                },
                div { class: "setting-info",
                    h3 { "{i18n.t(\"username\")}" }
                    p { "{i18n.t(\"acc_username_desc\")}" }
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
                    h3 { "{i18n.t(\"email_address\")}" }
                    p { "{i18n.t(\"acc_email_desc\")}" }
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
                    h3 { "{i18n.t(\"bio\")}" }
                    p { "{i18n.t(\"acc_bio_desc\")}" }
                }
                textarea {
                    class: "setting-input",
                    style: "width: 100%; box-sizing: border-box; min-height: 100px; resize: vertical;",
                    value: "{bio}",
                    oninput: move |e| bio.set(e.value()),
                }
                div { class: "setting-info", style: "margin-top: 16px;",
                    h3 { "{i18n.t(\"social_links\")}" }
                    p { "{i18n.t(\"acc_socials_desc\")}" }
                }
                div { style: "display: flex; flex-direction: column; gap: 8px; width: 100%;",
                    input {
                        class: "setting-input",
                        style: "width: 100%; box-sizing: border-box;",
                        r#type: "text",
                        placeholder: "{i18n.t(\"acc_x_url\")}",
                        value: "{x_url}",
                        oninput: move |e| x_url.set(e.value()),
                    }
                    input {
                        class: "setting-input",
                        style: "width: 100%; box-sizing: border-box;",
                        r#type: "text",
                        placeholder: "{i18n.t(\"acc_github_url\")}",
                        value: "{github_url}",
                        oninput: move |e| github_url.set(e.value()),
                    }
                    input {
                        class: "setting-input",
                        style: "width: 100%; box-sizing: border-box;",
                        r#type: "text",
                        placeholder: "{i18n.t(\"acc_instagram_url\")}",
                        value: "{instagram_url}",
                        oninput: move |e| instagram_url.set(e.value()),
                    }
                }
                button {
                    class: "btn-primary",
                    style: "margin-top: 8px;",
                    r#type: "submit",
                    "{i18n.t(\"acc_save_profile\")}"
                }
            }

            h2 { style: "margin-top: 32px;",
                ShieldAlert { size: 20 }
                "{i18n.t(\"acc_security_settings\")}"
            }

            form {
                class: "setting-group",
                style: "flex-direction: column; align-items: flex-start; gap: 12px;",
                onsubmit: move |e| {
                    e.prevent_default();
                    let current = current_password();
                    let new_p = new_password();
                    if new_p.len() < 8 {
                        let mut toaster = toaster;
                        toaster.error(i18n.t("err_password_length"));
                        return;
                    }
                    let mut toaster = toaster;
                    let nav = nav;
                    spawn(async move {
                        match api::change_password(current, new_p).await {
                            Ok(_) => {
                                toaster.success(i18n.t("success_password_changed"));
                                nav.push(Route::Login {});
                            }
                            Err(e) => toaster.error(e.to_string()),
                        }
                    });
                },
                div { class: "setting-info",
                    h3 { "{i18n.t(\"change_password\")}" }
                    p { "{i18n.t(\"acc_update_password_desc\")}" }
                }
                input {
                    class: "setting-input",
                    style: "width: 100%; box-sizing: border-box;",
                    r#type: "password",
                    placeholder: "{i18n.t(\"acc_current_password\")}",
                    oninput: move |e| current_password.set(e.value()),
                    required: true,
                }
                input {
                    class: "setting-input",
                    style: "width: 100%; box-sizing: border-box;",
                    r#type: "password",
                    placeholder: "{i18n.t(\"acc_new_password\")}",
                    oninput: move |e| new_password.set(e.value()),
                    required: true,
                    minlength: "8",
                }
                button {
                    class: "btn-primary",
                    style: "margin-top: 8px;",
                    r#type: "submit",
                    "{i18n.t(\"acc_update_password\")}"
                }
            }
        }

        div { class: "settings-card",
            h2 {
                CloudDownload { size: 20 }
                "{i18n.t(\"acc_data_privacy\")}"
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"export_account_data\")}" }
                    p { "{i18n.t(\"acc_download_backup_desc\")}" }
                }
                div { class: "setting-control",
                    a {
                        href: "/api/export_data",
                        class: "btn-primary",
                        style: "text-decoration: none; display: inline-flex; align-items: center; justify-content: center; gap: 8px;",
                        CloudDownload { size: 16 }
                        "{i18n.t(\"acc_download_archive\")}"
                    }
                }
            }
        }

        div { class: "settings-card",
            h2 {
                ShieldAlert { size: 20, color: "#ef4444" }
                span { style: "color: #ef4444;", "{i18n.t(\"acc_danger_zone\")}" }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"acc_revoke_sessions\")}" }
                    p { "{i18n.t(\"acc_revoke_sessions_desc\")}" }
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
                                        toaster.success(i18n.t("success_sessions_revoked"));
                                        nav.push(Route::Login {});
                                    }
                                    Err(e) => toaster.error(e.to_string()),
                                }
                            });
                        },
                        "{i18n.t(\"acc_revoke_btn\")}"
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"delete_account\")}" }
                    p { "{i18n.t(\"acc_delete_desc\")}" }
                }
                div { class: "setting-control",
                    button {
                        class: "btn-danger",
                        onclick: move |_| {
                            let mut toaster = toaster;
                            let nav = nav;
                            spawn(async move {
                                if let Ok(_) = api::delete_account().await {
                                    toaster.success(i18n.t("success_account_deleted"));
                                    nav.push(Route::Home {});
                                    #[cfg(target_arch = "wasm32")]
                                    let _ = web_sys::window().unwrap().location().reload();
                                } else {
                                    toaster.error(i18n.t("err_delete_account"));
                                }
                            });
                        },
                        "{i18n.t(\"delete_account\")}"
                    }
                }
            }
        }
    }
}

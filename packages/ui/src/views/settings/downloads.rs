use dioxus::prelude::*;
use lucide_dioxus::{CloudDownload, Eye};

#[component]
pub fn DownloadsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut toaster = crate::use_toaster();
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    
    let mut quality = use_signal(|| "Original (4K+)".to_string());
    let mut auto_download_avif = use_signal(|| true);
    let mut safe_search = use_signal(|| true);
    
    // Additional signals to trigger update without infinite loops
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if let crate::app::AuthState::Authenticated(u) = user_ctx() {
            if !initialized() {
                quality.set(u.download_quality.clone());
                auto_download_avif.set(u.auto_download_avif);
                safe_search.set(u.safe_search);
                initialized.set(true);
            }
        }
    });

    let q_val = quality();
    let avif_val = auto_download_avif();
    let ss_val = safe_search();
    let init_val = initialized();
    let user_state = user_ctx();

    use_effect(move || {
        if init_val {
            if let crate::app::AuthState::Authenticated(u) = &user_state {
                let u = u.clone();
                let q_val_clone = q_val.clone();
                spawn(async move {
                    if let Err(e) = api::update_preferences(
                        u.email_notifs,
                        u.push_notifs,
                        q_val_clone,
                        avif_val,
                        ss_val
                    ).await {
                        toaster.error(format!("Failed to save preferences: {}", e));
                    }
                });
            }
        }
    });

    rsx! {
        div {
            class: "settings-card",
            h2 { CloudDownload { size: 20 } "{i18n.t(\"dl_preferences\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"dl_default_res\")}" }
                    p { "{i18n.t(\"dl_default_res_desc\")}" }
                }
                div { class: "setting-control",
                    select {
                        class: "setting-select",
                        value: "{quality}",
                        onchange: move |e| quality.set(e.value()),
                        option { value: "Original (4K+)", "{i18n.t(\"dl_res_original\")}" }
                        option { value: "High (1440p)", "{i18n.t(\"dl_res_high\")}" }
                        option { value: "Standard (1080p)", "{i18n.t(\"dl_res_standard\")}" }
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"dl_prefer_avif\")}" }
                    p { "{i18n.t(\"dl_prefer_avif_desc\")}" }
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
            class: "settings-card",
            h2 { Eye { size: 20 } "{i18n.t(\"dl_content_filters\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"dl_safe_search\")}" }
                    p { "{i18n.t(\"dl_safe_search_desc\")}" }
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
}

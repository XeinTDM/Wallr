use super::use_stored_signal;
use dioxus::prelude::*;
use lucide_dioxus::{CloudDownload, Eye};

#[component]
pub fn DownloadsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut quality = use_stored_signal("settings_quality", "Original (4K+)".to_string());
    let mut auto_download_avif = use_stored_signal("settings_auto_download_avif", true);
    let mut safe_search = use_stored_signal("settings_safe_search", true);

    rsx! {
        div {
            class: "settings-card fade-in",
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
            class: "settings-card fade-in",
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

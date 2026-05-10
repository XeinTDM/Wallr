use super::use_stored_signal;
use dioxus::prelude::*;
use lucide_dioxus::Palette;

#[component]
pub fn AppearanceSettings() -> Element {
    let mut theme = use_stored_signal("settings_theme", "dark".to_string());
    let mut animations = use_stored_signal("settings_animations", true);
    let i18n = crate::i18n::use_i18n();

    rsx! {
        div {
            class: "settings-card fade-in",
            h2 { Palette { size: 20 } "{i18n.t(\"appr_theme_display\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"appr_theme_interface\")}" }
                    p { "{i18n.t(\"appr_theme_desc\")}" }
                }
                div { class: "setting-control",
                    select {
                        class: "setting-select",
                        value: "{theme}",
                        onchange: move |e| theme.set(e.value()),
                        option { value: "system", "{i18n.t(\"system_default\")}" }
                        option { value: "dark", "{i18n.t(\"dark_mode\")}" }
                        option { value: "light", "{i18n.t(\"light_mode\")}" }
                        option { value: "oled", "{i18n.t(\"oled_black\")}" }
                    }
                }
            }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"appr_animations\")}" }
                    p { "{i18n.t(\"appr_animations_desc\")}" }
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
}

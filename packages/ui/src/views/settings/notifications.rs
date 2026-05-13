use super::use_stored_signal;
use dioxus::prelude::*;
use lucide_dioxus::Bell;

#[component]
pub fn NotificationsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut email_notifs = use_stored_signal("settings_email_notifs", true);
    let mut push_notifs = use_stored_signal("settings_push_notifs", false);

    rsx! {
        div {
            class: "settings-card",
            h2 { Bell { size: 20 } "{i18n.t(\"notif_communication\")}" }

            div { class: "setting-group",
                div { class: "setting-info",
                    h3 { "{i18n.t(\"notif_email\")}" }
                    p { "{i18n.t(\"notif_email_desc\")}" }
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
                    h3 { "{i18n.t(\"notif_push\")}" }
                    p { "{i18n.t(\"notif_push_desc\")}" }
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

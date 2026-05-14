use dioxus::prelude::*;
use lucide_dioxus::Bell;

#[component]
pub fn NotificationsSettings() -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut toaster = crate::use_toaster();
    let user_ctx = use_context::<Signal<crate::app::AuthState>>();
    
    let mut email_notifs = use_signal(|| true);
    let mut push_notifs = use_signal(|| false);
    
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if let crate::app::AuthState::Authenticated(u) = user_ctx() {
            if !initialized() {
                email_notifs.set(u.email_notifs);
                push_notifs.set(u.push_notifs);
                initialized.set(true);
            }
        }
    });

    let en_val = email_notifs();
    let pn_val = push_notifs();
    let init_val = initialized();
    let user_state = user_ctx();

    use_effect(move || {
        if init_val {
            if let crate::app::AuthState::Authenticated(u) = &user_state {
                let u = u.clone();
                spawn(async move {
                    if let Err(e) = api::update_preferences(
                        en_val,
                        pn_val,
                        u.download_quality,
                        u.auto_download_avif,
                        u.safe_search
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

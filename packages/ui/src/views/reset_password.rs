use crate::app::Route;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowRight, Key};

#[component]
pub fn ResetPassword(token: String) -> Element {
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut success = use_signal(|| false);
    let mut toaster = crate::toast::use_toaster();
    let i18n = crate::i18n::use_i18n();

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();

        if password() != confirm_password() {
            error.set(Some(i18n.t("err_passwords_not_match").to_string()));
            return;
        }

        if password().len() < 8 {
            error.set(Some(i18n.t("err_password_length").to_string()));
            return;
        }

        let token_for_spawn = token.clone();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match api::reset_password_with_token(token_for_spawn, password()).await {
                Ok(_) => {
                    success.set(true);
                    toaster.success(i18n.t("success_password_reset"));
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    toaster.error(i18n.t("err_password_reset"));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 160px 0 80px; display: flex; justify-content: center;",

            div {
                class: "glass",
                style: "width: 100%; max-width: 450px; padding: 48px; border-radius: 32px; border: 1px solid rgba(255,255,255,0.1); text-align: center;",

                h2 { style: "font-size: 32px; font-weight: 900; margin-bottom: 8px;", "{i18n.t(\"reset_create_new\")}" }

                if success() {
                    div {
                        style: "margin-top: 24px;",
                        p { style: "color: var(--text-secondary); margin-bottom: 32px;", "{i18n.t(\"reset_success_msg\")}" }
                        Link {
                            to: Route::Login {},
                            class: "btn-primary glow-hover",
                            style: "padding: 16px; border-radius: 16px; font-weight: 800; color: white; background: var(--accent-primary, #3b82f6); border: none; display: flex; align-items: center; justify-content: center; gap: 8px; text-decoration: none;",
                            "{i18n.t(\"btn_log_in\")}"
                            ArrowRight { size: 20 }
                        }
                    }
                } else {
                    p { style: "color: var(--text-secondary); margin-bottom: 40px;", "{i18n.t(\"reset_enter_strong\")}" }

                    form {
                        style: "display: grid; gap: 16px; text-align: left;",
                        onsubmit: on_submit,

                        div {
                            label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "{i18n.t(\"reset_new_password\")}" }
                            input {
                                r#type: "password",
                                class: "glass",
                                style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                                placeholder: "••••••••",
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                                required: true,
                            }
                        }

                        div {
                            label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "{i18n.t(\"reset_confirm_password\")}" }
                            input {
                                r#type: "password",
                                class: "glass",
                                style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                                placeholder: "••••••••",
                                value: "{confirm_password}",
                                oninput: move |e| confirm_password.set(e.value()),
                                required: true,
                            }
                        }

                        if let Some(err) = error() {
                            p { style: "color: #ff4d4d; font-size: 13px; margin: 0; font-weight: 600;", "{err}" }
                        }

                        button {
                            r#type: "submit",
                            class: "btn-primary glow-hover",
                            style: "margin-top: 8px; padding: 16px; border-radius: 16px; font-weight: 800; color: white; background: var(--accent-primary, #3b82f6); border: none; display: flex; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;",
                            opacity: if is_loading() { 0.7 } else { 1.0 },
                            cursor: if is_loading() { "not-allowed" } else { "pointer" },
                            disabled: is_loading(),
                            Key { size: 20 }
                            if is_loading() { "{i18n.t(\"resetting\")}" } else { "{i18n.t(\"btn_reset_password\")}" }
                        }
                    }
                }
            }
        }
    }
}

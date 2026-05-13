use crate::app::Route;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Send};

#[component]
pub fn ForgotPassword() -> Element {
    let mut email = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut success = use_signal(|| false);

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        spawn(async move {
            is_loading.set(true);
            let _ = api::request_password_reset(email()).await;
            success.set(true);
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

                h2 { style: "font-size: 32px; font-weight: 900; margin-bottom: 8px;", "Reset Password" }

                if success() {
                    div {
                        style: "margin-top: 24px;",
                        p { style: "color: var(--text-secondary); margin-bottom: 32px;", "If an account exists for that email, we have sent a password reset link." }
                        Link {
                            to: Route::Login {},
                            class: "btn-primary glow-hover",
                            style: "padding: 16px; border-radius: 16px; font-weight: 800; color: white; background: var(--accent-primary, #3b82f6); border: none; display: flex; align-items: center; justify-content: center; gap: 8px; text-decoration: none;",
                            "Return to Login"
                        }
                    }
                } else {
                    p { style: "color: var(--text-secondary); margin-bottom: 40px;", "Enter your email address to receive a reset link." }

                    form {
                        style: "display: grid; gap: 16px; text-align: left;",
                        onsubmit: on_submit,
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "Email" }
                            input {
                                r#type: "email",
                                class: "glass",
                                style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                                placeholder: "alex@example.com",
                                value: "{email}",
                                oninput: move |e| email.set(e.value()),
                                required: true,
                            }
                        }

                        button {
                            r#type: "submit",
                            class: "btn-primary glow-hover",
                            style: "margin-top: 8px; padding: 16px; border-radius: 16px; font-weight: 800; color: white; background: var(--accent-primary, #3b82f6); border: none; display: flex; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;",
                            opacity: if is_loading() { 0.7 } else { 1.0 },
                            cursor: if is_loading() { "not-allowed" } else { "pointer" },
                            disabled: is_loading(),
                            Send { size: 20 }
                            if is_loading() { "Sending..." } else { "Send Reset Link" }
                        }
                    }
                }

                div {
                    style: "margin-top: 32px;",
                    Link {
                        to: Route::Login {},
                        style: "color: var(--text-secondary); font-size: 14px; display: inline-flex; align-items: center; gap: 8px; text-decoration: none;",
                        ArrowLeft { size: 16 }
                        "Back to login"
                    }
                }
            }
        }
    }
}

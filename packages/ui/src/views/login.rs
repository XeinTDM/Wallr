use crate::app::Route;
use dioxus::prelude::*;
use lucide_dioxus::{GitBranch, LogIn, MessageSquare};

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    let mut user_signal = use_context::<Signal<crate::app::AuthState>>();
    let nav = use_navigator();
    let i18n = crate::i18n::use_i18n();

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            match api::login(email(), password()).await {
                Ok(_) => {
                    if let Ok(Some(u)) = api::get_current_user().await {
                        user_signal.set(crate::app::AuthState::Authenticated(u));
                    }
                    nav.push(Route::Home {});
                    return;
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
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

                h2 { style: "font-size: 32px; font-weight: 900; margin-bottom: 8px;", "{i18n.t(\"welcome_back\")}" }
                p { style: "color: var(--text-secondary); margin-bottom: 40px;", "{i18n.t(\"login_subtitle\")}" }

                div {
                    style: "display: grid; gap: 20px;",

                    a {
                        href: "/api/oauth/google/login",
                        class: "glass glow-hover",
                        style: "padding: 16px; border-radius: 16px; display: flex; align-items: center; justify-content: center; gap: 12px; font-weight: 700; color: white; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); text-decoration: none; cursor: pointer; transition: all 0.2s ease;",
                        lucide_dioxus::Mail { size: 20 }
                        "{i18n.t(\"continue_google\")}"
                    }

                    a {
                        href: "/api/oauth/github/login",
                        class: "glass glow-hover",
                        style: "padding: 16px; border-radius: 16px; display: flex; align-items: center; justify-content: center; gap: 12px; font-weight: 700; color: white; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); text-decoration: none; cursor: pointer; transition: all 0.2s ease;",
                        GitBranch { size: 20 }
                        "{i18n.t(\"continue_github\")}"
                    }

                    a {
                        href: "/api/oauth/discord/login",
                        class: "glass glow-hover",
                        style: "padding: 16px; border-radius: 16px; display: flex; align-items: center; justify-content: center; gap: 12px; font-weight: 700; color: white; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); text-decoration: none; cursor: pointer; transition: all 0.2s ease;",
                        MessageSquare { size: 20 }
                        "{i18n.t(\"continue_discord\")}"
                    }
                }

                div {
                    style: "margin: 32px 0; display: flex; align-items: center; gap: 16px; color: var(--text-muted); font-size: 14px;",
                    div { style: "flex: 1; height: 1px; background: rgba(255,255,255,0.1);" }
                    "OR"
                    div { style: "flex: 1; height: 1px; background: rgba(255,255,255,0.1);" }
                }

                form {
                    style: "display: grid; gap: 16px; text-align: left;",
                    onsubmit: on_submit,
                    div {
                        label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "{i18n.t(\"email\")}" }
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
                    div {
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            label { style: "font-size: 14px; font-weight: 600;", "{i18n.t(\"password\")}" }
                            Link {
                                to: Route::ForgotPassword {},
                                style: "font-size: 13px; color: var(--accent-primary); text-decoration: none; font-weight: 600;",
                                "{i18n.t(\"forgot_password\")}"
                            }
                        }
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

                    if let Some(err) = error() {
                        p { style: "color: #ff4d4d; font-size: 13px; margin: 0; font-weight: 600;", "{i18n.t(&err)}" }
                    }

                    button {
                        r#type: "submit",
                        class: "btn-primary glow-hover",
                        style: "margin-top: 8px; padding: 16px; border-radius: 16px; font-weight: 800; color: white; background: var(--accent-primary, #3b82f6); border: none; display: flex; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;",
                        opacity: if is_loading() { 0.7 } else { 1.0 },
                        cursor: if is_loading() { "not-allowed" } else { "pointer" },
                        disabled: is_loading(),
                        LogIn { size: 20 }
                        if is_loading() { "{i18n.t(\"signing_in\")}" } else { "{i18n.t(\"sign_in\")}" }
                    }
                }

                p {
                    style: "margin-top: 32px; font-size: 14px; color: var(--text-secondary);",
                    "New to Wallr? "
                    Link { to: Route::Register {}, style: "color: var(--accent-primary); font-weight: 700;", "{i18n.t(\"create_an_account\")}" }
                }

                div {
                    style: "margin-top: 16px; padding: 12px; border-radius: 12px; background: rgba(59, 130, 246, 0.1); border: 1px solid rgba(59, 130, 246, 0.2); font-size: 12px; color: #60a5fa;",
                    "Demo Account: alex@example.com / password123"
                }
            }
        }
    }
}

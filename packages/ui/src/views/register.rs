use dioxus::prelude::*;
use crate::app::Route;
use lucide_dioxus::UserPlus;

#[component]
pub fn Register() -> Element {
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    
    let mut user_signal = use_context::<Signal<crate::app::AuthState>>();
    let nav = use_navigator();

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        spawn(async move {
            if name().is_empty() || email().is_empty() || password().is_empty() {
                error.set(Some("Please fill in all fields".into()));
                return;
            }

            is_loading.set(true);
            error.set(None);
            
            match api::register(name(), email(), password()).await {
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
            class: "container fade-in",
            style: "padding: 160px 0 80px; display: flex; justify-content: center;",
            
            div {
                class: "glass",
                style: "width: 100%; max-width: 450px; padding: 48px; border-radius: 32px; border: 1px solid rgba(255,255,255,0.1); text-align: center;",
                
                h2 { style: "font-size: 32px; font-weight: 900; margin-bottom: 8px;", "Join Wallr" }
                p { style: "color: var(--text-secondary); margin-bottom: 40px;", "Start your curated wallpaper journey today." }
                
                form {
                    style: "display: grid; gap: 16px; text-align: left;",
                    onsubmit: on_submit,
                    div {
                        label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "Full Name" }
                        input {
                            class: "glass",
                            style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                            placeholder: "Alex Dev",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            required: true,
                        }
                    }
                    div {
                        label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "Email" }
                        input {
                            r#type: "email",
                            class: "glass",
                            style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                            placeholder: "you@example.com",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            required: true,
                        }
                    }
                    div {
                        label { style: "display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600;", "Password" }
                        input {
                            r#type: "password",
                            class: "glass",
                            style: "width: 100%; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white;",
                            placeholder: "••••••••",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            required: true,
                            minlength: "8",
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
                        UserPlus { size: 20 }
                        if is_loading() { "Creating account..." } else { "Create Account" }
                    }
                }
                
                p {
                    style: "margin-top: 32px; font-size: 14px; color: var(--text-secondary);",
                    "Already have an account? "
                    Link { to: Route::Login {}, style: "color: var(--accent-primary); font-weight: 700;", "Sign in" }
                }
            }
        }
    }
}

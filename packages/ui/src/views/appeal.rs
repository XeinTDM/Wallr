use crate::app::{AuthState, Route};
use api::submit_moderation_appeal;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Send};

#[component]
pub fn Appeal(target_type: String, target_id: String) -> Element {
    let nav = use_navigator();
    let mut toaster = crate::toast::use_toaster();
    let auth_state = use_context::<Signal<AuthState>>();

    let is_logged_in = matches!(auth_state(), AuthState::Authenticated(_));

    if !is_logged_in {
        nav.push(Route::Login {});
        return rsx! { div {} };
    }

    let mut reason = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let target_type_submit = target_type.clone();
    let target_id_submit = target_id.clone();

    let onsubmit = move |evt: FormEvent| {
        evt.stop_propagation();
        if reason().trim().is_empty() {
            toaster.error("Please provide a reason for your appeal.");
            return;
        }

        is_submitting.set(true);
        let target_type_inner = target_type_submit.clone();
        let target_id_inner = target_id_submit.clone();
        
        spawn(async move {
            match submit_moderation_appeal(target_id_inner, target_type_inner, reason()).await {
                Ok(_) => {
                    toaster.success("Appeal submitted successfully.");
                    nav.push(Route::Home {});
                }
                Err(e) => {
                    toaster.error(&format!("Failed to submit appeal: {}", e));
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 120px 0 80px; max-width: 600px; margin: 0 auto;",
            
            button {
                style: "background: none; border: none; color: var(--text-secondary); display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 0; margin-bottom: 24px; font-size: 16px;",
                onclick: move |_| {
                    if nav.can_go_back() {
                        nav.go_back();
                    } else {
                        nav.push(Route::Home {});
                    }
                },
                ArrowLeft { size: 20 }
                "Back"
            }

            div {
                class: "glass",
                style: "border-radius: 24px; padding: 40px;",

                h1 {
                    style: "font-size: 28px; font-weight: 800; margin: 0 0 16px 0;",
                    "Submit Moderation Appeal"
                }
                
                p {
                    style: "color: var(--text-secondary); margin: 0 0 32px 0; line-height: 1.6;",
                    "If you believe a moderation action was taken in error, you can submit an appeal. Please provide a detailed explanation of why you think the decision should be reversed."
                }

                div {
                    style: "background: rgba(255,255,255,0.03); padding: 16px; border-radius: 12px; margin-bottom: 24px;",
                    div {
                        style: "font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px;",
                        "Target Type"
                    }
                    div {
                        style: "font-weight: 600;",
                        "{target_type}"
                    }
                    div {
                        style: "font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-top: 12px; margin-bottom: 4px;",
                        "Target ID"
                    }
                    div {
                        style: "font-family: monospace; font-size: 14px; background: rgba(0,0,0,0.3); padding: 4px 8px; border-radius: 4px; display: inline-block;",
                        "{target_id}"
                    }
                }

                form {
                    onsubmit: onsubmit,
                    style: "display: flex; flex-direction: column; gap: 24px;",

                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        label {
                            style: "font-weight: 600; font-size: 14px;",
                            "Reason for Appeal"
                        }
                        textarea {
                            class: "input-field",
                            style: "min-height: 120px; resize: vertical; padding: 16px;",
                            placeholder: "Explain why the moderation action should be reversed...",
                            value: "{reason}",
                            oninput: move |e| reason.set(e.value().clone()),
                            required: true,
                        }
                    }

                    button {
                        class: "btn btn-primary",
                        style: "width: 100%; justify-content: center; gap: 8px;",
                        type: "submit",
                        disabled: is_submitting(),
                        if is_submitting() {
                            "Submitting..."
                        } else {
                            Send { size: 18 }
                            "Submit Appeal"
                        }
                    }
                }
            }
        }
    }
}

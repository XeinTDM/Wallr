use dioxus::prelude::*;

#[component]
pub fn About() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "{i18n.t(\"info_about_title\")}"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "font-size: 20px; color: white; margin-bottom: 24px; font-weight: 600;",
                        "{i18n.t(\"info_about_p1\")}"
                    }
                    p {
                        style: "margin-bottom: 24px;",
                        "{i18n.t(\"info_about_p2\")}"
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"info_about_h2_1\")}" }
                    p { style: "margin-bottom: 16px;", "{i18n.t(\"info_about_p3\")}" }

                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"info_about_li1_strong\")}" } "{i18n.t(\"info_about_li1\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"info_about_li2_strong\")}" } "{i18n.t(\"info_about_li2\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"info_about_li3_strong\")}" } "{i18n.t(\"info_about_li3\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"info_about_h2_2\")}" }
                    p { "{i18n.t(\"info_about_p4\")}" }
                }
            }
        }
    }
}

#[component]
pub fn FAQ() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "{i18n.t(\"info_faq_title\")}"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    h3 { style: "color: white; margin: 0 0 12px; font-size: 20px;", "{i18n.t(\"info_faq_q1\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"info_faq_a1\")}" }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "{i18n.t(\"info_faq_q2\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"info_faq_a2\")}" }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "{i18n.t(\"info_faq_q3\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"info_faq_a3\")}" }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "{i18n.t(\"info_faq_q4\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"info_faq_a4\")}" }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "{i18n.t(\"info_faq_q5\")}" }
                    p { "{i18n.t(\"info_faq_a5\")}" }
                }
            }
        }
    }
}

#[component]
pub fn ContactUs() -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "{i18n.t(\"info_contact_title\")}"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "font-size: 18px; margin-bottom: 32px;",
                        "{i18n.t(\"info_contact_p1\")}"
                    }

                    div {
                        style: "margin-top: 32px; padding-bottom: 24px; border-bottom: 1px solid rgba(255, 255, 255, 0.1);",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "{i18n.t(\"info_contact_h4_1\")}" }
                        p { "{i18n.t(\"info_contact_p2\")}" }
                        a {
                            href: "mailto:support@wallr.dev",
                            style: "color: #4a90e2; text-decoration: none; font-weight: bold;",
                            "support@wallr.dev"
                        }
                    }

                    div {
                        style: "margin-top: 24px; padding-bottom: 24px; border-bottom: 1px solid rgba(255, 255, 255, 0.1);",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "{i18n.t(\"info_contact_h4_2\")}" }
                        p { "{i18n.t(\"info_contact_p3\")}" }
                        div {
                            style: "display: flex; gap: 16px; margin-top: 12px;",
                            a {
                                href: "mailto:legal@wallr.dev",
                                style: "color: #4a90e2; text-decoration: none; font-weight: bold;",
                                "legal@wallr.dev"
                            }
                            Link {
                                to: crate::app::Route::Dmca {},
                                style: "color: #ef4444; text-decoration: none; font-weight: bold;",
                                "Submit DMCA Claim"
                            }
                        }
                    }

                    div {
                        style: "margin-top: 24px;",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "{i18n.t(\"info_contact_h4_3\")}" }
                        p { "{i18n.t(\"info_contact_p4\")}" }
                        div {
                            style: "display: flex; gap: 16px; margin-top: 12px;",
                            a {
                                href: "#",
                                style: "color: white; text-decoration: underline;",
                                "{i18n.t(\"info_contact_github\")}"
                            }
                            a {
                                href: "#",
                                style: "color: white; text-decoration: underline;",
                                "{i18n.t(\"info_contact_discord\")}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Dmca() -> Element {
    let mut toaster = crate::toast::use_toaster();
    let nav = use_navigator();
    let _i18n = crate::i18n::use_i18n();

    let mut wallpaper_id = use_signal(String::new);
    let mut claimant_name = use_signal(String::new);
    let mut claimant_email = use_signal(String::new);
    let mut original_url = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut digital_signature = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let submit_action = move |_| {
        if wallpaper_id().is_empty() || claimant_name().is_empty() || claimant_email().is_empty() || description().is_empty() || digital_signature().is_empty() {
            toaster.error("Please fill in all required fields.");
            return;
        }

        is_submitting.set(true);
        let id = wallpaper_id();
        let name = claimant_name();
        let email = claimant_email();
        let orig_url = original_url();
        let url_opt = if orig_url.is_empty() { None } else { Some(orig_url) };
        let desc = description();
        let sig = digital_signature();

        spawn(async move {
            if let Ok(_) = api::submit_dmca_claim(id, name, email, url_opt, desc, sig, None).await {
                toaster.success("DMCA claim submitted successfully.");
                nav.push(crate::app::Route::Home {});
            } else {
                toaster.error("Failed to submit DMCA claim. Please verify the Wallpaper ID.");
                is_submitting.set(false);
            }
        });
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "Submit DMCA Takedown Notice"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "font-size: 16px; margin-bottom: 32px;",
                        "If you believe that your copyrighted work has been copied in a way that constitutes copyright infringement and is accessible on Wallr, please notify us by filling out the form below."
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Wallpaper ID *" }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none;",
                                placeholder: "e.g., 094a8711da...",
                                value: "{wallpaper_id}",
                                oninput: move |e| wallpaper_id.set(e.value())
                            }
                        }
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Your Full Name *" }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none;",
                                placeholder: "John Doe",
                                value: "{claimant_name}",
                                oninput: move |e| claimant_name.set(e.value())
                            }
                        }
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Your Email Address *" }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none;",
                                placeholder: "john@example.com",
                                r#type: "email",
                                value: "{claimant_email}",
                                oninput: move |e| claimant_email.set(e.value())
                            }
                        }
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Original Work URL (Optional)" }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none;",
                                placeholder: "https://your-portfolio.com/original-art",
                                value: "{original_url}",
                                oninput: move |e| original_url.set(e.value())
                            }
                        }
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Description of Infringement *" }
                            textarea {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none; min-height: 120px; resize: vertical;",
                                placeholder: "Describe how your copyright is being infringed...",
                                value: "{description}",
                                oninput: move |e| description.set(e.value())
                            }
                        }
                        div {
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold; color: white;", "Digital Signature *" }
                            p { style: "font-size: 13px; color: var(--text-muted); margin-top: 0; margin-bottom: 8px;", "By typing your full name, you act as the digital signature for this claim and swear under penalty of perjury that the information is accurate." }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.2); color: white; outline: none;",
                                placeholder: "Type your full legal name",
                                value: "{digital_signature}",
                                oninput: move |e| digital_signature.set(e.value())
                            }
                        }

                        button {
                            class: "glow-hover",
                            style: format!("margin-top: 16px; padding: 16px; border-radius: 12px; border: none; background: rgba(239, 68, 68, 0.2); color: #f87171; font-weight: 700; font-size: 16px; cursor: {}; transition: all 0.2s;", if is_submitting() { "not-allowed" } else { "pointer" }),
                            disabled: is_submitting(),
                            onclick: submit_action,
                            if is_submitting() {
                                "Submitting..."
                            } else {
                                "Submit DMCA Claim"
                            }
                        }
                    }
                }
            }
        }
    }
}

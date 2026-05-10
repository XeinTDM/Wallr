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
                        a {
                            href: "mailto:legal@wallr.dev",
                            style: "color: #4a90e2; text-decoration: none; font-weight: bold;",
                            "legal@wallr.dev"
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

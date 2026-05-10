use dioxus::prelude::*;

#[component]
pub fn TermsOfService() -> Element {
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
                    "{i18n.t(\"tos_title\")}"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "margin-bottom: 24px; font-weight: bold;",
                        "{i18n.t(\"tos_updated\")}"
                    }

                    p {
                        style: "margin-bottom: 32px;",
                        "{i18n.t(\"tos_intro\")}"
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h1\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"tos_p1\")}" }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h2\")}" }
                    p { "{i18n.t(\"tos_p2\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"tos_p2_1_strong\")}" } "{i18n.t(\"tos_p2_1\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"tos_p2_2_strong\")}" } "{i18n.t(\"tos_p2_2\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"tos_p2_3_strong\")}" } "{i18n.t(\"tos_p2_3\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h3\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"tos_p3\")}" }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h4\")}" }
                    p { "{i18n.t(\"tos_p4\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"tos_p4_1\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"tos_p4_2\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"tos_p4_3\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"tos_p4_4\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h5\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"tos_p5\")}" }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"tos_h6\")}" }
                    p { "{i18n.t(\"tos_p6\")}" }
                }
            }
        }
    }
}

#[component]
pub fn PrivacyPolicy() -> Element {
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
                    "{i18n.t(\"pp_title\")}"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "margin-bottom: 24px; font-weight: bold;",
                        "{i18n.t(\"tos_updated\")}"
                    }

                    p {
                        style: "margin-bottom: 32px;",
                        "{i18n.t(\"pp_intro\")}"
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h1\")}" }
                    p { "{i18n.t(\"pp_p1\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"pp_p1_1_strong\")}" } "{i18n.t(\"pp_p1_1\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"pp_p1_2_strong\")}" } "{i18n.t(\"pp_p1_2\")}" }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"pp_p1_3_strong\")}" } "{i18n.t(\"pp_p1_3\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h2\")}" }
                    p { "{i18n.t(\"pp_p2\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li {
                            style: "margin-bottom: 8px;",
                            strong { "{i18n.t(\"pp_p2_1_strong\")}" }
                            "{i18n.t(\"pp_p2_1\")}",
                            span { style: "color: white; font-weight: bold;", "{i18n.t(\"pp_p2_1_span\")}" }
                        }
                        li { style: "margin-bottom: 8px;", strong { "{i18n.t(\"pp_p2_2_strong\")}" } "{i18n.t(\"pp_p2_2\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h3\")}" }
                    p { "{i18n.t(\"pp_p3\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p3_1\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p3_2\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p3_3\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h4\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"pp_p4\")}" }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h5\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"pp_p5\")}" }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h6\")}" }
                    p { style: "margin-bottom: 24px;", "{i18n.t(\"pp_p6\")}" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p6_1\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p6_2\")}" }
                        li { style: "margin-bottom: 8px;", "{i18n.t(\"pp_p6_3\")}" }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "{i18n.t(\"pp_h7\")}" }
                    p { "{i18n.t(\"pp_p7\")}" }
                }
            }
        }
    }
}

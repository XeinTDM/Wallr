use dioxus::prelude::*;

const FOOTER_CSS: Asset = asset!("/assets/styling/footer.css");
const LOGO_PNG: Asset = asset!("/assets/logo.png");

#[derive(Props, Clone, PartialEq)]
pub struct FooterProps<R: Routable + Clone + PartialEq + 'static> {
    pub home_route: R,
    pub terms_route: R,
    pub privacy_route: R,
    pub about_route: R,
    pub faq_route: R,
}

#[component]
pub fn Footer<R: Routable + Clone + PartialEq + 'static>(props: FooterProps<R>) -> Element {
    let i18n = crate::i18n::use_i18n();

    rsx! {
        document::Stylesheet { href: FOOTER_CSS }
        footer {
            class: "glass",
            div {
                class: "container footer-content",
                div {
                    class: "footer-left",
                    Link {
                        class: "logo",
                        to: props.home_route.clone(),
                        style: "display: inline-flex; align-items: center; text-decoration: none; margin-bottom: 20px; width: fit-content;",
                        img {
                            src: LOGO_PNG,
                            alt: "Wallr Logo",
                            style: "height: 62px; opacity: 0.9;"
                        }
                    }
                    p { "{i18n.t(\"footer_desc\")}" }
                }
                div {
                    class: "footer-links-container",
                    div {
                        class: "footer-group",
                        h6 { "{i18n.t(\"platform\")}" }
                        Link { to: props.home_route.clone(), "{i18n.t(\"home\")}" }
                        Link { to: props.about_route.clone(), "{i18n.t(\"about\")}" }
                        Link { to: props.faq_route.clone(), "{i18n.t(\"faq\")}" }
                    }
                    div {
                        class: "footer-group",
                        h6 { "{i18n.t(\"legal\")}" }
                        Link { to: props.terms_route.clone(), "{i18n.t(\"terms\")}" }
                        Link { to: props.privacy_route.clone(), "{i18n.t(\"privacy\")}" }
                    }
                    div {
                        class: "footer-group",
                        h6 { "{i18n.t(\"community\")}" }
                        a { href: "https://github.com/XeinTDM/wallr", target: "_blank", "{i18n.t(\"github\")}" }
                        a { href: "https://discord.gg/dioxus", target: "_blank", "{i18n.t(\"discord\")}" }
                    }
                }
                div {
                    class: "footer-bottom",
                    style: "display: flex; justify-content: center; align-items: center; width: 100%;",
                    p {
                        "© 2026 WALLR. {i18n.t(\"built_with\")}"
                        span { style: "color: var(--accent-secondary); font-weight: 600;", "Dioxus" }
                        "{i18n.t(\"and\")}"
                        span { style: "color: #ff4d4d;", "❤️" }
                    }
                }
            }
        }
    }
}

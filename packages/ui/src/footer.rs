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
                    p { "A hyper-optimized, pure-Rust wallpaper engine. AVIF-native, zero-latency, and privacy-focused." }
                }
                div {
                    class: "footer-links-container",
                    div {
                        class: "footer-group",
                        h6 { "Platform" }
                        Link { to: props.home_route.clone(), "Home" }
                        Link { to: props.about_route.clone(), "About" }
                        Link { to: props.faq_route.clone(), "FAQ" }
                    }
                    div {
                        class: "footer-group",
                        h6 { "Legal" }
                        Link { to: props.terms_route.clone(), "Terms" }
                        Link { to: props.privacy_route.clone(), "Privacy" }
                    }
                    div {
                        class: "footer-group",
                        h6 { "Community" }
                        a { href: "https://github.com/XeinTDM/wallr", target: "_blank", "GitHub" }
                        a { href: "https://discord.gg/dioxus", target: "_blank", "Discord" }
                    }
                }
                div {
                    class: "footer-bottom",
                    p {
                        "© 2026 WALLR. Built with "
                        span { style: "color: var(--accent-secondary); font-weight: 600;", "Dioxus" }
                        " and "
                        span { style: "color: #ff4d4d;", "❤️" }
                    }
                }
            }
        }
    }
}

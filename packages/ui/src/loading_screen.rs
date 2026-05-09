use dioxus::prelude::*;
use lucide_dioxus::LoaderCircle;

const LOADING_CSS: Asset = asset!("/assets/styling/loading.css");

#[component]
pub fn LoadingScreen() -> Element {
    rsx! {
        document::Stylesheet { href: LOADING_CSS }
        div {
            class: "loading-screen",
            style: "display: flex; align-items: center; justify-content: center; width: 100%; min-height: 300px; padding: 60px;",
            div {
                class: "spinner-container",
                style: "display: flex; flex-direction: column; align-items: center; gap: 16px;",
                LoaderCircle {
                    size: 40,
                    class: "spinner-icon",
                    style: "color: var(--accent-primary);",
                }
                div {
                    class: "loading-text",
                    style: "font-size: 16px; font-weight: 600; color: var(--text-secondary); display: flex; align-items: center;",
                    "Loading"
                    span { class: "dots" }
                }
            }
        }
    }
}

use crate::app::Route;
use dioxus::prelude::*;

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div {
            class: "not-found-container",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 80vh; text-align: center; padding: 20px;",
            h1 {
                style: "font-size: 6rem; margin-bottom: 20px; color: var(--accent-primary, #0070f3); font-weight: bold;",
                "404"
            }
            h2 {
                style: "font-size: 2rem; margin-bottom: 20px; color: var(--text-primary, #ffffff);",
                "Page Not Found"
            }
            p {
                style: "font-size: 1.2rem; margin-bottom: 30px; color: var(--text-secondary, #a0a0a0);",
                "The page you are looking for doesn't exist or has been moved."
            }
            Link {
                to: Route::Home {},
                style: "padding: 12px 24px; font-size: 1.1rem; border-radius: 8px; text-decoration: none; background-color: var(--accent-primary, #0070f3); color: white; font-weight: 600;",
                "Go to Home"
            }
        }
    }
}

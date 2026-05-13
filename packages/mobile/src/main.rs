use dioxus::prelude::*;
use ui::app::{AuthState, Route};
use ui::{Theme, Toast, ToastContainer};

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(false)); // show_search
    use_context_provider(|| Signal::new(Vec::<Toast>::new())); // toasts

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Theme {}
        ToastContainer {}

        SuspenseBoundary {
            fallback: |_| rsx! {},
            Router::<Route> {}
        }
        }}

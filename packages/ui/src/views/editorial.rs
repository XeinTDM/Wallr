use dioxus::prelude::*;
use crate::CategoryHero;

#[component]
pub fn Editorial() -> Element {
    let category = use_signal(String::new);
    let resolution = use_signal(String::new);
    let sort = use_signal(|| "rating".to_string());
    let aspect_ratio = use_signal(String::new);
    let color = use_signal(String::new);
    let ai_filter = use_signal(String::new);
    let timeframe = use_signal(String::new);

    rsx! {
        CategoryHero {
            home_route: crate::app::Route::Home {},
            title: "Editorial Selections",
            breadcrumb: "Editorial",
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            div {
                style: "text-align: center; padding: 100px 0; color: var(--text-secondary);",
                h2 { "Curated by our team" }
                p { "This section is coming soon. Stay tuned for the best hand-picked wallpapers." }
            }
        }
    }
}

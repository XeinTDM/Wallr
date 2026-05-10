use crate::CategoryHero;
use dioxus::prelude::*;

#[component]
pub fn Editorial() -> Element {
    let i18n = crate::i18n::use_i18n();
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
            title: i18n.t("editorial_title"),
            breadcrumb: i18n.t("editorial_breadcrumb"),
            category,
            resolution,
            sort,
            aspect_ratio,
            color,
            ai_filter,
            timeframe,
            div {
                style: "text-align: center; padding: 100px 0; color: var(--text-secondary);",
                h2 { "{i18n.t(\"editorial_curated\")}" }
                p { "{i18n.t(\"editorial_coming_soon\")}" }
            }
        }
    }
}

use dioxus::prelude::*;

const CATEGORY_HERO_CSS: Asset = asset!("/assets/styling/category_hero.css");

#[derive(Props, Clone, PartialEq)]
pub struct CategoryHeroProps<R: Routable + Clone + PartialEq + 'static> {
    pub title: String,
    pub breadcrumb: String,
    pub home_route: R,
    pub category: Signal<String>,
    pub resolution: Signal<String>,
    pub sort: Signal<String>,
    pub aspect_ratio: Signal<String>,
    pub color: Signal<String>,
    pub ai_filter: Signal<String>,
    pub timeframe: Signal<String>,
    #[props(default)]
    pub license: Option<Signal<String>>,
    #[props(default)]
    pub source: Option<Signal<String>>,
    #[props(default)]
    pub orientation: Option<Signal<String>>,
    #[props(default)]
    pub curated: Option<Signal<bool>>,
    #[props(default)]
    pub tags: Option<Signal<String>>,
    #[props(default)]
    pub excluded_tags: Option<Signal<String>>,
    #[props(default)]
    pub min_width: Option<Signal<String>>,
    #[props(default)]
    pub min_height: Option<Signal<String>>,
    #[props(default)]
    pub author: Option<Signal<String>>,
    pub children: Element,
}

#[component]
pub fn CategoryHero<R: Routable + Clone + PartialEq + 'static>(
    props: CategoryHeroProps<R>,
) -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut category = props.category;
    let mut resolution = props.resolution;
    let mut sort = props.sort;
    let mut aspect_ratio = props.aspect_ratio;
    let mut color = props.color;
    let mut ai_filter = props.ai_filter;
    let mut timeframe = props.timeframe;
    let mut show_advanced_filters = use_signal(|| false);

    let is_showing_advanced = show_advanced_filters();
    let btn_bg = if is_showing_advanced {
        "var(--accent-primary)"
    } else {
        "rgba(255, 255, 255, 0.05)"
    };
    let btn_color = if is_showing_advanced {
        "white"
    } else {
        "var(--text-secondary)"
    };

    rsx! {
        document::Stylesheet { href: CATEGORY_HERO_CSS }
        div {
            div {
                class: "category-hero reveal-up show",
                style: "display: flex; flex-direction: column; gap: 24px; padding: 40px 2rem; margin-top: 40px;",
                div {
                    p {
                        class: "category-breadcrumb",
                        style: "font-size: 14px; color: var(--text-muted); display: flex; align-items: center; gap: 8px; margin-bottom: 12px;",
                        Link {
                            to: props.home_route.clone(),
                            style: "color: var(--text-muted); text-decoration: none; transition: color 0.2s;",
                            "{i18n.t(\"ch_home\")}"
                        }
                        span { class: "mx-2", "/" }
                        span { "{props.breadcrumb}" }
                    }
                    h1 {
                        class: "category-title",
                        style: "font-size: clamp(2rem, 5vw, 3.5rem); font-weight: 900; letter-spacing: -0.04em; color: var(--text-primary); text-transform: capitalize;",
                        "{props.title}"
                    }
                }

                div {
                    class: "category-filter-bar primary-filters",
                    style: "width: 100%; display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-end;",
                    div {
                        class: "filter-group",
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        label {
                            r#for: "filter-category",
                            style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                            "{i18n.t(\"ch_category\")}"
                        }
                        select {
                            id: "filter-category",
                            class: "form-select",
                            style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                            onchange: move |e| category.set(e.value()),
                            value: "{category}",
                            option { value: "", "{i18n.t(\"ch_all\")}" }
                            for (val , label) in api::tags::CATEGORIES.iter() {
                                option { key: "{val}", value: "{val}", "{label}" }
                            }
                        }
                    }
                    div {
                        class: "filter-group",
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        label {
                            r#for: "filter-resolution",
                            style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                            "{i18n.t(\"ch_resolution\")}"
                        }
                        select {
                            id: "filter-resolution",
                            class: "form-select",
                            style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                            onchange: move |e| resolution.set(e.value()),
                            value: "{resolution}",
                            option { value: "", "{i18n.t(\"ch_all\")}" }
                            option { value: "hd", "HD" }
                            option { value: "4k", "4K" }
                            option { value: "8k", "8K" }
                        }
                    }
                    div {
                        class: "filter-group",
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        label {
                            r#for: "filter-sort",
                            style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                            "{i18n.t(\"ch_sort_by\")}"
                        }
                        select {
                            id: "filter-sort",
                            class: "form-select",
                            style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                            onchange: move |e| sort.set(e.value()),
                            value: "{sort}",
                            option { value: "rating", "{i18n.t(\"ch_rating\")}" }
                            option { value: "downloads", "{i18n.t(\"ch_downloads\")}" }
                            option { value: "date", "{i18n.t(\"ch_date\")}" }
                        }
                    }

                    if is_showing_advanced {
                        div {
                            class: "filter-group",
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label {
                                r#for: "filter-aspect",
                                style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                "{i18n.t(\"ch_aspect_ratio\")}"
                            }
                            select {
                                id: "filter-aspect",
                                class: "form-select",
                                style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                onchange: move |e| aspect_ratio.set(e.value()),
                                value: "{aspect_ratio}",
                                option { value: "", "{i18n.t(\"ch_all\")}" }
                                option { value: "ultrawide", "{i18n.t(\"ch_ultrawide\")}" }
                                option { value: "desktop", "{i18n.t(\"ch_desktop\")}" }
                                option { value: "mobile", "{i18n.t(\"ch_mobile\")}" }
                            }
                        }
                        div {
                            class: "filter-group",
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label {
                                r#for: "filter-color",
                                style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                "{i18n.t(\"ch_color\")}"
                            }
                            select {
                                id: "filter-color",
                                class: "form-select",
                                style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                onchange: move |e| color.set(e.value()),
                                value: "{color}",
                                option { value: "", "{i18n.t(\"ch_all\")}" }
                                option { value: "dark", "{i18n.t(\"ch_color_dark\")}" }
                                option { value: "light", "{i18n.t(\"ch_color_light\")}" }
                                option { value: "red", "{i18n.t(\"ch_color_red\")}" }
                                option { value: "blue", "{i18n.t(\"ch_color_blue\")}" }
                                option { value: "green", "{i18n.t(\"ch_color_green\")}" }
                                option { value: "purple", "{i18n.t(\"ch_color_purple\")}" }
                                option { value: "orange", "{i18n.t(\"ch_color_orange\")}" }
                            }
                        }
                        div {
                            class: "filter-group",
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label {
                                r#for: "filter-ai",
                                style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                "{i18n.t(\"ch_ai_filter\")}"
                            }
                            select {
                                id: "filter-ai",
                                class: "form-select",
                                style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                onchange: move |e| ai_filter.set(e.value()),
                                value: "{ai_filter}",
                                option { value: "", "{i18n.t(\"ch_all\")}" }
                                option { value: "hide", "{i18n.t(\"ch_hide_ai\")}" }
                                option { value: "only", "{i18n.t(\"ch_only_ai\")}" }
                            }
                        }
                        div {
                            class: "filter-group",
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label {
                                r#for: "filter-timeframe",
                                style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                "{i18n.t(\"ch_timeframe\")}"
                            }
                            select {
                                id: "filter-timeframe",
                                class: "form-select",
                                style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                onchange: move |e| timeframe.set(e.value()),
                                value: "{timeframe}",
                                option { value: "", "{i18n.t(\"ch_all_time\")}" }
                                option { value: "daily", "{i18n.t(\"ch_past_24_hours\")}" }
                                option { value: "weekly", "{i18n.t(\"ch_past_week\")}" }
                                option { value: "monthly", "{i18n.t(\"ch_past_month\")}" }
                                option { value: "yearly", "{i18n.t(\"ch_past_year\")}" }
                            }
                        }
                        if let Some(mut license_sig) = props.license {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-license",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "License"
                                }
                                select {
                                    id: "filter-license",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                    onchange: move |e| license_sig.set(e.value()),
                                    value: "{license_sig}",
                                    option { value: "", "All Licenses" }
                                    option { value: "standard", "Standard" }
                                    option { value: "creative_commons", "Creative Commons" }
                                    option { value: "public_domain", "Public Domain" }
                                }
                            }
                        }
                        if let Some(mut orientation_sig) = props.orientation {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-orientation",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Orientation"
                                }
                                select {
                                    id: "filter-orientation",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none; cursor: pointer; transition: border-color 0.2s;",
                                    onchange: move |e| orientation_sig.set(e.value()),
                                    value: "{orientation_sig}",
                                    option { value: "", "Any" }
                                    option { value: "landscape", "Landscape" }
                                    option { value: "portrait", "Portrait" }
                                    option { value: "square", "Square" }
                                }
                            }
                        }
                        if let Some(mut source_sig) = props.source {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-source",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Source"
                                }
                                input {
                                    id: "filter-source",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none;",
                                    oninput: move |e| source_sig.set(e.value()),
                                    value: "{source_sig}",
                                    placeholder: "e.g. reddit, pixiv",
                                }
                            }
                        }
                        if let Some(mut tags_sig) = props.tags {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-tags",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Must Include Tags (comma sep)"
                                }
                                input {
                                    id: "filter-tags",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none;",
                                    oninput: move |e| tags_sig.set(e.value()),
                                    value: "{tags_sig}",
                                    placeholder: "e.g. anime, city",
                                }
                            }
                        }
                        if let Some(mut excluded_tags_sig) = props.excluded_tags {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-excluded-tags",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Exclude Tags (comma sep)"
                                }
                                input {
                                    id: "filter-excluded-tags",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 160px; color: var(--text-primary); outline: none;",
                                    oninput: move |e| excluded_tags_sig.set(e.value()),
                                    value: "{excluded_tags_sig}",
                                    placeholder: "e.g. nsfw, dark",
                                }
                            }
                        }
                        if let Some(mut min_width_sig) = props.min_width {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-min-width",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Min Width (px)"
                                }
                                input {
                                    id: "filter-min-width",
                                    r#type: "number",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 100px; max-width: 120px; color: var(--text-primary); outline: none;",
                                    oninput: move |e| min_width_sig.set(e.value()),
                                    value: "{min_width_sig}",
                                }
                            }
                        }
                        if let Some(mut min_height_sig) = props.min_height {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-min-height",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Min Height (px)"
                                }
                                input {
                                    id: "filter-min-height",
                                    r#type: "number",
                                    class: "form-select",
                                    style: "background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-color); border-radius: 10px; padding: 10px 16px; min-width: 100px; max-width: 120px; color: var(--text-primary); outline: none;",
                                    oninput: move |e| min_height_sig.set(e.value()),
                                    value: "{min_height_sig}",
                                }
                            }
                        }
                        if let Some(mut curated_sig) = props.curated {
                            div {
                                class: "filter-group",
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                label {
                                    r#for: "filter-curated",
                                    style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted);",
                                    "Curated Only"
                                }
                                input {
                                    id: "filter-curated",
                                    r#type: "checkbox",
                                    style: "width: 20px; height: 20px; cursor: pointer;",
                                    onchange: move |e| curated_sig.set(e.value() == "true"),
                                    checked: "{curated_sig}",
                                }
                            }
                        }
                    }

                    button {
                        style: "height: 42px; width: 42px; border-radius: 10px; border: 1px solid var(--border-color); margin-left: auto; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: all 0.2s ease; background: {btn_bg}; color: {btn_color};",
                        class: "glow-hover",
                        onclick: move |_| show_advanced_filters.set(!is_showing_advanced),
                        title: "{i18n.t(\"ch_toggle_advanced\")}",
                        lucide_dioxus::SlidersHorizontal { size: 20, color: "currentColor" }
                    }
                }
            }

            {props.children}
        }
    }
}

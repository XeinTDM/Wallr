use crate::LoadingScreen;
use api::get_collections;
use dioxus::prelude::*;

#[component]
pub fn Collections() -> Element {
    let i18n = crate::i18n::use_i18n();
    let collections = use_resource(move || async move { get_collections().await });

    rsx! {
        div {
            class: "container fade-in",
            style: "padding: 120px 0 80px;",

            div {
                class: "section-header",
                style: "margin-bottom: 64px; display: flex; justify-content: space-between; align-items: flex-end;",
                div {
                    h1 {
                        class: "text-gradient",
                        style: "font-size: 48px; font-weight: 900; margin-bottom: 12px;",
                        "{i18n.t(\"collections_title\")}"
                    }
                    p { style: "color: var(--text-secondary);", "{i18n.t(\"collections_desc\")}" }
                }
                button {
                    class: "btn-primary",
                    style: "padding: 12px 24px; border-radius: 12px;",
                    "{i18n.t(\"collections_create_btn\")}"
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 32px;",

                match collections() {
                    Some(Ok(list)) => rsx! {
                        for col in list {
                            CollectionCard {
                                key: "{col.id}",
                                id: col.id,
                                name: col.name,
                                count: col.item_count,
                                cover: col.cover_url
                            }
                        }
                    },
                    Some(Err(e)) => rsx! { div { "{i18n.t(\"error_prefix\")}: {e}" } },
                    None => rsx! { LoadingScreen {} }
                }
            }
        }
    }
}

#[component]
fn CollectionCard(id: String, name: String, count: u32, cover: String) -> Element {
    let i18n = crate::i18n::use_i18n();
    rsx! {
        div {
            class: "glass collection-card glow-hover",
            style: "border-radius: 24px; overflow: hidden; cursor: pointer; transition: all 0.3s ease;",

            div {
                style: "height: 200px; position: relative;",
                img {
                    src: "{crate::resolve_asset_url(&cover)}",
                    style: "width: 100%; height: 100%; object-fit: cover;"
                }
                div {
                    style: "position: absolute; bottom: 16px; right: 16px; background: rgba(0,0,0,0.6); backdrop-filter: blur(8px); padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 700;",
                    "{count} {i18n.t(\"collections_items\")}"
                }
            }

            div {
                style: "padding: 20px;",
                h3 { style: "font-size: 20px; font-weight: 800; margin-bottom: 4px;", "{name}" }
                span { style: "color: var(--text-muted); font-size: 14px;", "{i18n.t(\"collections_shared\")}" }
            }
        }
    }
}

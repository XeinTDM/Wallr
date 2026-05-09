use api::get_wallpapers;
use dioxus::prelude::*;
use ui::Hero;

#[component]
pub fn Home() -> Element {
    let mut page = use_signal(|| 0_u32);
    let mut all_wallpapers = use_signal(Vec::new);
    let mut has_more = use_signal(|| true);

    let _fetch = use_resource(move || async move {
        if !has_more() { return; }
        let p = page();
        if let Ok(new_wps) = get_wallpapers(p, 20, api::FilterOptions::default()).await {
            if new_wps.is_empty() {
                has_more.set(false);
            } else {
                all_wallpapers.with_mut(|w| w.extend_from_slice(new_wps.as_ref()));
            }
        }
    });

    rsx! {
        Hero {}

        div {
            style: "padding-top: 80px; padding-bottom: 80px;",

            div {
                class: "section-header",
                style: "margin-bottom: 1rem; text-align: left; padding: 0 2rem;",
                p {
                    style: "color: #ffc76f; text-transform: uppercase; letter-spacing: .18em; font-size: .74rem; margin-bottom: .5rem;",
                    "Fresh Daily Picks"
                }

                h2 {
                    style: "font-size: clamp(1.8rem, 2.2vw, 2.4rem); font-weight: 500; line-height: 1.2;",
                    "Trending Now"
                }
            }

            ui::WallpaperGrid {
                wallpapers: all_wallpapers,
                is_loading: _fetch().is_none(),
                on_end_reached: move |_| { if has_more() { page += 1 } }
            }
        }
    }
}

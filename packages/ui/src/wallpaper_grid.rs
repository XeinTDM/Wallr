use dioxus::prelude::*;
use api::Wallpaper;
use crate::WallpaperCard;

#[component]
pub fn WallpaperGrid(
    wallpapers: Signal<Vec<Wallpaper>>,
    is_loading: bool,
    #[props(default)] on_end_reached: EventHandler<()>,
    #[props(default)] empty_message: String,
    #[props(default)] empty_submessage: String,
) -> Element {
    let empty_msg = if empty_message.is_empty() { "No wallpapers found." } else { &empty_message };
    let empty_sub = if empty_submessage.is_empty() { "Check back later or upload your own!" } else { &empty_submessage };

    let on_end = on_end_reached;
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(r#"
                let sentinel = document.getElementById('infinite-scroll-sentinel');
                if (!sentinel) return;
                let observer = new IntersectionObserver((entries) => {
                    if (entries[0].isIntersecting) {
                        dioxus.send("end_reached");
                    }
                }, { rootMargin: '400px' });
                observer.observe(sentinel);
            "#);
            
            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "end_reached" {
                    on_end.call(());
                }
            }
        });
    });

    rsx! {
        div {
            class: "wallpaper-grid",
            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 400px), 1fr)); gap: 0.75rem; padding: 0 2rem;",

            if wallpapers.read().is_empty() && !is_loading {
                div {
                    style: "grid-column: 1 / -1; text-align: center; padding: 100px 0; opacity: 0.5;",
                    h3 { "{empty_msg}" }
                    p { "{empty_sub}" }
                }
            } else {
                for wp in wallpapers.read().iter() {
                    WallpaperCard {
                        key: "{wp.id}",
                        wallpaper: wp.clone(),
                    }
                }
                
                if is_loading {
                    for i in 0..4 {
                        div { key: "skeleton-{i}", class: "skeleton glass", style: "height: 240px; border-radius: 20px;" }
                    }
                }
                
                div {
                    id: "infinite-scroll-sentinel",
                    style: "grid-column: 1 / -1; height: 1px; visibility: hidden;",
                }
            }
        }
    }
}

use crate::app::{AuthState, Route};
use crate::{LoadingScreen, WallpaperCard, use_toaster};
use api::{get_wallpaper_by_id, get_wallpapers};
use dioxus::prelude::*;
use lucide_dioxus::{Download, Heart, Plus, Trash2};

#[component]
pub fn WallpaperDetail(id: String) -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let mut is_adding_tag = use_signal(|| false);
    let mut new_tag_input = use_signal(|| String::new());

    let mut toaster = use_toaster();
    let nav = use_navigator();

    let mut is_download_menu_open = use_signal(|| false);
    let mut is_adding_to_col = use_signal(|| false);
    let my_cols = use_resource(move || async move { api::get_my_collections().await });

    let mut current_id = use_signal(|| id.clone());
    if *current_id.peek() != id {
        current_id.set(id.clone());
    }

    let mut is_favorited = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    let mut toaster_clone = toaster.clone();
    #[cfg(target_arch = "wasm32")]
    let mut is_favorited_clone = is_favorited;
    let _shortcuts = use_hook(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_events::EventListener;
            use web_sys::wasm_bindgen::JsCast;

            let window = web_sys::window().unwrap();

            let listener = EventListener::new(&window, "keydown", move |event| {
                let e: &web_sys::KeyboardEvent = event.unchecked_ref();

                if e.ctrl_key() || e.meta_key() || e.alt_key() {
                    return;
                }

                let tag = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.tag_name().to_lowercase());
                if let Some(t) = tag {
                    if t == "input" || t == "textarea" {
                        return;
                    }
                }

                let key_str: String = e.key().into();
                let key_str = key_str.to_lowercase();
                if key_str == "d" {
                    event.prevent_default();
                    if let Some(doc) = web_sys::window().unwrap().document() {
                        let link = doc.create_element("a").unwrap();
                        let url = format!("/wallpaper/{}/download", current_id());
                        link.set_attribute("href", &url).unwrap();
                        link.set_attribute("target", "_blank").unwrap();
                        let _ = link.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
                        toaster_clone.success("Download started!");
                    }
                } else if key_str == "f" || key_str == "l" {
                    event.prevent_default();
                    let current_wp_id = current_id();
                    is_favorited_clone.toggle();
                    spawn(async move {
                        let _ = api::toggle_favorite(current_wp_id).await;
                    });
                } else if key_str == "escape" {
                    event.prevent_default();
                    let _ = web_sys::window().unwrap().history().unwrap().back();
                }
            });
            Some(std::rc::Rc::new(listener))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None::<()>
        }
    });

    let wallpaper = use_resource(move || {
        let current = current_id();
        async move { get_wallpaper_by_id(current).await }
    });

    let _favorite_status = use_resource(move || {
        let current = current_id();
        async move {
            if let Ok(liked) = api::check_is_favorited(current).await {
                is_favorited.set(liked);
            }
        }
    });

    let related = use_resource(move || {
        let current = current_id();
        async move { api::get_similar_wallpapers(current, 4).await }
    });

    rsx! {
        div {
            class: "container fade-in",
            style: "padding: 120px 0 80px;",

            match wallpaper() {
                Some(Ok(Some(wp))) => {
                    let wp_id_for_tag = wp.id.clone();
                    let size_display = if wp.size_bytes < 1048576 {
                        format!("{:.0} KB", wp.size_bytes as f64 / 1024.0)
                    } else {
                        format!("{:.2} MB", wp.size_bytes as f64 / 1048576.0)
                    };
                    let delete_id = wp.id.clone();
                    let is_author = if let AuthState::Authenticated(u) = auth_state() { u.name == wp.author } else { false };
                    let is_admin = if let AuthState::Authenticated(u) = auth_state() { u.role == "admin" } else { false };

                    rsx! {
                        div {
                            class: "detail-grid",
                            style: "display: grid; grid-template-columns: 2fr 1fr; gap: 48px;",

                            // Left: Large Image
                            div {
                                class: "detail-image-container",
                                style: "border-radius: 32px; overflow: hidden; height: fit-content;",
                                if wp.is_live {
                                    video {
                                        src: "{crate::resolve_asset_url(&wp.image_url)}",
                                        poster: "{crate::resolve_asset_url(&wp.thumbnail_url)}",
                                        autoplay: "true",
                                        loop: "true",
                                        muted: "true",
                                        playsinline: "true",
                                        style: "width: 100%; height: auto; display: block; border-radius: 32px;"
                                    }
                                } else {
                                    img {
                                        src: "{crate::resolve_asset_url(&wp.image_url)}",
                                        style: "width: 100%; height: auto; display: block; border-radius: 32px;"
                                    }
                                }
                            }

                            // Right: Metadata & Actions
                            div {
                                class: "detail-info",
                                div {
                                    style: "display: flex; align-items: center; gap: 16px; margin-bottom: 8px;",
                                    h1 { style: "font-size: 40px; font-weight: 900; margin: 0;", "{wp.title}" }
                                    if wp.is_private {
                                        span {
                                            style: "padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 800; background: rgba(139, 92, 246, 0.2); color: #a78bfa; border: 1px solid rgba(139, 92, 246, 0.3); text-transform: uppercase; letter-spacing: 0.05em;",
                                            "Private"
                                        }
                                    }
                                }
                                p {
                                    style: "color: var(--text-secondary); margin-bottom: 32px;",
                                    "by "
                                    Link {
                                        to: Route::PublicProfile { username: wp.author.replace(" ", "-") },
                                        style: "color: var(--accent-primary); text-decoration: none;",
                                        "{wp.author}"
                                    }
                                }

                                div {
                                    class: "glass",
                                    style: "padding: 24px; border-radius: 20px; margin-bottom: 24px;",
                                    h4 { style: "margin-bottom: 16px; font-size: 14px; color: var(--text-muted);", "TECHNICAL DETAILS" }
                                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                                        DetailItem { label: "Resolution", value: "{wp.dimensions.0}x{wp.dimensions.1}" }
                                        DetailItem { label: "Format", value: if wp.is_live { "Video" } else if wp.image_url.ends_with(".avif") { "AVIF" } else { "Processing..." } }
                                        DetailItem { label: "Size", value: "{size_display}" }
                                        DetailItem { label: "Likes", value: "{wp.likes}" }
                                        DetailItem { label: "Downloads", value: "{wp.downloads}" }
                                    }
                            }

                            div {
                                style: "display: flex; gap: 12px; margin-bottom: 24px;",
                                for color in wp.primary_colors.clone() {
                                    div {
                                        key: "{color}",
                                        class: "glow-hover",
                                        style: "width: 36px; height: 36px; border-radius: 10px; border: 2px solid rgba(255,255,255,0.1); background-color: {color}; box-shadow: 0 4px 16px {color}40; cursor: pointer; transition: transform 0.2s;",
                                        title: "{color}",
                                        onclick: move |_| toaster.info(format!("Copied color {}", color))
                                    }
                                }
                            }

                            div {
                                style: "display: flex; gap: 12px;",
                                if wp.is_live || wp.image_url.ends_with(".avif") {
                                    {
                                        rsx! {
                                            div {
                                                style: "position: relative; flex: 1; display: flex;",
                                                button {
                                                    class: "glow-hover",
                                                    style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; text-decoration: none; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.1); color: var(--text-primary); transition: all 0.2s ease; cursor: pointer;",
                                                    onclick: move |_| is_download_menu_open.toggle(),
                                                    Download { size: 20 }
                                                    "Download Options"
                                                }
                                                if is_download_menu_open() {
                                                    div {
                                                        style: "position: absolute; top: 70px; left: 0; right: 0; background: var(--bg-secondary); border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 12px; z-index: 100; box-shadow: 0 10px 30px rgba(0,0,0,0.5);",
                                                        div {
                                                            style: "display: flex; flex-direction: column; gap: 8px;",
                                                            a {
                                                                href: "/wallpaper/{wp.id}/download",
                                                                download: "{wp.title}",
                                                                target: "_blank",
                                                                style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                class: "menu-item-hover",
                                                                onclick: { let mut toaster = toaster; move |_| { is_download_menu_open.set(false); toaster.success("Download started!"); } },
                                                                span { "Original Size" }
                                                                span { style: "color: var(--text-muted); font-size: 12px;", "AVIF" }
                                                            }
                                                            a {
                                                                href: "/wallpaper/{wp.id}/download?width=3840&format=jpg",
                                                                download: "{wp.title}",
                                                                target: "_blank",
                                                                style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                class: "menu-item-hover",
                                                                onclick: { let mut toaster = toaster; move |_| { is_download_menu_open.set(false); toaster.success("Download started!"); } },
                                                                span { "4K UHD (3840w)" }
                                                                span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                            }
                                                            a {
                                                                href: "/wallpaper/{wp.id}/download?width=2560&format=jpg",
                                                                download: "{wp.title}",
                                                                target: "_blank",
                                                                style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                class: "menu-item-hover",
                                                                onclick: { let mut toaster = toaster; move |_| { is_download_menu_open.set(false); toaster.success("Download started!"); } },
                                                                span { "1440p (2560w)" }
                                                                span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                            }
                                                            a {
                                                                href: "/wallpaper/{wp.id}/download?width=1920&format=jpg",
                                                                download: "{wp.title}",
                                                                target: "_blank",
                                                                style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                class: "menu-item-hover",
                                                                onclick: { let mut toaster = toaster; move |_| { is_download_menu_open.set(false); toaster.success("Download started!"); } },
                                                                span { "1080p (1920w)" }
                                                                span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    button {
                                        disabled: true,
                                        style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; background: rgba(255, 255, 255, 0.02); border: 1px dashed rgba(255, 255, 255, 0.1); color: var(--text-muted); cursor: not-allowed;",
                                        "Processing AVIF..."
                                    }
                                }
                                button {
                                    class: "glass glow-hover",
                                    style: format!("width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: {}; cursor: pointer; transition: all 0.2s ease; color: {};", if is_favorited() { "rgba(239, 68, 68, 0.1)" } else { "rgba(255,255,255,0.05)" }, if is_favorited() { "#ef4444" } else { "white" }),
                                    onclick: move |_| {
                                        let current_wp_id = wp.id.clone();
                                        let _current_is_fav = is_favorited();
                                        is_favorited.toggle();
                                        spawn(async move {
                                            let _ = api::toggle_favorite(current_wp_id).await;
                                        });
                                    },
                                    Heart { size: 24, fill: if is_favorited() { "currentColor" } else { "none" } }
                                }
                                {
                                    rsx! {
                                        div {
                                            style: "position: relative;",
                                            button {
                                                class: "glass glow-hover",
                                                style: "width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); cursor: pointer; transition: all 0.2s ease; color: white;",
                                                onclick: move |_| is_adding_to_col.toggle(),
                                                lucide_dioxus::FolderPlus { size: 24 }
                                            }
                                            if is_adding_to_col() {
                                                div {
                                                    style: "position: absolute; bottom: 70px; right: 0; background: var(--bg-secondary); border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 12px; min-width: 200px; z-index: 100;",
                                                    match my_cols() {
                                                        Some(Ok(list)) => {
                                                            if list.is_empty() {
                                                                rsx! { div { style: "color: var(--text-muted); font-size: 14px;", "No collections found." } }
                                                            } else {
                                                                rsx! {
                                                                    div {
                                                                        style: "display: flex; flex-direction: column; gap: 8px;",
                                                                        for col in list.clone() {
                                                                            button {
                                                                                key: "{col.id}",
                                                                                style: "background: none; border: none; color: white; text-align: left; padding: 8px; border-radius: 6px; cursor: pointer;",
                                                                                class: "menu-item-hover",
                                                                                onclick: {
                                                                                    let w_id = wp.id.clone();
                                                                                    let c_id = col.id.clone();
                                                                                    let toaster = toaster;
                                                                                    move |_| {
                                                                                        is_adding_to_col.set(false);
                                                                                        let w_id = w_id.clone();
                                                                                        let c_id = c_id.clone();
                                                                                        let mut toaster = toaster;
                                                                                        spawn(async move {
                                                                                            if let Ok(_) = api::add_wallpaper_to_collection(c_id, w_id).await {
                                                                                                toaster.success("Added to collection!");
                                                                                            } else {
                                                                                                toaster.error("Failed to add to collection.");
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                },
                                                                                "{col.name}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        _ => rsx! { div { "Loading..." } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if is_author || is_admin {
                                    button {
                                        class: "glass glow-hover",
                                        style: "width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: rgba(239, 68, 68, 0.1); cursor: pointer; transition: all 0.2s ease; color: #ef4444;",
                                        title: if is_admin { "Admin: Delete Wallpaper" } else { "Delete Wallpaper" },
                                        onclick: move |_| {
                                            let id_to_delete = delete_id.clone();
                                            let wp_title = wp.title.clone();
                                            let mut toaster = toaster;
                                            let nav = nav;
                                            let is_admin_clone = is_admin;
                                            let is_author_clone = is_author;
                                            spawn(async move {
                                                #[allow(unused_mut)]
                                                let mut reason = None;
                                                if is_admin_clone && !is_author_clone {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if let Some(window) = web_sys::window() {
                                                        if let Ok(Some(r)) = window.prompt_with_message("Admin Deletion: Enter reason (optional):") {
                                                            reason = Some(r);
                                                        }
                                                    }
                                                }

                                                let result = if is_admin_clone && !is_author_clone {
                                                    api::admin_delete_wallpaper(id_to_delete.clone(), reason.clone()).await
                                                } else {
                                                    api::delete_wallpaper_endpoint(id_to_delete.clone()).await
                                                };

                                                if let Ok(_) = result {
                                                    if is_admin_clone && !is_author_clone {
                                                        toaster.info(format!("Admin: Deleted '{}'. Reason: {}", wp_title, reason.unwrap_or_else(|| "None provided".into())));
                                                    } else {
                                                        toaster.success("Wallpaper deleted.");
                                                    }
                                                    nav.push(Route::Home {});
                                                } else {
                                                    toaster.error("Failed to delete wallpaper");
                                                }
                                            });
                                        },
                                        Trash2 { size: 24 }
                                    }
                                }
                            }

                            div {
                                class: "card-tags",
                                style: "margin-top: 24px; display: flex; flex-wrap: wrap; gap: 10px; align-items: center;",
                                for tag in wp.tags.clone() {
                                    Tag { key: "{tag}", tag }
                                }
                                if let AuthState::Authenticated(u) = auth_state() {
                                    if u.name == wp.author {
                                        if is_adding_tag() {
                                            form {
                                                onsubmit: move |e| {
                                                    e.stop_propagation();
                                                    let tag = new_tag_input().trim().to_lowercase();
                                                    if !tag.is_empty() {
                                                        let wp_id = wp_id_for_tag.clone();
                                                        let mut wp_res = wallpaper;
                                                        spawn(async move {
                                                            if let Ok(_) = api::add_tag_to_wallpaper(wp_id, tag).await {
                                                                wp_res.restart();
                                                            }
                                                        });
                                                    }
                                                    is_adding_tag.set(false);
                                                    new_tag_input.set(String::new());
                                                },
                                                input {
                                                    type: "text",
                                                    value: "{new_tag_input}",
                                                    oninput: move |e| new_tag_input.set(e.value()),
                                                    placeholder: "New tag...",
                                                    style: "padding: 6px 12px; border-radius: 20px; font-size: 13px; font-weight: 700; background: rgba(0, 0, 0, 0.2); border: 1px solid rgba(255, 255, 255, 0.4); color: white; outline: none; width: 100px;",
                                                    autofocus: "true"
                                                }
                                            }
                                        } else {
                                            span {
                                                class: "glass tag glow-hover",
                                                onclick: move |_| is_adding_tag.set(true),
                                                style: "cursor: pointer; padding: 6px 12px; border-radius: 20px; font-size: 13px; font-weight: 700; background: rgba(255, 255, 255, 0.1); border: 1px dashed rgba(255, 255, 255, 0.4); transition: all 0.2s ease; display: inline-flex; align-items: center; justify-content: center; color: white;",
                                                Plus { size: 14 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    CommentsSection { wallpaper_id: wp.id.clone(), is_wallpaper_author: is_author }

                    // Related Wallpapers
                    div {
                        style: "margin-top: 80px;",
                        h2 { style: "margin-bottom: 32px;", "More like this" }
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px;",
                            match related() {
                                Some(Ok(list)) => rsx! {
                                    for related_wp in list.iter().filter(|w| w.id != wp.id).take(3) {
                                        WallpaperCard {
                                            key: "{related_wp.id}",
                                            wallpaper: related_wp.clone(),
                                        }
                                    }
                                },
                                _ => rsx! { LoadingScreen {} }
                            }
                        }
                    }
                } },
                Some(Ok(None)) => rsx! {
                    div { "Wallpaper not found." }
                },
                Some(Err(e)) => rsx! {
                    div { class: "error", "Error: {e}" }
                },
                None => rsx! {
                    LoadingScreen {}
                }
            }
        }
    }
}

#[component]
fn Tag(tag: String) -> Element {
    let nav = use_navigator();
    rsx! {
        span {
            class: "glass tag glow-hover",
            onclick: move |_| { nav.push(Route::Search { query: tag.clone() }); },
            style: "cursor: pointer; padding: 6px 16px; border-radius: 20px; font-size: 13px; font-weight: 700; background: rgba(255, 255, 255, 0.1); border: 1px solid rgba(255, 255, 255, 0.2); transition: all 0.2s ease; display: inline-block; color: white;",
            "#{tag}"
        }
    }
}

#[component]
fn DetailItem(label: String, value: String) -> Element {
    rsx! {
        div {
            span { style: "display: block; font-size: 12px; color: var(--text-muted);", "{label}" }
            span { style: "font-weight: 700; font-size: 15px;", "{value}" }
        }
    }
}

#[component]
fn CommentsSection(wallpaper_id: String, is_wallpaper_author: bool) -> Element {
    let mut page = use_signal(|| 0u32);
    let mut all_comments = use_signal(|| Vec::<api::WallpaperComment>::new());
    let mut has_more = use_signal(|| true);

    let wid_for_res = wallpaper_id.clone();
    let comments_res = use_resource(move || {
        let wid = wid_for_res.clone();
        let p = page();
        async move { api::get_wallpaper_comments(wid, p, 50).await }
    });

    use_effect(move || {
        if let Some(Ok(new_comments)) = comments_res() {
            if new_comments.len() < 50 {
                has_more.set(false);
            } else {
                has_more.set(true);
            }
            if page() == 0 {
                all_comments.set(new_comments);
            } else {
                all_comments.with_mut(|c| {
                    for nc in new_comments {
                        if !c.iter().any(|existing| existing.id == nc.id) {
                            c.push(nc);
                        }
                    }
                });
            }
        }
    });

    let mut new_comment = use_signal(String::new);
    let is_auth = use_context::<Signal<crate::app::AuthState>>();
    let toaster = use_toaster();

    rsx! {
        div {
            style: "margin-top: 80px;",
            h2 { style: "margin-bottom: 32px;", "Comments" }

            if let crate::app::AuthState::Authenticated(u) = is_auth() {
                div {
                    style: "display: flex; gap: 16px; margin-bottom: 32px;",
                    img { src: "{crate::resolve_asset_url(&u.pfp_url)}", style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover;" }
                    div {
                        style: "flex: 1;",
                        textarea {
                            class: "glass",
                            style: "width: 100%; box-sizing: border-box; min-height: 80px; padding: 12px; border-radius: 12px; color: white; resize: vertical; border: 1px solid rgba(255,255,255,0.1);",
                            placeholder: "Add a comment...",
                            value: "{new_comment}",
                            oninput: move |e| new_comment.set(e.value()),
                            maxlength: "500"
                        }
                        div {
                            style: "display: flex; justify-content: flex-end; margin-top: 8px;",
                            span {
                                style: "color: var(--text-muted); font-size: 12px; margin-right: 16px; align-self: center;",
                                "{new_comment().chars().count()}/500"
                            }
                            button {
                                class: "glow-hover",
                                style: "padding: 8px 24px; border-radius: 8px; background: var(--accent-primary); color: white; font-weight: 600; border: none; cursor: pointer;",
                                disabled: new_comment().trim().is_empty(),
                                onclick: move |_| {
                                    if !new_comment().trim().is_empty() {
                                        let mut toaster = toaster;
                                        let text = new_comment().clone();
                                        let wid = wallpaper_id.clone();
                                        #[allow(unused_mut)]
                                        let mut res = comments_res;
                                        #[allow(unused_mut)]
                                        let mut p = page;
                                        let mut text_sig = new_comment;
                                        spawn(async move {
                                            match api::add_wallpaper_comment(wid, text).await {
                                                Ok(_) => {
                                                    text_sig.set(String::new());
                                                    p.set(0);
                                                    res.restart();
                                                    toaster.success("Comment added");
                                                }
                                                Err(e) => {
                                                    toaster.error(&e.to_string());
                                                }
                                            }
                                        });
                                    }
                                },
                                "Post Comment"
                            }
                        }
                    }
                }
            } else {
                div {
                    style: "margin-bottom: 32px; padding: 24px; background: rgba(255,255,255,0.02); border-radius: 12px; text-align: center; border: 1px dashed rgba(255,255,255,0.1);",
                    p { style: "color: var(--text-secondary); margin-bottom: 12px;", "Log in to join the conversation." }
                    Link {
                        to: Route::Login {},
                        class: "glow-hover",
                        style: "padding: 8px 24px; border-radius: 8px; background: rgba(255,255,255,0.1); color: white; text-decoration: none; display: inline-block; font-weight: 600;",
                        "Log In"
                    }
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 24px;",
                if all_comments().is_empty() {
                    if let Some(Ok(_)) = comments_res() {
                        p { style: "color: var(--text-muted); text-align: center; padding: 24px;", "No comments yet. Be the first to share your thoughts!" }
                    } else if let Some(Err(_)) = comments_res() {
                        p { style: "color: #ef4444;", "Failed to load comments." }
                    } else {
                        div { "Loading comments..." }
                    }
                } else {
                    for comment in all_comments() {
                        div {
                            key: "{comment.id}",
                            style: "display: flex; gap: 16px;",
                            img { src: "{crate::resolve_asset_url(&comment.user_pfp)}", style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover;" }
                            div {
                                div {
                                    style: "display: flex; align-items: baseline; gap: 8px; margin-bottom: 4px;",
                                    Link {
                                        to: Route::PublicProfile { username: comment.user_name.replace(" ", "-") },
                                        style: "font-weight: 700; font-size: 15px; color: white; text-decoration: none;",
                                        class: "glow-hover-text",
                                        "{comment.user_name}"
                                    }
                                    span { style: "font-size: 12px; color: var(--text-muted);", "{comment.created_at.split('T').next().unwrap_or(&comment.created_at)}" }

                                    if let crate::app::AuthState::Authenticated(u) = is_auth() {
                                        if u.id == comment.user_id || is_wallpaper_author {
                                            button {
                                                class: "glow-hover",
                                                style: "background: none; border: none; padding: 4px; margin-left: auto; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                                                onclick: {
                                                    let c_id = comment.id.clone();
                                                    #[allow(unused_mut)]
                                                    let mut res = comments_res;
                                                    #[allow(unused_mut)]
                                                    let mut p = page;
                                                    let toaster = toaster;
                                                    move |_| {
                                                        let c_id = c_id.clone();
                                                        let mut res = res;
                                                        let mut p = p;
                                                        let mut toaster = toaster;
                                                        spawn(async move {
                                                            if let Ok(_) = api::delete_wallpaper_comment(c_id).await {
                                                                p.set(0);
                                                                res.restart();
                                                                toaster.success("Comment deleted");
                                                            } else {
                                                                toaster.error("Failed to delete comment");
                                                            }
                                                        });
                                                    }
                                                },
                                                lucide_dioxus::Trash2 { size: 14, color: "var(--text-muted)" }
                                            }
                                        }
                                    }
                                }
                                p { style: "color: var(--text-secondary); line-height: 1.5; font-size: 14px; white-space: pre-wrap;", "{comment.content}" }
                            }
                        }
                    }

                    if has_more() {
                        div {
                            style: "display: flex; justify-content: center; margin-top: 16px;",
                            button {
                                class: "glass glow-hover",
                                style: "padding: 12px 24px; border-radius: 12px; border: 1px dashed rgba(255,255,255,0.2); color: white; cursor: pointer; background: rgba(255,255,255,0.05); font-weight: 600;",
                                onclick: move |_| {
                                    page.set(page() + 1);
                                },
                                "Load More"
                            }
                        }
                    }
                }
            }
        }
    }
}

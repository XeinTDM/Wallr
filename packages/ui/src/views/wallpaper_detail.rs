use crate::app::{AuthState, Route};
use crate::{LoadingScreen, WallpaperCard, use_toaster};
use api::get_wallpaper_by_id;
use dioxus::prelude::*;
use lucide_dioxus::{Download, Heart, Plus, Shield, Trash2};

#[cfg(feature = "desktop")]
async fn set_desktop_wallpaper(wp_id: String, image_url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let filename = image_url
            .strip_prefix("/assets/uploads/")
            .unwrap_or(&image_url);
        let full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/uploads")
            .join(filename);
        let img = image::open(&full_path).map_err(|e| e.to_string())?;
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("wallr_{}.jpg", wp_id));
        img.save(&temp_path).map_err(|e| e.to_string())?;

        wallpaper::set_from_path(temp_path.to_str().unwrap()).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .unwrap_or(Err("Task failed".to_string()))
}

#[cfg(feature = "desktop")]
async fn save_as_native(wp_title: String, image_url: String) -> Result<(), String> {
    let filename = image_url
        .strip_prefix("/assets/uploads/")
        .unwrap_or(&image_url);
    let full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/uploads")
        .join(filename);

    let default_name = format!("{}.avif", wp_title.replace(" ", "_"));

    let target_path = tokio::task::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title("Save Wallpaper As")
            .set_file_name(&default_name)
            .add_filter("AVIF Image", &["avif"])
            .add_filter("All Files", &["*"])
            .save_file()
    })
    .await
    .unwrap_or(None);

    if let Some(path) = target_path {
        tokio::fs::copy(&full_path, &path)
            .await
            .map_err(|e| e.to_string())?;

        let parent = path.parent().map(|p| p.to_path_buf());
        if let Some(dir) = parent {
            let _ = tokio::task::spawn_blocking(move || {
                let _ = open::that(dir);
            })
            .await;
        }

        Ok(())
    } else {
        Err("Cancelled".to_string())
    }
}

#[component]
pub fn WallpaperDetail(id: String) -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let mut is_adding_tag = use_signal(|| false);
    let mut new_tag_input = use_signal(String::new);

    let mut toaster = use_toaster();
    let i18n = crate::i18n::use_i18n();
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
                        toaster_clone.success(i18n.t("success_download_started"));
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

    let wallpaper = use_server_future(move || {
        let current = current_id();
        async move { get_wallpaper_by_id(current).await }
    })?;

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
        div { class: "container", style: "padding: 120px 0 80px;",

            match wallpaper() {
                Some(Ok(Some(wp))) => {
                    let wp_id_for_tag = wp.id.clone();
                    let size_display = if wp.size_bytes < 1048576 {
                        format!("{:.0} KB", wp.size_bytes as f64 / 1024.0)
                    } else {
                        format!("{:.2} MB", wp.size_bytes as f64 / 1048576.0)
                    };
                    let delete_id = wp.id.clone();
                    let delete_id_clone = delete_id.clone();
                    let is_author = if let AuthState::Authenticated(u) = auth_state() {
                        u.id == wp.author_id
                    } else {
                        false
                    };
                    let is_admin = if let AuthState::Authenticated(u) = auth_state() {
                        u.role == "admin"
                    } else {
                        false
                    };
                    let absolute_img_url = format!(
                        "https://wallr.app{}",
                        crate::resolve_asset_url(&wp.thumbnail_url),
                    );
                    let title_text = format!("{} by {} - Wallr", wp.title, wp.author_name);
                    let desc_text = format!(
                        "View this high-quality wallpaper uploaded by {} on Wallr.",
                        wp.author_name,
                    );
                    rsx! {
                        document::Title { "{title_text}" }
                        document::Meta { property: "og:title", content: "{title_text}" }
                        document::Meta { property: "og:description", content: "{desc_text}" }
                        document::Meta { property: "og:image", content: "{absolute_img_url}" }
                        document::Meta { property: "og:type", content: "website" }
                        document::Meta { name: "twitter:card", content: "summary_large_image" }
                        document::Meta { name: "twitter:title", content: "{title_text}" }
                        document::Meta { name: "twitter:description", content: "{desc_text}" }
                        document::Meta { name: "twitter:image", content: "{absolute_img_url}" }

            

                        div {
            

                            class: "detail-grid",
                            style: "display: grid; grid-template-columns: 2fr 1fr; gap: 48px;",
            
                            div {
                                class: "detail-image-container",
                                style: "border-radius: 32px; overflow: hidden; height: fit-content;",
                                if wp.is_live {
                                    video {
                                        src: "{crate::resolve_asset_url(&wp.image_url)}",
                                        poster: "{crate::resolve_asset_url(&wp.thumbnail_url)}",
                                        autoplay: "true",
                                        r#loop: "true",
                                        muted: "true",
                                        playsinline: "true",
                                        style: "width: 100%; height: auto; display: block; border-radius: 32px;",
                                    }
                                } else {
                                    img {
                                        src: "{crate::resolve_asset_url(&wp.image_url)}",
                                        style: "width: 100%; height: auto; display: block; border-radius: 32px;",
                                    }
                                }
                            }
            
                            div { class: "detail-info",
                                div { style: "display: flex; align-items: center; gap: 16px; margin-bottom: 8px;",
                                    h1 { style: "font-size: 40px; font-weight: 900; margin: 0;", "{wp.title}" }
                                    if wp.is_private {
                                        span { style: "padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 800; background: rgba(139, 92, 246, 0.2); color: #a78bfa; border: 1px solid rgba(139, 92, 246, 0.3); text-transform: uppercase; letter-spacing: 0.05em;",
                                            "{i18n.t(\"wp_private\")}"
                                        }
                                    }
                                }
                                p { style: "color: var(--text-secondary); margin-bottom: 32px;",
                                    "{i18n.t(\"wp_by\")}"
                                    Link {
                                        to: Route::PublicProfile {
                                            username: wp.author_name.replace(" ", "-"),
                                        },
                                        style: "color: var(--accent-primary); text-decoration: none;",
                                        "{wp.author_name}"
                                    }
                                }
            
                                if let Some(desc) = wp.description.clone() {
                                    p { style: "color: var(--text-primary); margin-bottom: 24px; line-height: 1.6; white-space: pre-wrap;",
                                        "{desc}"
                                    }
                                }
            
                                if let Some(url) = wp.source_url.clone() {
                                    a {
                                        href: "{url}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        style: "color: var(--accent-primary); display: inline-flex; align-items: center; gap: 8px; margin-bottom: 24px; text-decoration: none; font-weight: 600;",
                                        lucide_dioxus::ExternalLink { size: 16 }
                                        "Original Source"
                                    }
                                }
            
                                div {
                                    class: "glass",
                                    style: "padding: 24px; border-radius: 20px; margin-bottom: 24px;",
                                    h4 { style: "margin-bottom: 16px; font-size: 14px; color: var(--text-muted);",
                                        "{i18n.t(\"wp_technical_details\")}"
                                    }
                                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                                        DetailItem {
                                            label: "{i18n.t(\"wp_resolution\")}",
                                            value: "{wp.dimensions.0}x{wp.dimensions.1}",
                                        }
                                        DetailItem {
                                            label: "{i18n.t(\"wp_format\")}",
                                            value: if wp.is_live { "{i18n.t(\"wp_format_video\")}" } else if wp.image_url.ends_with(".avif") { "{i18n.t(\"wp_format_avif\")}" } else { "{i18n.t(\"wp_format_processing\")}" },
                                        }
                                        DetailItem { label: "{i18n.t(\"wp_size\")}", value: "{size_display}" }
                                        DetailItem { label: "{i18n.t(\"wp_likes\")}", value: "{wp.likes}" }
                                        DetailItem {
                                            label: "{i18n.t(\"wp_downloads\")}",
                                            value: "{wp.downloads}",
                                        }
                                    }
                                }
            
                                div { style: "display: flex; gap: 12px; margin-bottom: 24px;",
                                    for color in wp.primary_colors.clone() {
                                        div {
                                            key: "{color}",
                                            class: "glow-hover",
                                            style: "width: 36px; height: 36px; border-radius: 10px; border: 2px solid rgba(255,255,255,0.1); background-color: {color}; box-shadow: 0 4px 16px {color}40; cursor: pointer; transition: transform 0.2s;",
                                            title: "{color}",
                                            onclick: move |_| toaster.info(format!("Copied color {}", color)),
                                        }
                                    }
                                }
            
                                div { style: "display: flex; gap: 12px;",
                                    if wp.is_live || wp.image_url.ends_with(".avif") {
                                        {
                                            rsx! {
                                                div { style: "position: relative; flex: 1; display: flex;",
                                                    button {
                                                        class: "glow-hover",
                                                        style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; text-decoration: none; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.1); color: var(--text-primary); transition: all 0.2s ease; cursor: pointer;",
                                                        onclick: move |_| is_download_menu_open.toggle(),
                                                        Download { size: 20 }
                                                        "{i18n.t(\"wp_download_options\")}"
                                                    }
                                                    if is_download_menu_open() {
                                                        div { style: "position: absolute; top: 70px; left: 0; right: 0; background: var(--bg-secondary); border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 12px; z-index: 100; box-shadow: 0 10px 30px rgba(0,0,0,0.5);",
                                                            div { style: "display: flex; flex-direction: column; gap: 8px;",
                                                                {
                                                                    let (pref_quality, pref_format) = if let crate::app::AuthState::Authenticated(ref u) = auth_state() {
                                                                        (u.download_quality.clone(), if u.auto_download_avif { "AVIF" } else { "JPG" })
                                                                    } else {
                                                                        (i18n.t("wp_original_size").to_string(), "AVIF")
                                                                    };
                                                                    rsx! {
                                                                        a {
                                                                            href: "/wallpaper/{wp.id}/download",
                                                                            download: "{wp.title}",
                                                                            target: "_blank",
                                                                            style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                            class: "menu-item-hover",
                                                                            onclick: {
                                                                                let mut toaster = toaster;
                                                                                move |_| {
                                                                                    is_download_menu_open.set(false);
                                                                                    toaster.success(i18n.t("success_download_started"));
                                                                                }
                                                                            },
                                                                            span { "{pref_quality} (Default)" }
                                                                            span { style: "color: var(--text-muted); font-size: 12px;", "{pref_format}" }
                                                                        }
                                                                    }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=3840&format=avif",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_original_size\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "AVIF" }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=3840&format=jpg",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_4k_uhd\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=3440&height=1440&format=jpg",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_ultrawide\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=3024&height=1964&format=jpg",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_macbook_pro\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=1920&format=jpg",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_1080p\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                                }
                                                                a {
                                                                    href: "/wallpaper/{wp.id}/download?width=1179&height=2556&format=jpg",
                                                                    download: "{wp.title}",
                                                                    target: "_blank",
                                                                    style: "background: none; border: none; color: white; text-align: left; padding: 12px; border-radius: 6px; cursor: pointer; text-decoration: none; display: flex; justify-content: space-between;",
                                                                    class: "menu-item-hover",
                                                                    onclick: {
                                                                        let mut toaster = toaster;
                                                                        move |_| {
                                                                            is_download_menu_open.set(false);
                                                                            toaster.success(i18n.t("success_download_started"));
                                                                        }
                                                                    },
                                                                    span { "{i18n.t(\"wp_iphone_15_pro\")}" }
                                                                    span { style: "color: var(--text-muted); font-size: 12px;", "JPG" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                {
                                                    #[cfg(feature = "desktop")]
                                                    rsx! {
                                                        button {
                                                            class: "glow-hover",
                                                            style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.1); color: var(--text-primary); transition: all 0.2s ease; cursor: pointer;",
                                                            onclick: {
                                                                let wp_title = wp.title.clone();
                                                                let image_url = wp.image_url.clone();
                                                                let toaster = toaster;
                                                                move |_| {
                                                                    let wp_title = wp_title.clone();
                                                                    let image_url = image_url.clone();
                                                                    let mut toaster = toaster;
                                                                    spawn(async move {
                                                                        match save_as_native(wp_title, image_url).await {
                                                                            Ok(_) => toaster.success("Saved successfully!"),
                                                                            Err(e) if e == "Cancelled" => {}
                                                                            Err(e) => toaster.error(format!("Failed: {}", e)),
                                                                        }
                                                                    });
                                                                }
                                                            },
                                                            lucide_dioxus::Save { size: 20 }
                                                            "Save As..."
                                                        }
                                                    }
                                                }
                                                {
                                                    #[cfg(feature = "desktop")]
                                                    rsx! {
                                                        button {
                                                            class: "glow-hover",
                                                            style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; background: var(--accent-primary); border: none; color: white; transition: all 0.2s ease; cursor: pointer;",
                                                            onclick: {
                                                                let wp_id = wp.id.clone();
                                                                let image_url = wp.image_url.clone();
                                                                let toaster = toaster;
                                                                move |_| {
                                                                    let wp_id = wp_id.clone();
                                                                    let image_url = image_url.clone();
                                                                    let mut toaster = toaster;
                                                                    spawn(async move {
                                                                        toaster.info("Applying wallpaper...");
                                                                        match set_desktop_wallpaper(wp_id, image_url).await {
                                                                            Ok(_) => toaster.success("Desktop background updated!"),
                                                                            Err(e) => toaster.error(format!("Failed: {}", e)),
                                                                        }
                                                                    });
                                                                }
                                                            },
                                                            lucide_dioxus::Monitor { size: 20 }
                                                            "Set Background"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        button {
                                            disabled: true,
                                            style: "flex: 1; padding: 16px; border-radius: 16px; font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px; background: rgba(255, 255, 255, 0.02); border: 1px dashed rgba(255, 255, 255, 0.1); color: var(--text-muted); cursor: not-allowed;",
                                            "{i18n.t(\"wp_processing_avif\")}"
                                        }
                                    }
                                    button {
                                        class: "glass glow-hover",
                                        style: format!(
                                            "width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: {}; cursor: pointer; transition: all 0.2s ease; color: {};",
                                            if is_favorited() { "rgba(239, 68, 68, 0.1)" } else { "rgba(255,255,255,0.05)" },
                                            if is_favorited() { "#ef4444" } else { "white" },
                                        ),
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
                                            div { style: "position: relative;",
                                                button {
                                                    class: "glass glow-hover",
                                                    style: "width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); cursor: pointer; transition: all 0.2s ease; color: white;",
                                                    onclick: move |_| is_adding_to_col.toggle(),
                                                    lucide_dioxus::FolderPlus { size: 24 }
                                                }
                                                if is_adding_to_col() {
                                                    div { style: "position: absolute; bottom: 70px; right: 0; background: var(--bg-secondary); border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 12px; min-width: 200px; z-index: 100;",
                                                        match my_cols() {
                                                            Some(Ok(list)) => {
                                                                if list.is_empty() {
                                                                    rsx! {
                                                                        div { style: "color: var(--text-muted); font-size: 14px;", "{i18n.t(\"wp_no_collections\")}" }
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div { style: "display: flex; flex-direction: column; gap: 8px;",
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
                                                                                                    toaster.success(i18n.t("success_added_collection"));
                                                                                                } else {
                                                                                                    toaster.error(i18n.t("err_add_collection"));
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
                                                            }
                                                            _ => rsx! {
                                                                div { "{i18n.t(\"loading\")}" }
                                                            },
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
                                                            if let Ok(Some(r)) = window
            
                                                                .prompt_with_message("Admin Deletion: Enter reason (optional):")
                                                            {
                                                                reason = Some(r);
                                                            }
                                                        }
                                                    }
                                                    let result = if is_admin_clone && !is_author_clone {
                                                        api::admin_delete_wallpaper(id_to_delete.clone(), reason.clone()).await
                                                    } else {
                                                        api::delete_wallpaper_endpoint(id_to_delete.clone()).await
                                                    };
                                                    if result.is_ok() {
                                                        if is_admin_clone && !is_author_clone {
                                                            toaster
                                                                .info(
                                                                    format!(
                                                                        "Admin: Deleted '{}'. Reason: {}",
                                                                        wp_title,
                                                                        reason.unwrap_or_else(|| "None provided".into()),
                                                                    ),
                                                                );
                                                        } else {
                                                            toaster.success(i18n.t("success_wallpaper_deleted"));
                                                        }
                                                        nav.push(Route::Home {});
                                                    } else {
                                                        toaster.error(i18n.t("err_delete_wallpaper"));
                                                    }
                                                });
                                            },
                                            Trash2 { size: 24 }
                                        }
                                    }
                                    if is_admin {
                                        button {
                                            class: "glass glow-hover",
                                            style: "width: 56px; height: 56px; border-radius: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(255,255,255,0.1); background: rgba(239, 68, 68, 0.3); cursor: pointer; transition: all 0.2s ease; color: white;",
                                            title: "Admin: Ban Wallpaper & Hash",
                                            onclick: move |_| {
                                                let id_to_delete = delete_id_clone.clone();
                                                let mut toaster = toaster;
                                                let nav = nav;
                                                spawn(async move {
                                                    #[allow(unused_mut)]
                                                    let mut reason = None;
                                                    #[cfg(target_arch = "wasm32")]
                                                    if let Some(window) = web_sys::window() {
                                                        if let Ok(Some(r)) = window.prompt_with_message("Ban Reason:") {
                                                            reason = Some(r);
                                                        } else {
                                                            return;
                                                        }
                                                    }
                                                    if let Some(r) = reason {
                                                        if let Ok(_) = api::admin_ban_wallpaper_and_hash(id_to_delete, r).await {
                                                            toaster.success(i18n.t("success_banned_hashed"));
                                                            nav.push(Route::Home {});
                                                        } else {
                                                            toaster.error(i18n.t("err_ban"));
                                                        }
                                                    }
                                                });
                                            },
                                            Shield { size: 24 }
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
                                        if u.id == wp.author_id {
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
                                                        r#type: "text",
                                                        value: "{new_tag_input}",
                                                        oninput: move |e| new_tag_input.set(e.value()),
                                                        placeholder: "{i18n.t(\"wp_new_tag\")}",
                                                        style: "padding: 6px 12px; border-radius: 20px; font-size: 13px; font-weight: 700; background: rgba(0, 0, 0, 0.2); border: 1px solid rgba(255, 255, 255, 0.4); color: white; outline: none; width: 100px;",
                                                        autofocus: "true",
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
            
                        div { style: "margin-top: 80px;",
                            h2 { style: "margin-bottom: 32px;", "{i18n.t(\"wp_more_like_this\")}" }
                            div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px;",
                                match related() {
                                    Some(Ok(list)) => rsx! {
                                        for related_wp in list.iter().filter(|w| w.id != wp.id).take(3) {
                                            WallpaperCard { key: "{related_wp.id}", wallpaper: related_wp.clone() }
                                        }
                                    },
                                    _ => rsx! {
                                        LoadingScreen {}
                                    },
                                }
                            }
                        }
                    }
                }
                Some(Ok(None)) => rsx! {
                    div { "{i18n.t(\"wp_not_found\")}" }
                },
                Some(Err(e)) => rsx! {
                    div { class: "error", "{i18n.t(\"error\")}: {e}" }
                },
                None => rsx! {
                    LoadingScreen {}
                },
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
            onclick: move |_| {
                nav.push(Route::Search {
                    query: Some(tag.clone()),
                });
            },
            style: "cursor: pointer; padding: 6px 16px; border-radius: 20px; font-size: 13px; font-weight: 700; background: rgba(255, 255, 255, 0.1); border: 1px solid rgba(255, 255, 255, 0.2); transition: all 0.2s ease; display: inline-block; color: white;",
            "#{tag}"
        }
    }
}

#[component]
fn DetailItem(label: String, value: String) -> Element {
    rsx! {
        div {
            span { style: "display: block; font-size: 12px; color: var(--text-muted);",
                "{label}"
            }
            span { style: "font-weight: 700; font-size: 15px;", "{value}" }
        }
    }
}

#[component]
fn CommentsSection(wallpaper_id: String, is_wallpaper_author: bool) -> Element {
    let mut page = use_signal(|| 0u32);
    let mut all_comments = use_signal(Vec::<api::WallpaperComment>::new);
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
    let i18n = crate::i18n::use_i18n();

    rsx! {
        div { style: "margin-top: 80px;",
            h2 { style: "margin-bottom: 32px;", "{i18n.t(\"wp_comments\")}" }

            if let crate::app::AuthState::Authenticated(u) = is_auth() {
                div { style: "display: flex; gap: 16px; margin-bottom: 32px;",
                    img {
                        referrerpolicy: "no-referrer",
                        src: "{crate::resolve_asset_url(&u.pfp_url)}",
                        style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover;",
                    }
                    div { style: "flex: 1;",
                        textarea {
                            class: "glass",
                            style: "width: 100%; box-sizing: border-box; min-height: 80px; padding: 12px; border-radius: 12px; color: white; resize: vertical; border: 1px solid rgba(255,255,255,0.1);",
                            placeholder: "{i18n.t(\"wp_add_comment\")}",
                            value: "{new_comment}",
                            oninput: move |e| new_comment.set(e.value()),
                            maxlength: "500",
                        }
                        div { style: "display: flex; justify-content: flex-end; margin-top: 8px;",
                            span { style: "color: var(--text-muted); font-size: 12px; margin-right: 16px; align-self: center;",
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
                                            match api::add_wallpaper_comment(wid, text, None).await {
                                                Ok(_) => {
                                                    text_sig.set(String::new());
                                                    p.set(0);
                                                    res.restart();
                                                    toaster.success(i18n.t("success_comment_added"));
                                                }
                                                Err(e) => {
                                                    toaster.error(e.to_string());
                                                }
                                            }
                                        });
                                    }
                                },
                                "{i18n.t(\"wp_post_comment\")}"
                            }
                        }
                    }
                }
            } else {
                div { style: "margin-bottom: 32px; padding: 24px; background: rgba(255,255,255,0.02); border-radius: 12px; text-align: center; border: 1px dashed rgba(255,255,255,0.1);",
                    p { style: "color: var(--text-secondary); margin-bottom: 12px;",
                        "{i18n.t(\"wp_login_to_comment\")}"
                    }
                    Link {
                        to: Route::Login {},
                        class: "glow-hover",
                        style: "padding: 8px 24px; border-radius: 8px; background: rgba(255,255,255,0.1); color: white; text-decoration: none; display: inline-block; font-weight: 600;",
                        "Log In"
                    }
                }
            }

            div { style: "display: flex; flex-direction: column; gap: 24px;",
                if all_comments().is_empty() {
                    if let Some(Ok(_)) = comments_res() {
                        p { style: "color: var(--text-muted); text-align: center; padding: 24px;",
                            "{i18n.t(\"wp_no_comments_yet\")}"
                        }
                    } else if let Some(Err(_)) = comments_res() {
                        p { style: "color: #ef4444;", "{i18n.t(\"wp_failed_load_comments\")}" }
                    } else {
                        div { "{i18n.t(\"wp_loading_comments\")}" }
                    }
                } else {
                    for comment in all_comments() {
                        div {
                            key: "{comment.id}",
                            style: "display: flex; gap: 16px;",
                            img {
                                referrerpolicy: "no-referrer",
                                src: "{crate::resolve_asset_url(&comment.user_pfp)}",
                                style: "width: 48px; height: 48px; border-radius: 50%; object-fit: cover;",
                            }
                            div {
                                div { style: "display: flex; align-items: baseline; gap: 8px; margin-bottom: 4px;",
                                    Link {
                                        to: Route::PublicProfile {
                                            username: comment.user_name.replace(" ", "-"),
                                        },
                                        style: "font-weight: 700; font-size: 15px; color: white; text-decoration: none;",
                                        class: "glow-hover-text",
                                        "{comment.user_name}"
                                    }
                                    span { style: "font-size: 12px; color: var(--text-muted);",
                                        "{comment.created_at.split('T').next().unwrap_or(&comment.created_at)}"
                                    }

                                    if let crate::app::AuthState::Authenticated(u) = is_auth() {
                                        if u.id == comment.user_id || is_wallpaper_author {
                                            button {
                                                class: "glow-hover",
                                                style: "background: none; border: none; padding: 4px; margin-left: auto; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                                                onclick: {
                                                    let c_id = comment.id.clone();
                                                    let res = comments_res;
                                                    let p = page;
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
                                                                toaster.success(i18n.t("success_comment_deleted"));
                                                            } else {
                                                                toaster.error(i18n.t("err_delete_comment"));
                                                            }
                                                        });
                                                    }
                                                },
                                                lucide_dioxus::Trash2 {
                                                    size: 14,
                                                    color: "var(--text-muted)",
                                                }
                                            }
                                        }
                                    }
                                }
                                p { style: "color: var(--text-secondary); line-height: 1.5; font-size: 14px; white-space: pre-wrap;",
                                    "{comment.content}"
                                }
                            }
                        }
                    }

                    if has_more() {
                        div { style: "display: flex; justify-content: center; margin-top: 16px;",
                            button {
                                class: "glass glow-hover",
                                style: "padding: 12px 24px; border-radius: 12px; border: 1px dashed rgba(255,255,255,0.2); color: white; cursor: pointer; background: rgba(255,255,255,0.05); font-weight: 600;",
                                onclick: move |_| {
                                    page.set(page() + 1);
                                },
                                "{i18n.t(\"btn_load_more\")}"
                            }
                        }
                    }
                }
            }
        }
    }
}

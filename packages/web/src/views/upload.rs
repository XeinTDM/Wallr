use crate::Route;

use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::{LoadingScreen, use_toaster};

#[component]
pub fn Upload() -> Element {
    let mut title = use_signal(String::new);
    let mut category = use_signal(|| "abstract".to_string());
    let mut is_ai = use_signal(|| false);
    let mut is_nsfw = use_signal(|| false);
    let mut custom_tags = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut is_private = use_signal(|| false);
    let mut source_url = use_signal(String::new);
    let mut tos_agreed = use_signal(|| false);
    let mut show_advanced = use_signal(|| false);
    let mut is_dragging = use_signal(|| false);
    let mut is_uploading = use_signal(|| false);
    let mut upload_progress = use_signal(|| 0);
    let mut selected_file = use_signal(|| None::<(String, Vec<u8>, String, usize)>);
    let mut toaster = use_toaster();
    let nav = use_navigator();

    let user = use_context::<Signal<crate::AuthState>>();

    match user() {
        crate::AuthState::Loading => return rsx! { LoadingScreen {} },
        crate::AuthState::Unauthenticated => {
            nav.push(Route::Login {});
            return rsx! {};
        }
        crate::AuthState::Authenticated(_) => {}
    }

    let upload_action = move |_| async move {
        if title().is_empty() {
            toaster.error("Please provide a title for your masterpiece!");
            return;
        }

        if !tos_agreed() {
            toaster.error("Please confirm you have the rights to upload this image.");
            return;
        }

        if let Some((_name, bytes, _, _)) = selected_file() {
            is_uploading.set(true);
            upload_progress.set(0);
            let current_title = title();

            let mut current_progress = upload_progress;
            let is_uploading_check = is_uploading;
            spawn(async move {
                while is_uploading_check() && current_progress() < 90 {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::TimeoutFuture::new(200).await;
                    #[cfg(not(target_arch = "wasm32"))]
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    let curr = current_progress();
                    let increment = if curr < 30 {
                        10
                    } else if curr < 60 {
                        5
                    } else if curr < 85 {
                        2
                    } else {
                        1
                    };
                    current_progress.set((curr + increment).min(90));
                }
            });

            let mut tags = Vec::new();
            if !category().is_empty() {
                tags.push(category());
            }
            if is_ai() {
                tags.push("ai".to_string());
            }
            if is_nsfw() {
                tags.push("nsfw".to_string());
            }
            if !custom_tags().is_empty() {
                tags.push(custom_tags());
            }
            let tags_str = tags.join(",");

            let res = gloo_net::http::Request::post("/api/upload_raw")
                .header("X-Title", &current_title)
                .header("X-Tags", &tags_str)
                .header("X-Description", &description())
                .header("X-Source", &source_url())
                .header("X-Is-Private", &is_private().to_string())
                .body(bytes)
                .unwrap()
                .send()
                .await;

            upload_progress.set(100);

            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(300).await;
            #[cfg(not(target_arch = "wasm32"))]
            std::thread::sleep(std::time::Duration::from_millis(300));

            match res {
                Ok(resp) if resp.ok() => {
                    let id = resp.text().await.unwrap_or_default();
                    toaster.success("Wallpaper successfully published!");
                    nav.push(Route::WallpaperDetail { id });
                    return;
                }
                Ok(resp) => {
                    let err = resp.text().await.unwrap_or_default();
                    toaster.error(format!("Upload failed: {}", err));
                }
                Err(e) => {
                    toaster.error(format!("Upload failed: {}", e));
                }
            }
            is_uploading.set(false);
        } else {
            toaster.error("Please select a file first!");
        }
    };

    rsx! {
        div {
            class: "container",
            style: "padding-top: 120px; padding-bottom: 80px; max-width: 800px;",

            div {
                class: "section-header",
                style: "margin-bottom: 48px; text-align: center;",
                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 12px;",
                    class: "text-gradient",
                    "Share Your Art"
                }
                p {
                    style: "color: var(--text-secondary); font-size: 18px;",
                    "Upload high-resolution wallpapers to the Wallr community."
                }
            }

            div {
                class: "glass",
                style: "padding: 40px; border-radius: 32px; border: 1px solid rgba(255,255,255,0.1);",

                label {
                    class: "upload-zone",
                    style: format!(
                        "height: 300px; border: 2px dashed {}; border-radius: 24px; display: flex; flex-direction: column; align-items: center; justify-content: center; transition: all 0.3s; margin-bottom: 32px; background: {}; cursor: pointer;",
                        if is_dragging() { "var(--accent-primary)" } else { "rgba(255,255,255,0.1)" },
                        if is_dragging() { "rgba(59, 130, 246, 0.05)" } else { "transparent" }
                    ),
                    ondragover: move |e| { e.stop_propagation(); is_dragging.set(true); },
                    ondragleave: move |_| is_dragging.set(false),
                    ondrop: move |e| async move {
                        e.stop_propagation();
                        is_dragging.set(false);
                        let files = e.files();
                        if !files.is_empty() {
                            let file = &files[0];
                            let name = file.name();
                            if let Ok(bytes) = file.read_bytes().await {
                                if bytes.len() > 50 * 1024 * 1024 {
                                    toaster.error("File is too large! Maximum size is 50MB.");
                                } else {
                                    use base64::Engine;
                                    let mime = if name.to_lowercase().ends_with(".png") { "image/png" } else if name.to_lowercase().ends_with(".jpg") || name.to_lowercase().ends_with(".jpeg") { "image/jpeg" } else if name.to_lowercase().ends_with(".avif") { "image/avif" } else if name.to_lowercase().ends_with(".webp") { "image/webp" } else if name.to_lowercase().ends_with(".gif") { "image/gif" } else if name.to_lowercase().ends_with(".bmp") { "image/bmp" } else if name.to_lowercase().ends_with(".tiff") || name.to_lowercase().ends_with(".tif") { "image/tiff" } else { "image/png" };
                                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                    selected_file.set(Some((name.clone(), bytes.to_vec(), format!("data:{};base64,{}", mime, b64), bytes.len())));
                                }
                            }
                        }
                    },

                    input {
                        r#type: "file",
                        accept: "image/*",
                        style: "display: none;",
                        onchange: move |e| async move {
                            let files = e.files();
                            if !files.is_empty() {
                                let file = &files[0];
                                let name = file.name();
                                if let Ok(bytes) = file.read_bytes().await {
                                    if bytes.len() > 50 * 1024 * 1024 {
                                        toaster.error("File is too large! Maximum size is 50MB.");
                                    } else {
                                        use base64::Engine;
                                        let mime = if name.to_lowercase().ends_with(".png") { "image/png" } else if name.to_lowercase().ends_with(".jpg") || name.to_lowercase().ends_with(".jpeg") { "image/jpeg" } else if name.to_lowercase().ends_with(".avif") { "image/avif" } else if name.to_lowercase().ends_with(".webp") { "image/webp" } else if name.to_lowercase().ends_with(".gif") { "image/gif" } else if name.to_lowercase().ends_with(".bmp") { "image/bmp" } else if name.to_lowercase().ends_with(".tiff") || name.to_lowercase().ends_with(".tif") { "image/tiff" } else { "image/png" };
                                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                        selected_file.set(Some((name.clone(), bytes.to_vec(), format!("data:{};base64,{}", mime, b64), bytes.len())));
                                    }
                                }
                            }
                        }
                    }

                    if let Some((name, _, b64, size)) = selected_file() {
                        div {
                            style: "width: 100%; height: 100%; position: relative; border-radius: 20px; overflow: hidden; display: flex; flex-direction: column; align-items: center; justify-content: center;",
                            div {
                                style: "position: absolute; top: 16px; right: 16px; background: rgba(0,0,0,0.6); padding: 6px 12px; border-radius: 12px; backdrop-filter: blur(8px); z-index: 2; font-size: 13px; font-weight: 700; border: 1px solid rgba(255,255,255,0.1);",
                                "{size / 1024 / 1024} MB"
                            }
                            img {
                                src: "{b64}",
                                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: cover; opacity: 0.5; filter: blur(8px);"
                            }
                            img {
                                src: "{b64}",
                                style: "z-index: 1; max-width: 90%; max-height: 70%; object-fit: contain; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.5);"
                            }
                            div {
                                style: "z-index: 1; margin-top: 16px; display: flex; flex-direction: column; align-items: center;",
                                h3 { style: "text-shadow: 0 2px 4px rgba(0,0,0,0.8); margin-bottom: 4px;", "{name}" }
                                p {
                                    style: "color: white; font-weight: 800; cursor: pointer; text-shadow: 0 1px 2px rgba(0,0,0,0.8); background: rgba(0,0,0,0.6); padding: 6px 16px; border-radius: 20px; margin-top: 8px; font-size: 14px; border: 1px solid rgba(255,255,255,0.2); transition: all 0.2s;",
                                    onclick: move |e| { e.stop_propagation(); selected_file.set(None); },
                                    "Change file"
                                }
                            }
                        }
                    } else {
                        span { style: "font-size: 48px; margin-bottom: 16px;", "📁" }
                        h3 { "Click or drag to upload" }
                        p { style: "color: var(--text-secondary);", "Supports AVIF, PNG, JPG, WEBP, GIF, BMP, TIFF (Max 50MB)" }
                    }
                }

                div {
                    style: "display: grid; gap: 24px;",

                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 24px;",
                        div {
                            class: "input-group",
                            label {
                                style: "display: block; margin-bottom: 8px; font-weight: 600;",
                                "Wallpaper Title"
                            }
                            input {
                                class: "glass",
                                style: "width: 100%; padding: 14px 20px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none;",
                                placeholder: "e.g. Neon Horizon",
                                value: "{title}",
                                oninput: move |e| title.set(e.value())
                            }
                        }

                        div {
                            class: "input-group",
                            label {
                                style: "display: block; margin-bottom: 8px; font-weight: 600;",
                                "Category"
                            }
                            select {
                                class: "glass",
                                style: "width: 100%; padding: 14px 20px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none; appearance: none; cursor: pointer;",
                                value: "{category}",
                                onchange: move |e| category.set(e.value()),
                                for (val, label) in api::tags::CATEGORIES.iter() {
                                    option { key: "{val}", value: "{val}", style: "background: var(--bg-primary);", "{label}" }
                                }
                                option { value: "misc", style: "background: var(--bg-primary);", "Miscellaneous" }
                            }
                        }
                    }

                    div {
                        class: "input-group",
                        label {
                            style: "display: block; margin-bottom: 8px; font-weight: 600;",
                            "Custom Tags"
                        }
                        input {
                            class: "glass",
                            style: "width: 100%; padding: 14px 20px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none;",
                            placeholder: "e.g. neon, synthwave, dark (comma separated)",
                            value: "{custom_tags}",
                            oninput: move |e| custom_tags.set(e.value())
                        }
                        p {
                            style: "font-size: 13px; color: var(--text-secondary); margin-top: 8px; font-style: italic;",
                            "Note: Our AI will automatically analyze and add additional relevant tags."
                        }
                    }

                    div {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer; color: var(--text-secondary); font-weight: 500; user-select: none; margin-top: 8px; transition: color 0.2s;",
                        onclick: move |_| show_advanced.set(!show_advanced()),
                        span { style: format!("font-size: 14px; transition: transform 0.2s; transform: rotate({}deg); display: inline-block;", if show_advanced() { 90 } else { 0 }), "▶" }
                        span { "Advanced Options" }
                    }

                    if show_advanced() {
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 24px; animation: fade-in 0.3s ease;",
                            div {
                                class: "input-group",
                                label {
                                    style: "display: block; margin-bottom: 8px; font-weight: 600;",
                                    "Description (Optional)"
                                }
                                textarea {
                                    class: "glass",
                                    style: "width: 100%; padding: 14px 20px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none; resize: vertical; min-height: 80px;",
                                    placeholder: "Share the story or prompt behind this wallpaper...",
                                    value: "{description}",
                                    oninput: move |e| description.set(e.value())
                                }
                            }

                            div {
                                class: "input-group",
                                label {
                                    style: "display: block; margin-bottom: 8px; font-weight: 600;",
                                    "Source URL (Optional)"
                                }
                                input {
                                    class: "glass",
                                    style: "width: 100%; padding: 14px 20px; border-radius: 12px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none;",
                                    placeholder: "https://artstation.com/...",
                                    value: "{source_url}",
                                    oninput: move |e| source_url.set(e.value())
                                }
                            }
                        }
                    }

                    div {
                        style: "display: flex; gap: 32px; margin-top: 12px; margin-bottom: 8px;",
                        div {
                            style: "display: flex; align-items: center; gap: 12px; cursor: pointer; font-weight: 500; user-select: none;",
                            onclick: move |_| is_ai.set(!is_ai()),
                            div {
                                style: format!(
                                    "width: 44px; height: 24px; border-radius: 12px; background: {}; position: relative; transition: all 0.3s ease; border: 1px solid rgba(255,255,255,0.1); box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);",
                                    if is_ai() { "#3b82f6" } else { "rgba(255,255,255,0.1)" }
                                ),
                                div {
                                    style: format!(
                                        "width: 18px; height: 18px; border-radius: 50%; background: white; position: absolute; top: 2px; left: {}; transition: all 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); box-shadow: 0 2px 4px rgba(0,0,0,0.3);",
                                        if is_ai() { "22px" } else { "2px" }
                                    ),
                                }
                            }
                            "AI Generated"
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 12px; cursor: pointer; font-weight: 500; user-select: none;",
                            onclick: move |_| is_nsfw.set(!is_nsfw()),
                            div {
                                style: format!(
                                    "width: 44px; height: 24px; border-radius: 12px; background: {}; position: relative; transition: all 0.3s ease; border: 1px solid rgba(255,255,255,0.1); box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);",
                                    if is_nsfw() { "#ef4444" } else { "rgba(255,255,255,0.1)" }
                                ),
                                div {
                                    style: format!(
                                        "width: 18px; height: 18px; border-radius: 50%; background: white; position: absolute; top: 2px; left: {}; transition: all 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); box-shadow: 0 2px 4px rgba(0,0,0,0.3);",
                                        if is_nsfw() { "22px" } else { "2px" }
                                    ),
                                }
                            }
                            "NSFW / Mature"
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 12px; cursor: pointer; font-weight: 500; user-select: none;",
                            onclick: move |_| is_private.set(!is_private()),
                            div {
                                style: format!(
                                    "width: 44px; height: 24px; border-radius: 12px; background: {}; position: relative; transition: all 0.3s ease; border: 1px solid rgba(255,255,255,0.1); box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);",
                                    if is_private() { "#8b5cf6" } else { "rgba(255,255,255,0.1)" }
                                ),
                                div {
                                    style: format!(
                                        "width: 18px; height: 18px; border-radius: 50%; background: white; position: absolute; top: 2px; left: {}; transition: all 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); box-shadow: 0 2px 4px rgba(0,0,0,0.3);",
                                        if is_private() { "22px" } else { "2px" }
                                    ),
                                }
                            }
                            "Private (Unlisted)"
                        }
                    }

                    div {
                        style: "display: flex; align-items: flex-start; gap: 12px; margin-top: 8px; cursor: pointer;",
                        onclick: move |_| tos_agreed.set(!tos_agreed()),
                        div {
                            style: format!("width: 24px; height: 24px; min-width: 24px; border-radius: 6px; border: 2px solid {}; display: flex; align-items: center; justify-content: center; background: {}; transition: all 0.2s;",
                                if tos_agreed() { "#3b82f6" } else { "rgba(255,255,255,0.2)" },
                                if tos_agreed() { "#3b82f6" } else { "transparent" }
                            ),
                            if tos_agreed() {
                                span { style: "color: white; font-size: 14px; font-weight: bold;", "✓" }
                            }
                        }
                        p {
                            style: "font-size: 14px; color: var(--text-secondary); line-height: 1.5; margin: 0;",
                            "I confirm that I own the rights to this image or it is in the public domain, and it complies with our Terms of Service."
                        }
                    }

                    button {
                        class: "glow-hover",
                        style: format!("position: relative; overflow: hidden; margin-top: 16px; padding: 16px; font-weight: 600; font-size: 16px; display: flex; align-items: center; justify-content: center; gap: 12px; border-radius: 16px; transition: all 0.2s ease; cursor: {}; border: 1px solid rgba(255, 255, 255, 0.1); background: rgba(255, 255, 255, 0.05); color: var(--text-primary);", if is_uploading() { "not-allowed" } else { "pointer" }),
                        disabled: is_uploading(),
                        onclick: upload_action,
                        if is_uploading() {
                            div {
                                style: format!("position: absolute; top: 0; left: 0; height: 100%; width: {}%; background: rgba(59, 130, 246, 0.3); transition: width 0.2s ease; z-index: 0;", upload_progress()),
                            }
                            span { style: "z-index: 1;", "Publishing... {upload_progress()}%" }
                        } else {
                            span { style: "z-index: 1;", "Publish Wallpaper" }
                        }
                    }
                }
            }
        }
    }
}

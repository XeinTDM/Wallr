use dioxus::prelude::*;
use ui::app::Route;

mod win32_wallpaper;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::tao::window::Icon;
        use dioxus::desktop::{Config, WindowBuilder};

        #[cfg(feature = "server")]
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@127.0.0.1:5432/wallr".to_string()
            });
            if let Err(e) = api::storage::init_db(&db_url).await {
                eprintln!(
                    "⚠️ DB initialization failed: {}. Some features might not work.",
                    e
                );
            }
            api::ai::init_tagger();
        });

        let icon_bytes = include_bytes!("../assets/favicon.ico");
        let icon_image = image::load_from_memory(icon_bytes)
            .expect("Failed to load icon")
            .into_rgba8();
        let (width, height) = icon_image.dimensions();
        let icon =
            Icon::from_rgba(icon_image.into_raw(), width, height).expect("Failed to create icon");

        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new()
                    .with_data_directory(
                        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".desktop_data"),
                    )
                    .with_custom_index(
                        r#"
                            <!DOCTYPE html>
                            <html>
                            <head>
                                <title>Wallr</title>
                                <link rel="icon" href="data:;base64,iVBORw0KGgo=">
                                <style>body { margin: 0 !important; padding: 0 !important; }</style>
                            </head>
                            <body>
                                <div id="main"></div>
                            </body>
                            </html>
                        "#
                        .into(),
                    )
                    .with_window(
                        WindowBuilder::new()
                            .with_title("Wallr - Premium Wallpaper Engine")
                            .with_window_icon(Some(icon))
                            .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(
                                1280.0, 800.0,
                            )),
                    ),
            )
            .launch(App);
    }
    #[cfg(not(feature = "desktop"))]
    {
        #[cfg(feature = "server")]
        dioxus::serve(|| async move {
            let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@127.0.0.1:5432/wallr".to_string()
            });
            if let Err(e) = api::storage::init_db(&db_url).await {
                eprintln!(
                    "⚠️ DB initialization failed: {}. Some features might not work.",
                    e
                );
            }
            api::ai::init_tagger();

            let router = dioxus::server::router(App).nest_service(
                "/assets/uploads",
                tower_http::services::ServeDir::new("packages/ui/assets/uploads"),
            );
            Ok(router)
        });

        #[cfg(not(feature = "server"))]
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    use ui::{Theme, Toast, ToastContainer};

    use_context_provider(|| Signal::new(false)); // show_search
    use_context_provider(|| Signal::new(Vec::<Toast>::new())); // toasts
    ui::init_i18n();
    let i18n = ui::use_i18n();

    #[cfg(feature = "desktop")]
    {
        use global_hotkey::{
            GlobalHotKeyEvent, GlobalHotKeyManager,
            hotkey::{Code, HotKey, Modifiers},
        };
        use muda::{Menu, MenuItem, PredefinedMenuItem};
        use rand::seq::SliceRandom;
        use tray_icon::{MouseButton, TrayIconBuilder, TrayIconEvent};

        let mut _tray_icon = use_signal(|| None::<tray_icon::TrayIcon>);
        let mut _menu_channel = use_signal(|| None::<muda::MenuEventReceiver>);
        let mut _hotkey_manager = use_signal(|| None::<GlobalHotKeyManager>);

        use_hook(move || {
            let icon_bytes = include_bytes!("../assets/favicon.ico");
            let icon_image = image::load_from_memory(icon_bytes)
                .expect("Failed to load icon")
                .into_rgba8();
            let (width, height) = icon_image.dimensions();
            let tray_icon_data =
                tray_icon::Icon::from_rgba(icon_image.into_raw(), width, height).unwrap();

            let tray_menu = Menu::new();
            let open_i = MenuItem::new(i18n.t("sys_open_wallr"), true, None);
            let quit_i = MenuItem::new(i18n.t("sys_quit"), true, None);

            let _ = tray_menu.append(&open_i);
            let _ = tray_menu.append(&PredefinedMenuItem::separator());
            let _ = tray_menu.append(&quit_i);

            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("Wallr")
                .with_icon(tray_icon_data)
                .build()
                .unwrap();

            let open_id = open_i.id().clone();
            let quit_id = quit_i.id().clone();

            _tray_icon.set(Some(tray_icon));
            _menu_channel.set(Some(muda::MenuEvent::receiver().clone()));

            let manager = GlobalHotKeyManager::new().unwrap();
            let hotkey_w = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW);
            let hotkey_right = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowRight);
            let hotkey_left = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft);
            let hotkey_s = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS);
            let hotkey_h = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH);

            let _ = manager.register(hotkey_w);
            let _ = manager.register(hotkey_right);
            let _ = manager.register(hotkey_left);
            let _ = manager.register(hotkey_s);
            let _ = manager.register(hotkey_h);

            let id_w = hotkey_w.id();
            let id_right = hotkey_right.id();
            let id_left = hotkey_left.id();
            let id_s = hotkey_s.id();
            let id_h = hotkey_h.id();
            
            _hotkey_manager.set(Some(manager));

            spawn(async move {
                let mut last_sync: Option<std::time::Instant> = None;
                loop {
                    if let Ok((playlist, interval_secs)) = api::get_active_playlist_items().await {
                        let should_sync = match last_sync {
                            None => true,
                            Some(sync) => sync.elapsed().as_secs() >= interval_secs as u64,
                        };

                        if !playlist.is_empty() && should_sync {
                            last_sync = Some(std::time::Instant::now());
                            let mut rng = rand::thread_rng();
                            if let Some(wp) = playlist.choose(&mut rng) {
                                let window = dioxus::desktop::window();
                                apply_wallpaper(window, wp.clone()).await;
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            });

            spawn(async move {
                let menu_channel = muda::MenuEvent::receiver();
                let tray_channel = TrayIconEvent::receiver();
                let hotkey_channel = GlobalHotKeyEvent::receiver();

                loop {
                    if let Ok(event) = menu_channel.try_recv() {
                        if event.id == open_id {
                            let window = dioxus::desktop::window();
                            window.set_minimized(false);
                            window.set_visible(true);
                            window.set_focus();
                        } else if event.id == quit_id {
                            std::process::exit(0);
                        }
                    }

                    if let Ok(event) = tray_channel.try_recv()
                        && let tray_icon::TrayIconEvent::Click {
                            button: MouseButton::Left,
                            ..
                        } = event
                        {
                            let window = dioxus::desktop::window();
                            let is_minimized = window.is_minimized();
                            let is_visible = window.is_visible();

                            if is_minimized || !is_visible {
                                window.set_minimized(false);
                                window.set_visible(true);
                                window.set_focus();
                            } else {
                                window.set_minimized(true);
                                window.set_visible(false);
                            }
                        }

                    if let Ok(event) = hotkey_channel.try_recv() {
                        if event.id == id_w || event.id == id_right || event.id == id_left {
                            let mut candidates = vec![];
                            if let Ok((playlist, _)) = api::get_active_playlist_items().await {
                                candidates = playlist;
                            }
                            if candidates.is_empty()
                                && let Ok(favs) = api::get_user_favorites(None::<String>, 100).await {
                                    candidates = favs.to_vec();
                                }

                            if !candidates.is_empty() {
                                let mut rng = rand::thread_rng();
                                if let Some(wp) = candidates.choose(&mut rng) {
                                    let window = dioxus::desktop::window();
                                    apply_wallpaper(window, wp.clone()).await;
                                }
                            }
                        } else if event.id == id_h {
                            let window = dioxus::desktop::window();
                            let is_minimized = window.is_minimized();
                            let is_visible = window.is_visible();

                            if is_minimized || !is_visible {
                                window.set_minimized(false);
                                window.set_visible(true);
                                window.set_focus();
                            } else {
                                window.set_minimized(true);
                                window.set_visible(false);
                            }
                        } else if event.id == id_s {
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(path) = wallpaper::get() {
                                    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:\\"));
                                    let downloads = std::path::PathBuf::from(home).join("Downloads");
                                    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                                    let dest = downloads.join(format!("wallr_saved_{}.jpg", ts));
                                    let _ = std::fs::copy(path, dest);
                                }
                            }).await;
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            });
        });
    }

    #[cfg(feature = "desktop")]
    dioxus::desktop::use_asset_handler("upload", move |req, responder| {
        let path = req.uri().path();
        let file_name = path
            .strip_prefix("/upload/")
            .unwrap_or(path)
            .trim_start_matches('/');
        let full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ui/assets/uploads")
            .join(file_name);

        let data = std::fs::read(&full_path).unwrap_or_default();
        
        let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mime = match ext {
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "image/avif",
        };

        let response = dioxus::desktop::wry::http::Response::builder()
            .header("Content-Type", mime)
            .header("Access-Control-Allow-Origin", "*")
            .body(data)
            .unwrap();
        responder.respond(response);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        Theme {}
        ToastContainer {}

        SuspenseBoundary {
            fallback: |_| rsx! {},
            Router::<Route> {}
        }
    }
}

#[cfg(feature = "desktop")]
#[derive(Props, Clone, PartialEq)]
struct LiveWallpaperProps {
    url: String,
}

#[cfg(feature = "desktop")]
static LIVE_WP_URL: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

#[cfg(feature = "desktop")]
#[component]
fn LiveWallpaperView(props: LiveWallpaperProps) -> Element {
    let window = dioxus::desktop::window();
    let url_clone = props.url.clone();
    
    use_hook(move || {
        let tao_window = window.window.clone();
        #[cfg(target_os = "windows")]
        {
            use dioxus::desktop::tao::platform::windows::WindowExtWindows;
            let hwnd = tao_window.hwnd();
            crate::win32_wallpaper::windows_wallpaper::attach_to_desktop(hwnd as isize);
        }
        
        let window_clone = window.clone();
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(guard) = LIVE_WP_URL.lock() {
                    if *guard != url_clone {
                        window_clone.close();
                        break;
                    }
                }
            }
        });
    });

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; overflow: hidden; background: black;",
            video {
                src: "{props.url}",
                autoplay: true,
                loop: true,
                muted: true,
                style: "width: 100%; height: 100%; object-fit: cover; object-position: center;"
            }
        }
    }
}

#[cfg(feature = "desktop")]
async fn apply_wallpaper(
    window: dioxus::desktop::DesktopContext,
    wp: api::Wallpaper,
) {
    if wp.is_live {
        use dioxus::desktop::{Config, WindowBuilder};
        let filename = wp.image_url.strip_prefix("/assets/uploads/").unwrap_or(&wp.image_url);
        let url = format!("/upload/{}", filename);
        
        if let Ok(mut guard) = LIVE_WP_URL.lock() {
            if *guard == url {
                return; // Already playing this live wallpaper
            }
            *guard = url.clone();
        }

        let dom = dioxus::core::VirtualDom::new_with_props(LiveWallpaperView, LiveWallpaperProps { url });
        let wb = WindowBuilder::new()
            .with_title("Wallr Live Background")
            .with_decorations(false)
            .with_always_on_bottom(true)
            .with_maximized(true);

        let cfg = Config::new().with_window(wb);
        window.new_window(dom, cfg);
    } else {
        if let Ok(mut guard) = LIVE_WP_URL.lock() {
            *guard = String::new(); // Stop any active live wallpaper
        }

        let image_url = wp.image_url.clone();
        let wp_id = wp.id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let filename = image_url
                .strip_prefix("/assets/uploads/")
                .unwrap_or(&image_url);
            let full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../ui/assets/uploads")
                .join(filename);
            if let Ok(img) = image::open(&full_path) {
                let temp_dir = std::env::temp_dir();
                let temp_path = temp_dir.join(format!("wallr_{}.jpg", wp_id));
                if img.save(&temp_path).is_ok() {
                    let _ = wallpaper::set_from_path(temp_path.to_str().unwrap());
                }
            }
        })
        .await;
    }
}

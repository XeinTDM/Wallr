use dioxus::prelude::*;
use ui::app::{AuthState, Route};

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
    use_context_provider(|| Signal::new(AuthState::Loading)); // auth state
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

                    if let Ok(event) = tray_channel.try_recv() {
                        if let tray_icon::TrayIconEvent::Click {
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
                    }

                    if let Ok(event) = hotkey_channel.try_recv() {
                        if event.id == id_w || event.id == id_right || event.id == id_left {
                            if let Ok(favs) = api::get_user_favorites(0, 100).await {
                                if !favs.is_empty() {
                                    let mut rng = rand::thread_rng();
                                    if let Some(wp) = favs.choose(&mut rng) {
                                        let image_url = wp.image_url.clone();
                                        let wp_id = wp.id.clone();
                                        let _ = tokio::task::spawn_blocking(move || {
                                            let filename = image_url
                                                .strip_prefix("/assets/uploads/")
                                                .unwrap_or(&image_url);
                                            let full_path = std::path::PathBuf::from(env!(
                                                "CARGO_MANIFEST_DIR"
                                            ))
                                            .join("../ui/assets/uploads")
                                            .join(filename);
                                            if let Ok(img) = image::open(&full_path) {
                                                let temp_dir = std::env::temp_dir();
                                                let temp_path =
                                                    temp_dir.join(format!("wallr_{}.jpg", wp_id));
                                                if img.save(&temp_path).is_ok() {
                                                    let _ = wallpaper::set_from_path(
                                                        temp_path.to_str().unwrap(),
                                                    );
                                                }
                                            }
                                        })
                                        .await;
                                    }
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

        let response = dioxus::desktop::wry::http::Response::builder()
            .header("Content-Type", "image/avif")
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

        Router::<Route> {}
    }
}

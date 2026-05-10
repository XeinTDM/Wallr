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

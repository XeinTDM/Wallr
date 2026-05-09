use dioxus::prelude::*;
use ui::app::{AuthState, Route};
use ui::{Theme, Toast, ToastContainer};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "server")]
    {
        use axum::extract::DefaultBodyLimit;
        use axum::response::IntoResponse;
        use axum::routing::{get, post};

        #[derive(serde::Deserialize)]
        struct DownloadQuery {
            format: Option<String>,
            width: Option<u32>,
        }

        fn extract_session_token(headers: &axum::http::HeaderMap) -> Option<String> {
            headers
                .get_all(axum::http::header::COOKIE)
                .iter()
                .filter_map(|val| val.to_str().ok())
                .flat_map(|cookie_str| cookie_str.split(';'))
                .map(|s| s.trim())
                .find(|s| s.starts_with("session_token="))
                .map(|s| s.trim_start_matches("session_token=").to_string())
        }

        async fn download_handler(
            headers: axum::http::HeaderMap,
            axum::extract::Path(id): axum::extract::Path<String>,
            axum::extract::Query(query): axum::extract::Query<DownloadQuery>,
        ) -> impl axum::response::IntoResponse {
            use axum::body::Body;
            use axum::http::StatusCode;

            let format = query
                .format
                .unwrap_or_else(|| "avif".to_string())
                .to_lowercase();
            let width = query.width;

            let source_path = if format == "avif" && width.is_none() {
                format!("packages/ui/assets/uploads/{}_master.avif", id)
            } else {
                format!("packages/ui/assets/uploads/{}_master.jpg", id)
            };

            let mut ip = "unknown".to_string();
            if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
                if let Some(first_ip) = xff.split(',').next() {
                    let trimmed = first_ip.trim();
                    if !trimmed.is_empty() {
                        ip = trimmed.to_string();
                    }
                }
            } else if let Some(xrip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
                let trimmed = xrip.trim();
                if !trimmed.is_empty() {
                    ip = trimmed.to_string();
                }
            }

            if ip == "unknown" {
                if let Some(token) = extract_session_token(&headers) {
                    ip = format!("session_{}", token);
                }
            }

            let download_id = id.clone();
            let download_ip = ip.clone();
            tokio::spawn(async move {
                let _ = api::storage::increment_download(&download_id, &download_ip).await;
            });

            let is_native_format = format == "avif" || format == "jpg" || format == "jpeg";
            if width.is_none() && is_native_format {
                let file = match tokio::fs::File::open(&source_path).await {
                    Ok(f) => f,
                    Err(_) => {
                        let fallback_path = format!("packages/ui/assets/uploads/{}_master.jpg", id);
                        match tokio::fs::File::open(&fallback_path).await {
                            Ok(f) => f,
                            Err(_) => {
                                return (StatusCode::NOT_FOUND, "Not found".to_string())
                                    .into_response();
                            }
                        }
                    }
                };

                let stream = tokio_util::io::ReaderStream::new(file);
                let body = Body::from_stream(stream);

                let mut res_headers = axum::http::HeaderMap::new();
                let content_type = match format.as_str() {
                    "avif" => "image/avif",
                    "jpeg" | "jpg" => "image/jpeg",
                    "png" => "image/png",
                    _ => "application/octet-stream",
                };
                res_headers.insert("Content-Type", content_type.parse().unwrap());
                res_headers.insert(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}.{}\"", id, format)
                        .parse()
                        .unwrap(),
                );
                return (res_headers, body).into_response();
            }

            let data = match tokio::fs::read(&source_path).await {
                Ok(d) => d,
                Err(_) => {
                    let fallback_path = format!("packages/ui/assets/uploads/{}_master.jpg", id);
                    match tokio::fs::read(&fallback_path).await {
                        Ok(d) => d,
                        Err(_) => {
                            return (StatusCode::NOT_FOUND, "Not found".to_string())
                                .into_response();
                        }
                    }
                }
            };

            let format_str = format.clone();
            let out_bytes = match tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
                let mut img = match image::load_from_memory(&data) {
                    Ok(i) => i,
                    Err(e) => return Err(format!("Failed to decode source image: {}", e)),
                };

                if let Some(w) = width {
                    if w < img.width() {
                        img = img.resize(w, u32::MAX, image::imageops::FilterType::Lanczos3);
                    }
                }

                let mut out = std::io::Cursor::new(Vec::new());
                let img_format = match format_str.as_str() {
                    "jpeg" | "jpg" => image::ImageFormat::Jpeg,
                    "png" => image::ImageFormat::Png,
                    _ => return Err("Unsupported format".to_string()),
                };

                if let Err(e) = img.write_to(&mut out, img_format) {
                    return Err(format!("Failed to encode: {}", e));
                }

                Ok(out.into_inner())
            })
            .await
            {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(e)) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Task failed: {}", e),
                    )
                        .into_response();
                }
            };

            let mut headers = axum::http::HeaderMap::new();
            let content_type = match format.as_str() {
                "jpeg" | "jpg" => "image/jpeg",
                "png" => "image/png",
                _ => "application/octet-stream",
            };
            headers.insert("Content-Type", content_type.parse().unwrap());
            headers.insert(
                "Content-Disposition",
                format!("attachment; filename=\"{}.{}\"", id, format)
                    .parse()
                    .unwrap(),
            );

            (headers, out_bytes).into_response()
        }

        async fn upload_raw_handler(
            headers: axum::http::HeaderMap,
            body: bytes::Bytes,
        ) -> axum::response::Response {
            let title = headers
                .get("X-Title")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("Untitled")
                .to_string();
            let author = {
                let token = extract_session_token(&headers);

                if let Some(token_str) = token {
                    api::storage::verify_token(&token_str)
                        .await
                        .map(|u| u.name)
                        .unwrap_or_else(|_| "Anonymous".to_string())
                } else {
                    "Anonymous".to_string()
                }
            };
            let user_tags: Vec<String> = headers
                .get("X-Tags")
                .and_then(|v| v.to_str().ok())
                .map(|s| {
                    s.split(',')
                        .filter(|t| !t.trim().is_empty())
                        .map(|t| t.trim().to_lowercase())
                        .collect()
                })
                .unwrap_or_default();

            let is_private = headers
                .get("X-Is-Private")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "true")
                .unwrap_or(false);

            match api::upload_raw_impl(title, author, user_tags, body.to_vec(), is_private).await {
                Ok(id) => axum::response::Response::new(id.into()),
                Err(e) => {
                    let mut res = axum::response::Response::new(e.to_string().into());
                    *res.status_mut() = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                    res
                }
            }
        }

        async fn upload_media_handler(
            headers: axum::http::HeaderMap,
            body: bytes::Bytes,
        ) -> axum::response::Response {
            let media_type = headers
                .get("X-Media-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("pfp")
                .to_string();

            let user_id = {
                let token = extract_session_token(&headers);

                if let Some(token_str) = token {
                    api::storage::verify_token(&token_str)
                        .await
                        .map(|u| u.id)
                        .unwrap_or_default()
                } else {
                    "".to_string()
                }
            };

            if user_id.is_empty() {
                let mut res = axum::response::Response::new("Unauthorized".into());
                *res.status_mut() = axum::http::StatusCode::UNAUTHORIZED;
                return res;
            }

            match api::upload_media_impl(user_id.clone(), media_type, body.to_vec()).await {
                Ok(url) => {
                    let mut res = axum::response::Response::new(url.into());
                    if let Ok(Some(record)) = api::storage::get_user_by_id(&user_id).await {
                        if let Ok(new_token) =
                            api::storage::generate_token(&record.user, record.token_version)
                        {
                            let cookie = format!(
                                "session_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=2592000",
                                new_token
                            );
                            res.headers_mut()
                                .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
                        }
                    }
                    res
                }
                Err(e) => {
                    let mut res = axum::response::Response::new(e.to_string().into());
                    *res.status_mut() = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                    res
                }
            }
        }

        async fn export_data_handler(
            headers: axum::http::HeaderMap,
        ) -> impl axum::response::IntoResponse {
            let user_id = {
                let token = extract_session_token(&headers);
                if let Some(token_str) = token {
                    api::storage::verify_token(&token_str)
                        .await
                        .map(|u| u.id)
                        .unwrap_or_default()
                } else {
                    "".to_string()
                }
            };

            if user_id.is_empty() {
                return (
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Unauthorized".to_string(),
                )
                    .into_response();
            }

            match api::storage::export_user_data(&user_id).await {
                Ok(filepath) => {
                    let file = match tokio::fs::File::open(&filepath).await {
                        Ok(f) => f,
                        Err(e) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to open generated export: {}", e),
                            )
                                .into_response();
                        }
                    };

                    let path_to_delete = filepath.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(10 * 60)).await;
                        let _ = tokio::fs::remove_file(path_to_delete).await;
                    });

                    let stream = tokio_util::io::ReaderStream::new(file);
                    let body = axum::body::Body::from_stream(stream);

                    let mut res_headers = axum::http::HeaderMap::new();
                    res_headers.insert("Content-Type", "application/gzip".parse().unwrap());
                    res_headers.insert(
                        "Content-Disposition",
                        "attachment; filename=\"wallr_backup.tar.gz\""
                            .parse()
                            .unwrap(),
                    );
                    (res_headers, body).into_response()
                }
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Export failed: {}", e),
                )
                    .into_response(),
            }
        }

        dioxus::serve(|| async move {
            let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@127.0.0.1:5432/wallr".to_string()
            });
            if let Err(e) = api::storage::init_db(&db_url).await {
                eprintln!(
                    "⚠️ DB initialization failed: {}. Some features might not work.",
                    e
                );
            } else {
            }
            api::ai::init_tagger();
            let router = dioxus::server::router(App);
            let app = router
                .nest_service(
                    "/assets/uploads",
                    tower_http::services::ServeDir::new("packages/ui/assets/uploads"),
                )
                .route("/api/upload_raw", post(upload_raw_handler))
                .route("/api/upload_media", post(upload_media_handler))
                .route("/api/export_data", get(export_data_handler))
                .route("/wallpaper/{id}/download", get(download_handler))
                .layer(DefaultBodyLimit::disable());
            Ok(app)
        });
    }
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(false));
    use_context_provider(|| Signal::new(Vec::<Toast>::new()));
    use_context_provider(|| Signal::new(AuthState::Loading));

    rsx! {
        document::Title { "Wallr | Optimal Wallpaper Engine" }
        document::Meta { name: "description", content: "A hyper-optimized wallpaper platform. AVIF-native, zero-latency focused, and powered by local AI." }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "icon", href: FAVICON }
        Theme {}
        ToastContainer {}
        document::Stylesheet { href: MAIN_CSS }

        Router::<Route> {}
    }
}

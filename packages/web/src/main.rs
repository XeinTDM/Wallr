use dioxus::prelude::*;
use ui::app::{AuthState, Route};
use ui::{Theme, Toast, ToastContainer};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "server")]
    dotenvy::dotenv().ok();

    #[cfg(feature = "server")]
    {
        use axum::extract::DefaultBodyLimit;
        use axum::response::IntoResponse;
        use axum::routing::{get, post};

        #[derive(serde::Deserialize)]
        struct DownloadQuery {
            format: Option<String>,
            width: Option<u32>,
            height: Option<u32>,
            crop: Option<String>,
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

            if id.contains('.') || id.contains('/') || id.contains('\\') {
                return (StatusCode::BAD_REQUEST, "Invalid ID".to_string()).into_response();
            }

            let format = query
                .format
                .unwrap_or_else(|| "avif".to_string())
                .to_lowercase();
            let width = query.width;
            let height = query.height;

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
            let token_opt = extract_session_token(&headers);
            tokio::spawn(async move {
                let _ = api::storage::increment_download(&download_id, &download_ip).await;
                if let Some(token_str) = token_opt {
                    if let Ok(user) = api::storage::verify_token(&token_str).await {
                        let _ = api::storage::record_user_download_db(&user.id, &download_id).await;
                    }
                }
            });

            let is_native_format = format == "avif" || format == "jpg" || format == "jpeg";
            if width.is_none() && height.is_none() && is_native_format {
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

                let perform_resize = width.is_some() || height.is_some();
                if perform_resize {
                    use fast_image_resize as fr;
                    
                    let src_image = fr::images::Image::from_vec_u8(
                        img.width(),
                        img.height(),
                        img.to_rgba8().into_raw(),
                        fr::PixelType::U8x4,
                    ).map_err(|e| format!("FR Error: {:?}", e))?;

                    let mut dst_width = img.width();
                    let mut dst_height = img.height();
                    let mut crop_left = 0.0;
                    let mut crop_top = 0.0;
                    let mut crop_width = img.width() as f64;
                    let mut crop_height = img.height() as f64;

                    if let Some(w) = width {
                        if let Some(h) = height {
                            dst_width = w;
                            dst_height = h;

                            let src_ratio = img.width() as f64 / img.height() as f64;
                            let dst_ratio = w as f64 / h as f64;

                            if src_ratio > dst_ratio {
                                crop_height = img.height() as f64;
                                crop_width = crop_height * dst_ratio;
                                let crop_val = query.crop.as_deref().unwrap_or("center").to_lowercase();
                                if crop_val == "left" {
                                    crop_left = 0.0;
                                } else if crop_val == "right" {
                                    crop_left = img.width() as f64 - crop_width;
                                } else {
                                    crop_left = (img.width() as f64 - crop_width) / 2.0;
                                }
                            } else {
                                crop_width = img.width() as f64;
                                crop_height = crop_width / dst_ratio;
                                let crop_val = query.crop.as_deref().unwrap_or("center").to_lowercase();
                                if crop_val == "top" {
                                    crop_top = 0.0;
                                } else if crop_val == "bottom" {
                                    crop_top = img.height() as f64 - crop_height;
                                } else {
                                    crop_top = (img.height() as f64 - crop_height) / 2.0;
                                }
                            }
                        } else if w < img.width() {
                            dst_width = w;
                            let scale = w as f64 / img.width() as f64;
                            dst_height = (img.height() as f64 * scale) as u32;
                        }
                    } else if let Some(h) = height {
                        if h < img.height() {
                            dst_height = h;
                            let scale = h as f64 / img.height() as f64;
                            dst_width = (img.width() as f64 * scale) as u32;
                        }
                    }

                    if dst_width == 0 { dst_width = 1; }
                    if dst_height == 0 { dst_height = 1; }

                    let mut dst_image = fr::images::Image::new(dst_width, dst_height, fr::PixelType::U8x4);
                    let mut resizer = fr::Resizer::new();

                    resizer.resize(
                        &src_image,
                        &mut dst_image,
                        &fr::ResizeOptions::new()
                            .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3))
                            .crop(crop_left, crop_top, crop_width, crop_height),
                    ).map_err(|e| format!("FR Resize Error: {:?}", e))?;

                    img = image::DynamicImage::ImageRgba8(
                        image::RgbaImage::from_raw(dst_width, dst_height, dst_image.into_vec())
                            .ok_or("Failed to create RgbaImage")?
                    );
                }

                let mut out = std::io::Cursor::new(Vec::new());
                let img_format = match format_str.as_str() {
                    "jpeg" | "jpg" => image::ImageFormat::Jpeg,
                    "avif" => image::ImageFormat::Avif,
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
            let (author_id, author_name) = {
                let token = extract_session_token(&headers);

                if let Some(token_str) = token {
                    api::storage::verify_token(&token_str)
                        .await
                        .map(|u| (u.id, u.name))
                        .unwrap_or_else(|_| ("".to_string(), "Anonymous".to_string()))
                } else {
                    ("".to_string(), "Anonymous".to_string())
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

            match api::upload_raw_impl(title, author_id, author_name, user_tags, body.to_vec(), is_private).await {
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
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
                    loop {
                        interval.tick().await;
                        let _ = api::storage::refresh_trending_tags_view().await;
                    }
                });
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
                .nest("/api/oauth", api::oauth::oauth_router())
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
    ui::init_i18n();

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

# Scaffold Server Endpoint

Use this skill when asked to create a new backend API endpoint, data fetcher, or server mutation for Wallr.

## Gotchas

- **Endpoint Types**: Wallr uses **two** distinct types of endpoints depending on the payload:
  1. **Standard Data/Auth (Default)**: Use Dioxus `#[server]` functions located in `packages/api/src/lib.rs`.
  2. **Large Binary Payloads (e.g., Wallpaper Uploads)**: Use custom raw **Axum** handlers in `packages/web/src/main.rs`. Dioxus server functions have default body size limits that cause HTTP 500 errors for files > 20MB.
- **ServerFnError**: When using `#[server]`, the return type must be `Result<T, ServerFnError>`. Map underlying string or SQLx errors using `.map_err(|e| ServerFnError::new(e))`.
- **Client Hydration**: When consuming `#[server]` functions on the client for data that is critical for the initial page render, use `use_server_future` instead of `use_resource`. This ensures the server pre-fetches the data and prevents UI hydration mismatches.

## Workflow

### Option 1: Standard Dioxus Server Function (Use for 95% of tasks)

Progress:
- [ ] Step 1: Open `packages/api/src/lib.rs`.
- [ ] Step 2: Add the `#[server]` macro above your new async function.
- [ ] Step 3: Ensure the signature returns `Result<T, ServerFnError>`.
- [ ] Step 4: Map your database or storage call inside the function using `.map_err(|e| ServerFnError::new(e))?`.
- [ ] Step 5: On the frontend (e.g., in `packages/ui/src/`), call the function using `use_server_future` (if it blocks render) or `use_resource` (if it loads subsequently).

### Option 2: Raw Axum Handler (Use ONLY for large file uploads/binary streams)

Progress:
- [ ] Step 1: Open `packages/web/src/main.rs`.
- [ ] Step 2: Define your async handler function (e.g., `async fn handle_upload(body: axum::body::Bytes) -> impl IntoResponse`).
- [ ] Step 3: Add it to the Axum Router *before* the fallback/serve-dir configuration. Example: `.route("/api/upload", post(handle_upload))`.
- [ ] Step 4: Ensure Axum's `DefaultBodyLimit::max()` or a custom limit middleware is applied to the router.
- [ ] Step 5: On the frontend, do not use `#[server]` bindings. Instead, manually execute a `POST` request using `gloo_net::http::Request`.

## Templates

### 1. Dioxus Server Function (`packages/api/src/lib.rs`)

```rust
use dioxus::prelude::*;

#[server]
pub async fn get_custom_data(id: String) -> Result<String, ServerFnError> {
    // 1. Call storage or db
    let data = crate::storage::fetch_custom_data(&id)
        .await
        .map_err(|e| ServerFnError::new(e))?;
        
    Ok(data)
}
```

### 2. Frontend Consumption (`packages/ui/src/my_component.rs`)

```rust
use dioxus::prelude::*;

#[component]
pub fn CustomDataView(id: String) -> Element {
    // use_server_future prevents hydration mismatch by resolving on the server first
    let custom_data = use_server_future(move || {
        let id_clone = id.clone();
        async move {
            api::get_custom_data(id_clone).await
        }
    })?;

    match custom_data() {
        Some(Ok(data)) => rsx! { div { "Data: {data}" } },
        Some(Err(e)) => rsx! { div { class: "error", "Error: {e}" } },
        None => rsx! { div { class: "skeleton-pulse", "Loading..." } }, // Fallback for client-side routing
    }
}
```

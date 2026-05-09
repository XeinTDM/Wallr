# Wallr API

This crate contains the backend services, Dioxus Server Functions, and raw Axum endpoints for the Wallr project. It handles:
- Compile-time verified PostgreSQL interactions via `sqlx`.
- The AI image tagging pipeline using `candle`.
- Axum handlers for large binary file uploads.

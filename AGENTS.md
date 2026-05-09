# Wallr Project Overview & Agent Instructions

Welcome to the Wallr repository. Wallr is a premium, full-stack wallpaper engine built with Rust. 

## Technology Stack
- **Frontend**: Dioxus 0.7 (Web & Desktop).
- **Backend**: Dioxus Server Functions + Axum (for raw handlers like large file uploads).
- **Database**: PostgreSQL (via Docker) with `sqlx` for compile-time verified queries.
- **AI/ML**: `candle` pipeline for automated image tagging via Hugging Face weights.
- **Design System**: Custom CSS (No Tailwind). We use a "glassmorphic" UI, deep responsive styling, and `lucide-dioxus` icons.

## Agent Guidelines & Progressive Disclosure

Do not rely on generic LLM knowledge. Always refer to the following resources depending on the task:

### 1. Dioxus 0.7 Syntax Reference
Dioxus 0.7 has breaking changes (no `cx`, no `Scope`). If you need to understand the syntax for RSX, hooks, state, routing, or fullstack server functions, read this file:
👉 `docs/dioxus_0.7_reference.md`

### 2. Available Skills (Workflows)
We have formalized specific workflows in the `skills/` directory. Load these files when performing their respective tasks to ensure you don't miss project-specific gotchas:

- **Creating UI Components**: Read `skills/create_dioxus_component/SKILL.md`
- **Database Migrations/Schema Updates**: Read `skills/sqlx_migration_workflow/SKILL.md`
- **Adding Server Endpoints**: Read `skills/scaffold_server_endpoint/SKILL.md`

## Key Architecture Rules
- **Aesthetics First**: Every UI component must feel premium. Use blurred backgrounds, smooth transitions, and refined color palettes. Never use generic styles.
- **Compile-Time SQL**: Any changes to database queries or structs MUST be followed by `cargo sqlx prepare --workspace`.
- **Upload Limits**: Use Axum for binary file uploads; Dioxus `#[server]` functions will return HTTP 500 on large files.

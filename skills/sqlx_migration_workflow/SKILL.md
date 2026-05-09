# SQLx Migration Workflow

Use this skill when asked to update the database schema, add new tables, modify existing database structures, or write new SQL queries in Wallr.

## Gotchas

- **Legacy Schema Initialization**: Historically, Wallr initialized its database using an array of `CREATE TABLE IF NOT EXISTS` strings in `packages/api/src/storage.rs`. If the user asks to add formal migrations, you must use the `sqlx-cli` to create `.sql` files instead. If you just need a quick schema update without formal migrations, update the `schemas` array in `packages/api/src/storage.rs`.
- **Docker Dependency**: Formal Migrations and `sqlx prepare` require a live database connection. Ensure the local PostgreSQL Docker container (`wallr_db`) is running on port `5432`.
- **Compile-Time Verification**: Wallr uses `sqlx` query verification. If you change the schema or add/modify queries in Rust files, you **must** run `cargo sqlx prepare` before finishing your task, or the build will fail on other machines/CI.
- **Data Types**: Postgres `BIGINT` maps to Rust `i64`. Postgres `JSONB` maps to `serde_json::Value`.

## Workflow (Formal Migrations)

If generating a formal migration, follow this exact sequence:

Progress:
- [ ] Step 1: Ensure the database is running. Check with `docker ps`. If not running, execute `docker-compose up -d db` in the root directory.
- [ ] Step 2: Create a new migration by running `cargo sqlx migrate add <migration_name>`. (Ensure `sqlx-cli` is installed).
- [ ] Step 3: Fill in the generated `<timestamp>_<migration_name>.up.sql` with your `CREATE` or `ALTER` statements.
- [ ] Step 4: Fill in the generated `<timestamp>_<migration_name>.down.sql` with the corresponding `DROP` or reverse `ALTER` statements.
- [ ] Step 5: Run the migration against the local DB: `DATABASE_URL=postgres://postgres:postgres@localhost:5432/wallr cargo sqlx migrate run`.
- [ ] Step 6: Update any Rust structs and queries (e.g., in `packages/api/src/storage.rs`) to reflect the new schema.
- [ ] Step 7: **CRITICAL**: Run `DATABASE_URL=postgres://postgres:postgres@localhost:5432/wallr cargo sqlx prepare --workspace` to update the `.sqlx` cached queries.

## Troubleshooting `sqlx prepare`

If `cargo sqlx prepare` fails:
1. Verify `DATABASE_URL=postgres://postgres:postgres@localhost:5432/wallr` is explicitly passed.
2. Ensure you actually ran `cargo sqlx migrate run` before preparing.
3. Check that your Rust structs exactly match the database column types (e.g., if you have `COUNT(*)`, it maps to `i64`).

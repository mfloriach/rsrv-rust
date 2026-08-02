# Repository Guidelines

## Project Structure & Module Organization

This is a Rust (`edition = 2024`) reservation service built with Actix Web.
Application code lives in `src/`: `routes/` contains HTTP handlers, `services/`
coordinates business logic, `repositories/` contains persistence access, and
`infrastructure/` provides database, cache, queue, locking, and logging clients.
Shared models and configuration are in `src/models.rs` and `src/configuration.rs`.
The HTTP API binary is `src/bin/api.rs`; background workers are under `src/bin/`
and `src/workers/`. SQLx migrations belong in `migrations/` and use timestamped
paired `.up.sql`/`.down.sql` files. Integration tests are in `tests/`, with
common app and HTTP helpers in `tests/helper/`.

## Build, Test, and Development Commands

- `docker compose up -d` starts local dependencies (PostgreSQL, Redis, Kafka).
- `make check` runs a fast type check.
- `make test` runs the Rust test suite; integration tests use test containers.
- `make clippy` runs linting with warnings treated as errors.
- `make fmt` formats the workspace; run it before committing.
- `cargo run --bin api` starts the API after required environment variables are set.
- `make migrate` applies SQLx migrations using values from `.env`.

## Coding Style & Naming Conventions

Follow `rustfmt.toml`: Rust 2024 style, 100-column maximum width, reordered
imports, and field-init shorthand. Use `snake_case` for modules, files,
functions, and variables; `PascalCase` for types and traits; and `SCREAMING_SNAKE_CASE`
for constants. Keep route handlers thin: validate/request-map in `routes/`, put
use cases in `services/`, and isolate storage details in `repositories/`.
Use `thiserror` for domain/infrastructure errors and preserve actionable context.

## Testing Guidelines

Write async endpoint tests with `#[actix_web::test]`. Name tests for observable
behavior, e.g. `test_create_event_validation_fails`. Reuse `tests/helper` for
spawning the app, creating users, and HTTP requests. Cover successful requests,
validation failures, authorization, and persistence failures when changing an
endpoint. Run `make test` and `make clippy` before submitting changes.

## Commit & Pull Request Guidelines

Recent history uses concise imperative prefixes such as `feat:`, `fix:`,
`refactor:`, `test:`, and `add:`. Use the same format, for example
`feat: add reservation cancellation`. Keep commits focused. Pull requests should
explain the behavior change, list validation run, link the relevant issue when
available, and include request/response examples or screenshots for API or docs
changes. Do not commit secrets from `.env`.

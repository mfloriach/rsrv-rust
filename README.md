# RSV

RSV is a Rust 2024 reservation-service API built with Actix Web. It supports
user registration and sign-in, event creation and listing, seat reservations,
payment-webhook processing, and asynchronous reservation expiry.

## Prerequisites

- Rust toolchain with Cargo (edition 2024)
- Docker and Docker Compose
- Optional CLI tools: `sqlx-cli`, `cargo-watch`, `cargo-tarpaulin`, and
  `cargo-audit` for the matching Make targets

## Quick Start

1. Configure `.env`. The API reads these variables:

   ```dotenv
   POSTGRES_USERNAME=postgres
   POSTGRES_PASSWORD=postgres
   POSTGRES_DATABASE=rsv
   POSTGRES_HOST=127.0.0.1
   POSTGRES_PORT=5432
   REDIS_HOST=127.0.0.1
   REDIS_PORT=6379
   JWT_SECRET=replace-with-a-long-random-secret
   JWT_EXPIRATION_SECONDS=86400
   ```

   Keep real credentials and JWT secrets out of version control.

2. Start PostgreSQL, Redis, and Kafka:

   ```sh
   docker compose up -d db redis kafka
   ```

3. Apply the database schema and start the API:

   ```sh
   make migrate
   cargo run --bin api
   ```

   The service listens at `http://localhost:8080`. Verify it with:

   ```sh
   curl http://localhost:8080/api/v1/health
   ```

## API

Authentication endpoints are public:

- `POST /api/v1/auth/sign_up`
- `POST /api/v1/auth/sign_in`

Send the returned bearer token to access `GET`/`POST /api/v1/events/`,
`POST /api/v1/events/{event_id}/reservations`, and
`GET /api/v1/reservations/`. In debug builds, interactive OpenAPI documentation
is available at `http://localhost:8080/scalar`. Example requests are in
[`app.http`](app.http).

## Development

```sh
make check       # fast type checking
make fmt         # format with rustfmt
make clippy      # lint; warnings fail the command
make test        # unit and integration tests (uses test containers)
make coverage    # coverage report via cargo-tarpaulin
```

The source is organized by responsibility: routes in `src/routes/`, business
logic in `src/services/`, persistence in `src/repositories/`, infrastructure
adapters in `src/infrastructure/`, and background-worker code in `src/workers/`.
SQLx migrations live in `migrations/`.

## Background Workers

Kafka-backed workers are defined in `src/bin/delay_queue.rs` and
`src/bin/reservation_expiration.rs`. Run them with `cargo run --bin delay_queue`
and `cargo run --bin reservation_expiration` after configuring their Kafka and
database environment variables. Use `make create-topics` to create the two local
Kafka topics defined by `TOPIC_DELAY_QUEUE` and `TOPIC_EXPIRE_RESERVATION`.

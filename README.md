# RSV

RSV is a Rust reservation-service API for registering users, creating events,
allocating seats, and processing reservation payments. It uses PostgreSQL for
persistent state, Redis for caching and locking, and Kafka-backed workers for
reservation expiry.

## Features

- User sign-up and sign-in with password hashing and JWT authentication
- Event creation and paginated event listing
- Seat-aware reservation creation with distributed locking
- Reservation listing and payment webhook handling
- Health checks for the API, PostgreSQL, and Redis
- Background workers for delayed reservation expiration
- Interactive Scalar/OpenAPI documentation in debug builds

## Prerequisites

- Rust and Cargo with Rust 2024 edition support
- Docker and Docker Compose
- `sqlx-cli` for applying migrations

Install the migration CLI if it is not already available:

```sh
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

## Quick start

1. Create a `.env` file in the repository root:

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

2. Start the local dependencies:

   ```sh
   docker compose up -d db redis kafka
   ```

3. Apply migrations and start the API:

   ```sh
   make migrate
   cargo run --bin api
   ```

The API listens on `http://localhost:8080`. Check that it is running:

```sh
curl http://localhost:8080/api/v1/health
```

In debug builds, open the interactive API documentation at
<http://localhost:8080/scalar>.

## API usage

The authentication endpoints do not require a token:

```sh
curl -X POST http://localhost:8080/api/v1/auth/sign_up \
  -H 'Content-Type: application/json' \
  -d '{"email":"user@example.com","username":"user","password":"secret123"}'

curl -X POST http://localhost:8080/api/v1/auth/sign_in \
  -H 'Content-Type: application/json' \
  -d '{"email":"user@example.com","password":"secret123"}'
```

Use the returned JWT as a bearer token for protected endpoints:

```sh
TOKEN='paste-the-token-from-sign-in'

curl -X POST http://localhost:8080/api/v1/events/ \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"Saturday concert","description":"Main hall","capacity":100}'

curl 'http://localhost:8080/api/v1/events/?page=1&limit=20' \
  -H "Authorization: Bearer $TOKEN"
```

Available routes:

| Method | Path | Authentication |
| --- | --- | --- |
| `GET` | `/api/v1/health` | Public |
| `POST` | `/api/v1/auth/sign_up` | Public |
| `POST` | `/api/v1/auth/sign_in` | Public |
| `GET` | `/api/v1/events/` | Bearer token |
| `POST` | `/api/v1/events/` | Bearer token |
| `POST` | `/api/v1/events/{event_id}/reservations` | Bearer token |
| `GET` | `/api/v1/reservations/` | Bearer token |
| `POST` | `/api/v1/reservations/paied` | Public webhook |

More request examples are available in [`app.http`](app.http).

## Background workers

The worker binaries are separate processes:

```sh
cargo run --bin delay_queue
cargo run --bin reservation_expiration
```

`delay_queue` consumes delayed reservations and publishes expired reservations.
It requires:

```dotenv
KAFKA_BROKER_PRODUCER=127.0.0.1:29092
KAFKA_TOPIC_PRODUCER=reservation.expire
KAFKA_GROUP_ID_PRODUCER=reservation_expiring
KAFKA_BROKER_CONSUMER=127.0.0.1:29092
KAFKA_TOPIC_CONSUMER=reservation.delay
KAFKA_GROUP_ID_CONSUMER=reservation_expiring
```

`reservation_expiration` consumes expiration events and updates PostgreSQL. It
requires the database URL plus these Kafka settings:

```dotenv
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/rsv
KAFKA_BROKER=127.0.0.1:29092
KAFKA_TOPIC=reservation.expire
KAFKA_GROUP_ID=reservation_expiring
```

Create the local Kafka topics with:

```sh
make create-topics
```

Set `TOPIC_DELAY_QUEUE` and `TOPIC_EXPIRE_RESERVATION` in `.env` before running
that target. The Docker Compose Kafka broker is exposed to the host on port
`29092`.

## Development

Useful Make targets:

```sh
make check       # Type-check the project
make fmt         # Format Rust code
make clippy      # Run Clippy with warnings denied
make test        # Run unit and integration tests
make coverage    # Generate coverage with cargo-tarpaulin
make audit       # Check dependencies with cargo-audit
make migrate     # Apply SQLx migrations
```

Integration tests use test containers, so Docker must be available. SQLx query
macros also require access to the configured PostgreSQL database during a
build unless an offline query cache is provided.

The source is organized by responsibility:

- `src/routes/` — HTTP handlers and request validation
- `src/services/` — business workflows
- `src/repositories/` — PostgreSQL persistence
- `src/infrastructure/` — database, Redis, Kafka, locking, and logging adapters
- `src/workers/` — background processing support
- `migrations/` — paired SQLx up/down migrations

## Configuration reference

The API loads configuration from `.env` or the process environment:

| Variable | Purpose |
| --- | --- |
| `POSTGRES_USERNAME` | PostgreSQL username |
| `POSTGRES_PASSWORD` | PostgreSQL password |
| `POSTGRES_DATABASE` | PostgreSQL database name |
| `POSTGRES_HOST` | PostgreSQL host |
| `POSTGRES_PORT` | PostgreSQL port |
| `REDIS_HOST` | Redis host |
| `REDIS_PORT` | Redis port |
| `JWT_SECRET` | Signing secret for access tokens |
| `JWT_EXPIRATION_SECONDS` | Access-token lifetime |

Worker-specific Kafka variables are listed in the [Background workers](#background-workers)
section.

## Contributing

Before opening a change, run:

```sh
make fmt
make check
make clippy
make test
```

Keep commits focused and use the repository's conventional prefixes such as
`feat:`, `fix:`, `refactor:`, and `test:`. Do not commit `.env` files or other
secrets.

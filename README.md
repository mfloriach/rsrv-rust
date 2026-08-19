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

1. Start the local dependencies:

  ```sh
  docker compose up
  ```

2. Apply migrations and start the API:

  ```sh
  make migrate
  cargo run --bin api
  ```

3. Create the local Kafka topics with:

  ```sh
  make create-topics
  ```

In debug builds, open the interactive API documentation at
<http://localhost:8080/scalar>.

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



Set `TOPIC_DELAY_QUEUE` and `TOPIC_EXPIRE_RESERVATION` in `.env` before running
that target. The Docker Compose Kafka broker is exposed to the host on port
`29092`.

## Development

- `src/routes/` — HTTP handlers and request validation
- `src/services/` — business workflows
- `src/repositories/` — PostgreSQL persistence
- `src/infrastructure/` — database, Redis, Kafka, locking, and logging adapters
- `src/workers/` — background processing support
- `migrations/` — paired SQLx up/down migrations




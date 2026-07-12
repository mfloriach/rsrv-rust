# Hot reload
cargo watch -x check

cargo watch -x check -x test -x run

# code coverage
cargo tarpaulin --ignore-tests

# linters
cargo clippy -- -D warnings

# formatting
cargo fmt

# audit vulnabilities
cargo audit

# create migration
sqlx migrate add create_subscriptions_table

# migrate
sqlx migrate run

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

# DATABASE_URL="postgres://myuser:mysecretpassword@localhost:5432/mydatabase"

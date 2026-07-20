# Stage 1: Plan the recipe (caching layer)
FROM lukemathwalker/cargo-chef:latest-rust-1.83.0-alpine AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build the dependencies and the binary
FROM lukemathwalker/cargo-chef:latest-rust-1.83.0-alpine AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build and cache dependencies only
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy actual source code and build the real binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin zero2prod

# Stage 3: Minimal production runtime
FROM scratch
WORKDIR /app

# Copy SSL certificates for HTTPS requests if your app makes outbound API calls
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the statically compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/my_rust_app /app/zero2prod

# Use a non-root user id for security (scratch supports numeric IDs out of the box)
USER 10001

# Expose application port
EXPOSE 8080

ENTRYPOINT ["/app/zero2prod"]

# syntax=docker/dockerfile:1

#########################
# Base (Rust + cargo-chef)
#########################
FROM rust:1.96-bookworm AS chef

RUN cargo install cargo-chef --version 0.1.71

WORKDIR /app

#########################
# Planner
#########################
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

#########################
# Builder
#########################
FROM chef AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json

ENV SQLX_OFFLINE=true

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --locked --bins

RUN mkdir /dist && \
    find target/release \
        -maxdepth 1 \
        -type f \
        -executable \
        -exec cp {} /dist/ \;

#########################
# Runtime
#########################
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /dist/ /app/
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080
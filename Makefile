.PHONY: help check watch watch-run test coverage clippy fmt audit \
        migrate migrate-add db-url

DATABASE_URL ?= postgres://$(POSTGRES_USERNAME):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DATABASE)

help:
	@echo "Available targets:"
	@echo "  make check"
	@echo "  make watch"
	@echo "  make watch-run"
	@echo "  make test"
	@echo "  make coverage"
	@echo "  make clippy"
	@echo "  make fmt"
	@echo "  make audit"
	@echo "  make migrate"
	@echo "  make migrate-add NAME=<migration_name>"

check:
	cargo check

watch:
	cargo watch -x check

watch-run:
	cargo watch -x check -x test -x run

test:
	cargo test

coverage:
	cargo tarpaulin --ignore-tests

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

audit:
	cargo audit

migrate:
	DATABASE_URL="$(DATABASE_URL)" cargo sqlx migrate run

migrate-add:
ifndef NAME
	$(error Usage: make migrate-add NAME=create_users_table)
endif
	cargo sqlx migrate add $(NAME)

db-url:
	@echo $(DATABASE_URL)
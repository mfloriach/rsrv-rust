include .env

.PHONY: help check watch watch-run test coverage clippy fmt audit \
        migrate migrate-add db-url

DATABASE_URL ?= postgres://$(POSTGRES_USERNAME):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DATABASE)
KAFKA_BOOTSTRAP := localhost:9092


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
	DATABASE_URL="$(DATABASE_URL)" sqlx migrate run

migrate-add:
ifndef NAME
	$(error Usage: make migrate-add NAME=create_users_table)
endif
	sqlx migrate add $(NAME)

db-url:
	@echo $(DATABASE_URL)

create-topics:
	docker compose exec kafka \
	/opt/kafka/bin/kafka-topics.sh \
	--create \
	--if-not-exists \
	--topic $(TOPIC_DELAY_QUEUE) \
	--bootstrap-server $(KAFKA_BOOTSTRAP) \
	--partitions 3 \
	--replication-factor 1

	docker compose exec kafka \
	/opt/kafka/bin/kafka-topics.sh \
	--create \
	--if-not-exists \
	--topic $(TOPIC_EXPIRE_RESERVATION) \
	--bootstrap-server $(KAFKA_BOOTSTRAP) \
	--partitions 3 \
	--replication-factor 1

list-topics:
	docker compose exec kafka \
	/opt/kafka/bin/kafka-topics.sh \
	--bootstrap-server $(KAFKA_BOOTSTRAP) \
	--list 


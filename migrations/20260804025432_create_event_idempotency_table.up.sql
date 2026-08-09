-- Add up migration script here
CREATE TABLE idempotency (
    aggregate_id UUID NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    PRIMARY KEY (aggregate_id)
);
CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id UUID NOT NULL REFERENCES users(id),

    -- The client-provided Idempotency-Key
    key TEXT NOT NULL,

    -- Prevent using the same key for different endpoints
    endpoint TEXT NOT NULL,

    -- Optional: detect the same key being reused with different payloads
    request_hash TEXT NOT NULL,

    response_status INTEGER NOT NULL,
    response_body JSONB NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT idempotency_keys_user_key_endpoint_unique
        UNIQUE (user_id, key, endpoint)
);

CREATE INDEX idx_idempotency_keys_expires_at
    ON idempotency_keys (expires_at);

CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    /* reservation id, event id*/
    aggregate_id UUID NOT NULL, 
    event_type VARCHAR(20) NOT NULL CHECK (
            event_type IN (
                'reservation_expire'
            )
        ),
    payload JSONB NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,

    retries INT NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE INDEX idx_outbox_unpublished
    ON outbox (published_at, created_at)
    WHERE published_at IS NULL;
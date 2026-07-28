-- Add up migration script here
CREATE TABLE reservations (
    id uuid NOT NULL PRIMARY KEY,
    event_id uuid NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (
            status IN (
                'pending',
                'paied',
                'expired'
            )
        ),
    reserved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reservations_event ON reservations(event_id);
CREATE INDEX idx_reservations_user ON reservations(user_id);
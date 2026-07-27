-- Add up migration script here
CREATE TABLE seats (
    id UUID NOT NULL PRIMARY KEY,
    event_id UUID NOT NULL
        REFERENCES events(id)
        ON DELETE CASCADE,
    seat_number INTEGER NOT NULL CHECK (seat_number > 0),
    status VARCHAR(20) NOT NULL DEFAULT 'available'
        CHECK (status IN (
            'available',
            'reserved',
            'blocked'
        )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (event_id, seat_number)
);

CREATE INDEX idx_seats_event
    ON seats(event_id);

CREATE INDEX idx_seats_status
    ON seats(event_id, status);
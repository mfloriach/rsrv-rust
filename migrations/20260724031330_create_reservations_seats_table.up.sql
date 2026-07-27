-- Add up migration script here
CREATE TABLE reservation_seats (
    reservation_id UUID NOT NULL
        REFERENCES reservations(id)
        ON DELETE CASCADE,

    seat_id UUID NOT NULL
        REFERENCES seats(id)
        ON DELETE CASCADE,

    PRIMARY KEY (reservation_id, seat_id)
);

-- CREATE UNIQUE INDEX uq_active_seat
-- ON reservation_seats (seat_id)
-- WHERE EXISTS (
--     SELECT 1
--     FROM reservations r
--     WHERE r.id = reservation_seats.reservation_id
--       AND r.status IN ('pending')
-- );
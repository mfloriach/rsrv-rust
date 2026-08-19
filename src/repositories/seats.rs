use crate::models::{Available, Seat, SeatsError};
use crate::types::{EventId, ReservationId, SeatId};
use anyhow::Result;
use sqlx::{Executor, Postgres};

#[derive(Clone)]
pub struct SeatsRepository;

impl SeatsRepository {
    pub async fn create<'e>(
        &self,
        event_id: EventId,
        seats: i32,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO seats (id, event_id, seat_number)
            SELECT
                gen_random_uuid(),
                $1,
                gs
            FROM generate_series(1, $2) AS gs
            "#,
            event_id.0,
            seats
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn lock<'e>(
        &self,
        event_id: EventId,
        seats: &[Seat<Available>],
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE seats 
            SET status = 'blocked' , event_id = $1 
            WHERE id = ANY($2)
            "#,
            event_id.0,
            &seats.iter().map(|s| s.id.0).collect::<Vec<_>>()
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn unlock<'e>(
        &self,
        reservation_id: ReservationId,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        // TODO: only when state is lock
        sqlx::query!(
            r#"
                UPDATE seats s
                SET status = 'free'
                FROM reservation_seats rs
                WHERE s.id = rs.seat_id
                AND rs.reservation_id = $1 AND s.status = 'blocked';
                "#,
            reservation_id.0
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn reserved<'e>(
        &self,
        reservation_id: ReservationId,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE seats s
            SET status = 'reserved'
            FROM reservation_seats rs
            WHERE s.id = rs.seat_id
            AND rs.reservation_id = $1
            AND s.status = 'lock'
            "#,
            reservation_id.0,
        )
        .fetch_optional(executor)
        .await?;

        Ok(())
    }

    pub async fn find_available<'e>(
        &self,
        event_id: EventId,
        seats: i64,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Vec<Seat<Available>>> {
        let rows = sqlx::query!(
            r#"
            WITH event_exists AS (
                SELECT EXISTS (
                    SELECT 1
                    FROM events
                    WHERE id = $1
                ) AS exists
            )
            SELECT
                e.exists,
                s.id AS seat_id
            FROM event_exists e
            LEFT JOIN LATERAL (
                SELECT id
                FROM seats
                WHERE event_id = $1
                AND status = 'available'
                ORDER BY seat_number
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            ) s ON TRUE
            "#,
            event_id.0,
            seats
        )
        .fetch_all(executor)
        .await?;

        if rows.is_empty() || !rows[0].exists.unwrap_or(false) {
            return Err(SeatsError::EventNotFound(event_id).into());
        }

        if rows.len() < seats as usize {
            return Err(SeatsError::NotEnoughSeats.into());
        }

        Ok(rows
            .into_iter()
            .map(|row| Seat::<Available>::new(SeatId(row.seat_id.unwrap()), EventId(event_id.0)))
            .collect())
    }
}

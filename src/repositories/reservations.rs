use crate::database::Database;
use anyhow::{Ok, Result, bail};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct ReservationRepository {
    db: Database,
}

impl ReservationRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, event_id: Uuid, seats: i64) -> Result<Uuid> {
        let mut tx: Transaction<'_, Postgres> = self.db.get_connection().begin().await?;

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
            event_id,
            seats
        )
        .fetch_all(&mut *tx)
        .await?;

        if rows.is_empty() || !rows[0].exists.unwrap_or(false) {
            tx.rollback().await?;
            bail!("Event does not exit {:}", event_id);
        }

        let seats_id: Vec<Uuid> = rows.into_iter().filter_map(|r| r.seat_id).collect();

        if seats_id.len() < seats as usize {
            tx.rollback().await?;
            bail!("Not enough seats available {:}", seats);
        }

        let reservation_id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO reservations (id, event_id, user_id, status)
            VALUES ($1, $2, $3, 'pending')
            "#,
            reservation_id,
            event_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE seats 
            SET status = 'blocked' , event_id = $1 
            WHERE id = ANY($2)
            "#,
            event_id,
            &seats_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO reservation_seats (reservation_id, seat_id)
            SELECT $1, unnest($2::uuid[])
            "#,
            reservation_id,
            &seats_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(reservation_id)
    }
}

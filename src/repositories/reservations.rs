use crate::errors::AppError;
use crate::infrastructure::database::Database;
use anyhow::{Ok, Result, bail};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::{Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct Reservation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub seats: i32,
    pub reserved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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

    pub async fn expired(&self, reservation_id: Uuid) -> Result<()> {
        let mut tx: Transaction<'_, Postgres> = self.db.get_connection().begin().await?;

        sqlx::query!(
            r#"
                UPDATE seats s
                SET status = 'available'
                FROM reservation_seats rs
                WHERE s.id = rs.seat_id
                AND rs.reservation_id = $1;
                "#,
            reservation_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
                UPDATE reservations
                SET status = 'expired'
                WHERE id = $1;
                "#,
            reservation_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        page: i64,
        limit: i64,
        status: String,
    ) -> Result<Vec<Reservation>> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM reservations");

        qb.push(" WHERE user_id = ").push_bind(user_id);

        if status != "all" {
            qb.push(" AND status = ").push_bind(status.clone());
        }

        qb.push(" ORDER BY id LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind((page - 1) * limit);

        let rows = qb.build_query_as::<Reservation>().fetch_all(self.db.get_connection()).await?;

        Ok(rows)
    }

    pub async fn paid(&self, reservation_id: Uuid) -> Result<()> {
        let mut tx: Transaction<'_, Postgres> = self.db.get_connection().begin().await?;

        self.update_status(reservation_id, &mut *tx).await?;
        self.block_seats(reservation_id, &mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_status<'e, E>(&self, reservation_id: Uuid, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            Reservation,
            "UPDATE reservations SET status = 'paied' WHERE id = $1 AND status = 'pending'",
            reservation_id
        )
        .fetch_optional(executor)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "Database query failed");
            err
        })?;

        Ok(())
    }

    async fn block_seats<'e, E>(&self, reservation_id: Uuid, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query!(
            r#"
        UPDATE seats s
        SET status = 'reserved'
        FROM reservation_seats rs
        WHERE s.id = rs.seat_id
        AND rs.reservation_id = $1
        AND s.status = 'blocked'
        "#,
            reservation_id,
        )
        .fetch_optional(executor)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "Database query failed");
            err
        })?;

        Ok(())
    }
}

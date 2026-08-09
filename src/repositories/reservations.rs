use crate::types::{EventId, ReservationId, SeatId, UserId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, FromRow, PgConnection, Postgres, QueryBuilder, Type};
use uuid::Uuid;

#[derive(Serialize, Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum ReservationStatus {
    Pending,
    Paied,
    Expired,
}

#[derive(FromRow, Serialize)]
pub struct Reservation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: ReservationStatus,
    pub seats: i32,
    pub reserved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ReservationRepository;

impl ReservationRepository {
    pub async fn list<'e>(
        &self,
        user_id: &UserId,
        page: i64,
        limit: i64,
        status: &str,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Vec<Reservation>> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM reservations");

        qb.push(" WHERE user_id = ").push_bind(user_id.0);

        if status != "all" {
            qb.push(" AND status = ").push_bind(status);
        }

        qb.push(" ORDER BY id LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind((page - 1) * limit);

        let rows = qb.build_query_as::<Reservation>().fetch_all(executor).await?;

        Ok(rows)
    }

    pub async fn update_status<'e>(
        &self,
        reservation_id: ReservationId,
        status: ReservationStatus,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE reservations SET status = $2 WHERE id = $1 AND status = 'pending'",
            reservation_id.0,
            status as ReservationStatus
        )
        .fetch_optional(executor)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "Database query failed");
            err
        })?;

        Ok(())
    }

    pub async fn create(
        &self,
        event_id: EventId,
        user_id: UserId,
        seats_id: &[SeatId],
        executor: &mut PgConnection,
    ) -> Result<ReservationId> {
        let reservation_id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO reservations (id, event_id, user_id, status)
            VALUES ($1, $2, $3, 'pending')
            "#,
            reservation_id,
            event_id.0,
            user_id.0
        )
        .execute(&mut *executor)
        .await?;

        let seat_ids: Vec<Uuid> = seats_id.iter().map(|id| id.0).collect();

        sqlx::query!(
            r#"
            INSERT INTO reservation_seats (reservation_id, seat_id)
            SELECT $1, unnest($2::uuid[])
            "#,
            reservation_id,
            &seat_ids
        )
        .execute(&mut *executor)
        .await?;

        Ok(ReservationId(reservation_id))
    }
}

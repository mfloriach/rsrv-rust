use crate::hash::hash_data;
use crate::types::UserId;
use actix_web::{http::StatusCode, web::Bytes};
use anyhow::Result;
use chrono::{Duration, Utc};
use rdkafka::message::ToBytes;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::types::Json;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct IdempotencyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub endpoint: String,
    pub request_hash: String,
    pub response_status: i32,
    pub response_body: serde_json::Value,
}

impl IdempotencyRecord {
    pub fn as_response(&self) -> actix_web::HttpResponse {
        let status = StatusCode::from_u16(self.response_status as u16)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        actix_web::HttpResponse::build(status)
            .content_type("application/json")
            .json(&self.response_body)
    }
}

#[derive(Serialize, Clone)]
pub struct Request {
    pub endpoint: String,
    pub status: i32,
    pub body: serde_json::Value,
}

#[derive(Clone)]
pub struct IdempotencyRepository;

impl IdempotencyRepository {
    pub async fn create<'e>(
        &self,
        user_id: UserId,
        idempotency_key: &str,
        req: &Request,
        request_body_hash: &str,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Uuid> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO idempotency_keys (id, user_id, key, endpoint, response_body, response_status, request_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            user_id.0,
            idempotency_key,
            req.endpoint,
            req.body,
            req.status,
            request_body_hash,
            Utc::now() + Duration::minutes(10)
        )
        .execute(executor)
        .await?;

        Ok(id)
    }

    pub async fn find_by_key<'e>(
        &self,
        idempotency_key: &str,
        executor: impl Executor<'e, Database = Postgres>,
    ) -> Result<Option<IdempotencyRecord>> {
        let row = sqlx::query_as!(
            IdempotencyRecord,
            r#"
            SELECT
                id,
                user_id,
                key,
                endpoint,
                request_hash,
                response_status,
                response_body AS "response_body!"
            FROM idempotency_keys
            WHERE key = $1
            "#,
            idempotency_key
        )
        .fetch_optional(executor)
        .await?;

        Ok(row)
    }
}

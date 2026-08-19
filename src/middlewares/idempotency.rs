use crate::errors::AppError;
use crate::hash::hash_data;
use crate::infrastructure::server::AppState;
use crate::repositories::Request;
use crate::types::UserId;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::{BoxBody, to_bytes},
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};
use anyhow::{Context, Result};
use futures_util::StreamExt;

pub async fn idempotency(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let state = req
        .app_data::<web::Data<AppState>>()
        .cloned()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("AppState not found")))?;

    let idempotency_key = req
        .headers()
        .get("Idempotency-Key")
        .ok_or_else(|| AppError::BadRequest("Missing Idempotency-Key header".into()))?
        .to_str()
        .map_err(|_| AppError::InvalidIdempotencyKey)?
        .to_owned();

    let user_id =
        req.extensions().get::<UserId>().copied().ok_or_else(|| AppError::Unauthorized)?;

    let existing_response = state
        .repositories
        .idempotency
        .find_by_key(&idempotency_key, state.db_pool.get_connection())
        .await
        .map_err(|_| AppError::InvalidIdempotencyKey)?;

    let request_body_hash =
        req.body_hash().await.map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(record) = existing_response {
        let response = record.as_response();

        if record.request_hash != request_body_hash {
            return Err(AppError::IdempotencyConflict.into());
        }

        return Ok(req.into_response(response));
    }

    let res = next.call(req).await?;
    let (req_from_service, response_from_service) = res.into_parts();

    let path = req_from_service.path().to_owned();
    let status = response_from_service.status();

    let body = to_bytes(response_from_service.into_body())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response_body: serde_json::Value =
        serde_json::from_slice(&body).map_err(actix_web::error::ErrorInternalServerError)?;

    let request =
        Request { endpoint: path, status: status.as_u16() as i32, body: response_body.clone() };

    state
        .repositories
        .idempotency
        .create(
            user_id,
            &idempotency_key,
            &request,
            &request_body_hash,
            state.db_pool.get_connection(),
        )
        .await
        .map_err(AppError::Internal)?;

    Ok(ServiceResponse::new(
        req_from_service,
        HttpResponse::build(status).json(response_body).map_into_boxed_body(),
    ))
}

trait RequestBodyHash {
    async fn body_hash(&mut self) -> Result<String>;
}

impl RequestBodyHash for ServiceRequest {
    async fn body_hash(&mut self) -> Result<String> {
        let mut body = self.take_payload();

        let mut bytes = Vec::new();

        while let Some(chunk) = body.next().await {
            let chunk = chunk.context("failed to read request body")?;
            bytes.extend_from_slice(&chunk);
        }

        let request_body_hash = hash_data(&bytes);

        // Restore body so downstream handlers can read it
        self.set_payload(bytes.into());

        Ok(request_body_hash)
    }
}

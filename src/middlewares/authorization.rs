use crate::jwt::verify_token;
use actix_web::HttpMessage;
use actix_web::{
    Error, HttpResponse,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn auth(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    match req.headers().get("Authorization") {
        Some(header_value) => {
            let header_value = header_value.to_str().unwrap();
            if header_value.starts_with("Bearer ") {
                let token = header_value.split(" ").collect::<Vec<&str>>()[1];
                match verify_token(token) {
                    Ok(sub) => {
                        req.extensions_mut().insert(UserId(sub));
                        next.call(req).await
                    }
                    Err(_) => Ok(req.into_response(HttpResponse::Unauthorized().finish())),
                }
            } else {
                Ok(req.into_response(HttpResponse::Unauthorized().body("Invalid Token")))
            }
        }
        None => Ok(req.into_response(HttpResponse::Unauthorized().body("No Token"))),
    }
}

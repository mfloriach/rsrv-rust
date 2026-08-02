pub mod events;
pub mod health;
pub mod login;
pub mod reservations;
use crate::middlewares::auth;
use actix_web::web;
pub use events::{CreateEventRequest, Meta as EventListQuery, create_event, get_events};
pub use health::health_check;
pub use login::{SignInRequest, SignInResponse, SignUpRequest, sign_in, sign_up};
pub use reservations::{
    CreateReservationRequest, Meta as ReservationListQuery, PaymentIntentRequest, PaymentStatus,
    create_reservation, get_reservations, paid_reservation_webhook,
};
use serde::{Deserialize, Serialize};

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1/auth").service(sign_in).service(sign_up));
}

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .wrap(actix_web::middleware::from_fn(auth))
            .service(
                web::scope("/events")
                    .service(create_event)
                    .service(get_events)
                    .service(create_reservation),
            )
            .service(web::scope("/reservations").service(get_reservations)),
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List<D, M> {
    pub meta: M,
    pub data: Vec<D>,
}

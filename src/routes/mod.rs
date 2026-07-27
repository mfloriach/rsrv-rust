mod events;
mod health;
mod login;
mod reservations;
use crate::routes::events::get_events;
use actix_web::web;
pub use events::create_event;
pub use health::health_check;
pub use login::{SignInRequest, SignInResponse, sign_in, sign_up};
pub use reservations::{create_reservation, get_reservations, paid_reservation_webhook};
use serde::{Deserialize, Serialize};

pub fn reservations_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/reservations").route("/", web::get().to(get_reservations)));
}

pub fn events_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/events")
            .route("/", web::post().to(create_event))
            .route("/", web::get().to(get_events))
            .route("/{event_id}/reservations", web::post().to(create_reservation)),
    );
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/auth")
            .route("/sign_in", web::post().to(sign_in))
            .route("/sign_up", web::post().to(sign_up)),
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List<D, M> {
    pub meta: M,
    pub data: Vec<D>,
}

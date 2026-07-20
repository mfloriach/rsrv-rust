mod events;
mod health;
mod login;
mod reservations;
use actix_web::web;
pub use events::create_event;
pub use health::health_check;
pub use login::{SignInRequest, SignInResponse, hello, sign_in, sign_up};
pub use reservations::{create_reservation, get_reservations};
use serde::{Deserialize, Serialize};

use crate::routes::events::get_events;

pub fn posts_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/posts").route("/", web::get().to(hello)));
}

pub fn reservations_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/reservations")
            .route("/", web::post().to(create_reservation))
            .route("/", web::get().to(get_reservations)),
    );
}

pub fn events_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/events")
            .route("/", web::post().to(create_event))
            .route("/", web::get().to(get_events)),
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List<D, M> {
    pub meta: M,
    pub data: Vec<D>,
}

mod events;
mod health;
mod login;
mod reservations;

use actix_web::web;

pub use events::create_event;
pub use health::health_check;
pub use login::{hello, sign_in, sign_up};
pub use reservations::create_reservation;

pub fn posts_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/posts").route("/", web::get().to(hello)));
}

pub fn reservations_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/reservations").route("/", web::post().to(create_reservation)));
}

pub fn events_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/events").route("/", web::post().to(create_event)));
}

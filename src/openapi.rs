use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RSV API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Reservation service HTTP API"
    ),
    paths(
        crate::routes::login::sign_in,
        crate::routes::login::sign_up,
        crate::routes::events::get_events,
        crate::routes::events::create_event,
        crate::routes::reservations::create_reservation,
        crate::routes::reservations::get_reservations,
        crate::routes::reservations::paid_reservation_webhook,
        crate::routes::health::health_check
    ),
    components(schemas(
        crate::routes::SignInRequest,
        crate::routes::SignUpRequest,
        crate::routes::SignInResponse,
        crate::routes::CreateEventRequest,
        crate::routes::EventListQuery,
        crate::routes::CreateReservationRequest,
        crate::routes::ReservationListQuery,
        crate::routes::PaymentIntentRequest,
        crate::routes::PaymentStatus
    )),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "events", description = "Event management"),
        (name = "reservations", description = "Reservation management"),
        (name = "health", description = "Service health")
    )
)]
pub struct ApiDoc;

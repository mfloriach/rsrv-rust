use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,actix_web=info,tracing_actix_web=info,sqlx=debug",
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .try_init();
}

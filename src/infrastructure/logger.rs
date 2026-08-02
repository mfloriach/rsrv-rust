use std::str::FromStr;
use tracing_subscriber::{Layer, Registry, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_LOG_FILTER: &str = "info,actix_web=info,tracing_actix_web=info,sqlx=warn";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

impl FromStr for LogFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "text" | "pretty" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err("log format must be either text or json"),
        }
    }
}

impl LogFormat {
    fn from_environment() -> Self {
        std::env::var("LOG_FORMAT").ok().and_then(|value| value.parse().ok()).unwrap_or_default()
    }
}

pub fn init_logger() {
    init_logger_with_format(LogFormat::from_environment());
}

pub fn init_logger_with_format(format: LogFormat) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(DEFAULT_LOG_FILTER));

    let result = match format {
        LogFormat::Text => {
            tracing_subscriber::registry().with(text_layer()).with(filter).try_init()
        }
        LogFormat::Json => {
            tracing_subscriber::registry().with(json_layer()).with(filter).try_init()
        }
    };

    let _ = result;
}

fn text_layer() -> impl Layer<Registry> {
    tracing_subscriber::fmt::layer()
        .pretty()
        .with_target(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
}

fn json_layer() -> impl Layer<Registry> {
    tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
}

#[cfg(test)]
mod tests {
    use super::LogFormat;
    use std::str::FromStr;

    #[test]
    fn parses_supported_log_formats() {
        assert_eq!(LogFormat::from_str("text"), Ok(LogFormat::Text));
        assert_eq!(LogFormat::from_str("pretty"), Ok(LogFormat::Text));
        assert_eq!(LogFormat::from_str("JSON"), Ok(LogFormat::Json));
    }

    #[test]
    fn rejects_unknown_log_formats() {
        assert!(LogFormat::from_str("yaml").is_err());
    }
}

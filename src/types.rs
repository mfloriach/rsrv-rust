use chrono::DateTime;
use chrono::Utc;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
        #[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
        #[display("{}", _0)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
    };
}

uuid_id!(EventId);
uuid_id!(SeatId);
uuid_id!(ReservationId);
uuid_id!(PaymentId);

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(utoipa::ToSchema))]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ReservationExpired {
    pub reservation_id: ReservationId,
    pub expired_at: DateTime<Utc>,
}

impl ReservationExpired {
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

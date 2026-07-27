use crate::kafka::{ReservationExpired, create_consumer};
use anyhow::Result;
use futures_util::StreamExt;
use rdkafka::Message;
use rdkafka::consumer::Consumer;
use std::thread::sleep;
use std::time::Duration;

pub async fn reservation_expiration_worker() -> Result<()> {
    sleep(Duration::from_secs(10));
    let consumer = create_consumer().await;
    consumer.subscribe(&["reservation-events"])?;
    println!("starting worker");

    let mut stream = consumer.stream();

    while let Some(message) = stream.next().await {
        let message = message?;

        if let Some(Ok(payload)) = message.payload_view::<str>() {
            let event: ReservationExpired = serde_json::from_slice(payload.as_bytes())?;
            println!("{:?}", event);
        }
    }

    Ok(())
}

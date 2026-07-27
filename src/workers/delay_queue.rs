use futures_util::StreamExt;
use rdkafka::Message;
use rdkafka::consumer::Consumer;
use std::thread::sleep;
use std::time::Duration;

struct MessagePrinter {}

impl MessagePrinter {
    fn new() -> Box<Self> {
        Box::new(MessagePrinter {})
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> Result<()> {
        println!("Key: {}", String::from_utf8_lossy(key));
        println!("Payload: {}", String::from_utf8_lossy(payload));

        Ok(())
    }
}

pub async fn reservation_expiration_worker() -> anyhow::Result<()> {
    sleep(Duration::from_secs(10));
    Rece
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

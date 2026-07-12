// use rdkafka::config::ClientConfig;
// use rdkafka::producer::{FutureProducer};

// pub struct QueueKafka {
//     producer: FutureProducer
// }

// impl QueueKafka {
//     pub fn new(address: &str) -> Self {
//         let producer: FutureProducer = ClientConfig::new()
//             .set("bootstrap.servers", address)
//             .set("message.timeout.ms", "5000")
//             .create()
//             .expect("Producer creation error");

//         Self { producer: producer }
//     }
// }

// pub async fn send_to_queue() {
//     let producer: FutureProducer = ClientConfig::new()
//         .set("bootstrap.servers", "localhost:9092")
//         .set("message.timeout.ms", "5000")
//         .create()
//         .expect("Producer creation error");

//     let record = FutureRecord::to("my-topic")
//         .key("user_123")
//         .payload("Hello from Rust!");

//     match producer.send(record, Duration::from_secs(0)).await {
//         Ok(delivery) => println!("Sent successfully: {:?}", delivery),
//         Err((e, _)) => eprintln!("Delivery failed: {:?}", e),
//     }
// }

// pub async fn consume_from_queue() {
//     let consumer: StreamConsumer = ClientConfig::new()
//         .set("group.id", "my_group")
//         .set("bootstrap.servers", "localhost:9092")
//         .set("auto.offset.reset", "earliest")
//         .create()
//         .expect("Consumer creation error");

//     consumer.subscribe(&["my-topic"]).expect("Can't subscribe to specified topic");

//     let mut message_stream = consumer.stream();

//     while let Some(message) = message_stream.next().await {
//         match message {
//             Ok(m) => {
//                 if let Some(payload) = m.payload_view::<str>() {
//                     match payload {
//                         Ok(s) => println!("Received message: {}", s),
//                         Err(e) => eprintln!("Error while deserializing message payload: {:?}", e),
//                     }
//                 }
//             }
//             Err(e) => eprintln!("Kafka error: {:?}", e),
//         }
//     }
// }

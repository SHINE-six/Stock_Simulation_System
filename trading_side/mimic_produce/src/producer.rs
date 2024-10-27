use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;
use crate::models::Order;

pub struct StockOrderProducer {
    producer: FutureProducer,
    topic: String,
}

impl StockOrderProducer {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .expect("Producer creation failed");

            StockOrderProducer {
                producer,
                topic: topic.to_string(),
            }
    }

    pub async fn send_message(&self, message: Order) {
        let payload = serde_json::to_string(&message).expect("Failed to serialize message");

        self.producer
            .send(
                FutureRecord::to(&self.topic)
                    .payload(&payload)
                    .key(&message.id),
                Timeout::After(Duration::from_secs(0)),
                )
            .await
            .unwrap();
    }
}
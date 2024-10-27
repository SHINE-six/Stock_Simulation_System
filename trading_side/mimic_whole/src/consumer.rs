use tokio::sync::broadcast;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use futures::StreamExt;
use crate::models::Stock;

pub async fn start_kafka_consumer(brokers: &str, topic: &str, tx: broadcast::Sender<Stock>) {
    let consumer: StreamConsumer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", "stock-consumer-group")
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Subscribing to topic failed");

    let mut message_stream = consumer.stream();

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    if let Ok(stock) = serde_json::from_slice::<Stock>(payload) {
                        let _ = tx.send(stock);
                    } else {
                        eprintln!("Failed to deserialize message");
                    }
                }
            }
            Err(e) => {
                eprintln!("Kafka error: {}", e);
            }
        }
    }
}

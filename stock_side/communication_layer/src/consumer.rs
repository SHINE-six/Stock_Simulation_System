use common::models::Order;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::{BorrowedMessage, Message};
use tokio_stream::StreamExt;
use tokio::sync::mpsc::Sender;
use serde_json::from_slice;

pub struct OrderConsumer {
    consumer: StreamConsumer,
}

impl OrderConsumer {
    pub fn new(
        brokers: &str, 
        topic: &str, 
        group_id: &str,
    ) -> Self {
        println!("OrderConsumer: Connecting to Kafka: {}", brokers);

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("auto.offset.reset", "latest")  // read from the end of the topic
            .set("fetch.min.bytes", "1")  // fetch messages as soon as they arrive
            .set("fetch.wait.max.ms", "100")  // wait for at most 100ms for messages
            .create()
            .expect("OrderConsumer: Consumer creation failed");

        consumer
            .subscribe(&[topic])
            .expect("OrderConsumer: Subscribing to topic failed");

        println!("OrderConsumer: Connected to Kafka");

        Self {
            consumer,
        }
    }

    pub async fn consume_messages(&self, order_sender: Sender<Order>) {
        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let Some(order) = parse_order(&message) {
                        if let Err(e) = order_sender.send(order).await {
                            eprintln!("Failed to send order: {}", e);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Panda error: {}", error);
                }
            }
        }
    }
}

fn parse_order(message: &BorrowedMessage) -> Option<Order> {
    if let Some(payload) = message.payload() {
        match from_slice::<Order>(payload) {
            Ok(order) => Some(order),
            Err(e) => {
                eprintln!("Failed to deserialize order: {}", e);
                None
            }
        }
    } else {
        eprintln!("Received message with empty payload");
        None
    }
}

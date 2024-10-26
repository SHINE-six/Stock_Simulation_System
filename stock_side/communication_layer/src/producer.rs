use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use common::models::Stock;

pub struct StockProducer {
    producer: FutureProducer,
}

impl StockProducer {
    pub fn new(brokers: &str) -> Self {
        println!("StockProducer: Connecting to Kafka: {}", brokers);

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")  // 5 seconds
            .create()
            .expect("StockProducer: Producer creation failed");

        println!("StockProducer: Connected to Kafka");

        Self {
            producer,
        }
    }

    pub async fn produce_stock(&self, stock: Stock, topic: &str) {
        let stock_json = serde_json::to_string(&stock).expect("Failed to serialize stock");

        let record = FutureRecord::to(topic)
            .key(&stock.symbol)
            .payload(&stock_json);

        match self.producer.send(record, Timeout::Never).await {
            Ok(_) => {()}
            Err((e, _)) => {
                eprintln!("Failed to produce stock: {}", e);
            }
        }
    }

    pub async fn produce_stocks(&self, stocks: Vec<Stock>, topic: &str) {
        for stock in stocks {
            self.produce_stock(stock, topic).await;
        }
    }
}
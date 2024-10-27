use crate::models::{Order, OrderType};

use redis::Commands;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;
use chrono::Utc;
use serde_json;
use tokio::time::{Duration, sleep};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration as StdDuration;

pub struct OrderProducer {
    producer: FutureProducer,
    topic: String,
}
impl OrderProducer {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .expect("Producer creation failed");

        OrderProducer {
            producer,
            topic: topic.to_string(),
        }
    }

    pub async fn start_order_producer(&self) {
        let redis_url = "redis://localhost:6379";

        let client = redis::Client::open(redis_url).unwrap();
        let mut con = client.get_connection().unwrap();
        let stock_symbols_string: Vec<(String, String)>= con.hgetall("stocks:prices").unwrap();

        let symbol_price: Vec<(String, f64)> = stock_symbols_string.iter().map(|(k, v)| (k.clone(), v.parse::<f64>().unwrap())).collect();


        let producer = self.producer.clone();
        let topic = self.topic.clone();
        tokio::spawn({
            let symbol_price = symbol_price.clone();
            async move {
                let seed: [u8; 32] = rand::random();
                let mut rng = StdRng::from_seed(seed);
                loop {
                    let order = generate_random_order(&symbol_price[rng.gen_range(0..symbol_price.len())]);
                    println!("Generated order: {:?}", order);
                    send_message(&producer, &topic, order).await;

                    sleep(Duration::from_secs(1)).await;
                }
            }
        }).await.unwrap();
    }

    pub async fn produce_custom_order(&self, &json_body: serde_json::Value) -> String {
        let order: Order = serde_json::from_value(json_body).unwrap();
        order.id = Uuid::new_v4().to_string();
        order.timestamp = Utc::now().timestamp() as u64;
        order.partial_fill = true;

        let producer = self.producer.clone();
        let topic = self.topic.clone();

        tokio::spawn(async move {
            send_message(&producer, &topic, order).await;
        }).await.unwrap();

        serde_json::to_string(&order).unwrap()
    }
}

async fn send_message(producer: &FutureProducer, topic: &str, message: Order) {
    let payload = serde_json::to_string(&message).expect("Failed to serialize message");

    producer
        .send(
            FutureRecord::to(topic)
                .payload(&payload)
                .key(&message.id),
            Timeout::After(StdDuration::from_secs(0)),
            )
        .await
        .unwrap();
}

fn generate_random_order(symbol_price: &(String, f64)) -> Order {
    // let stock_symbols = vec!["AAPL", "GOOGL", "AMZN", "MSFT", "TSLA"];

    let order_types = vec![OrderType::Buy, OrderType::Sell];
    let mut rng = rand::thread_rng();

    Order {
        id: Uuid::new_v4().to_string(),
        stock_symbol: symbol_price.0.clone(),
        order_type: order_types[rng.gen_range(0..order_types.len())].clone(),
        quantity: rng.gen_range(5..150),
        price: symbol_price.1.clone() * (1.0 + (rng.gen_range(-15..15) as f64 / 100.0)),    // Random price between -15% and +15% of the current price
        timestamp: Utc::now().timestamp() as u64,
        partial_fill: true,
    }
}
use crate::models::{Order, OrderType};

use redis::Commands;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;
use chrono::Utc;
use serde_json::{Map, Value};
use tokio::time::{Duration, sleep};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration as StdDuration;

#[derive(Clone)]
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

                    sleep(Duration::from_millis(300)).await;
                }
            }
        }).await.unwrap();
    }

    pub async fn produce_custom_order(&self, json_body: &Value) -> String {
        let mut map: Map<String, Value> = serde_json::from_value(json_body.clone()).unwrap_or_default();

        // Populate missing fields
        map.insert("id".to_string(), Value::String(Uuid::new_v4().to_string()));
        map.insert("timestamp".to_string(), Value::Number(Utc::now().timestamp().into()));
        map.insert("partial_fill".to_string(), Value::Bool(true));

        // Change quanity type from str to u32
        if let Some(quantity_str) = map.get("quantity").and_then(|v| v.as_str()) {
            if let Ok(quantity_u32) = quantity_str.parse::<u32>() {
                map.insert("quantity".to_string(), Value::Number(serde_json::Number::from(quantity_u32)));
            }
        }

        // Change price type from str to f64
        if let Some(price_str) = map.get("price").and_then(|v| v.as_str()) {
            if let Ok(price_f64) = price_str.parse::<f64>() {
                map.insert("price".to_string(), Value::Number(serde_json::Number::from_f64(price_f64).unwrap()));
            }
        }

        // Convert map back to Value
        let updated_json_body = Value::Object(map);
    
        // Deserialize into Order
        let order: Order = serde_json::from_value(updated_json_body).unwrap();
    
        let producer = self.producer.clone();
        let topic = self.topic.clone();

        tokio::spawn({
            let order = order.clone();
            async move {
                send_message(&producer, &topic, order).await;
        }}).await.unwrap();

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
        // TODO: limit_order: research possibilities
    }
}
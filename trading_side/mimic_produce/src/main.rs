mod producer;
mod models;

use crate::producer::StockOrderProducer;
use crate::models::{Order, OrderType};
use redis::Commands;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;
use chrono::Utc;
use tokio::time::{Duration, sleep};
// use std::thread;
// use std::time::Duration;

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

#[tokio::main]
async fn main() {
    let brokers = "localhost:19092";
    let topic = "broker-orders";
    let redis_url = "redis://localhost:6379";

    let client = redis::Client::open(redis_url).unwrap();
    let mut con = client.get_connection().unwrap();
    let stock_symbols_string: Vec<(String, String)>= con.hgetall("stocks:prices").unwrap();

    let symbol_price: Vec<(String, f64)> = stock_symbols_string.iter().map(|(k, v)| (k.clone(), v.parse::<f64>().unwrap())).collect();
    let producer = StockOrderProducer::new(brokers, topic);

    
    tokio::spawn({
        let symbol_price = symbol_price.clone();
        async move {
            let seed: [u8; 32] = rand::random();
            let mut rng = StdRng::from_seed(seed);
            loop {
                let order = generate_random_order(&symbol_price[rng.gen_range(0..symbol_price.len())]);
                println!("Generated order: {:?}", order);
                producer.send_message(order).await;

                sleep(Duration::from_secs(1)).await;
            }
        }
    }).await.unwrap();
}
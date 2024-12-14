use crate::algorithm;

use common::models::{Order, Stock, Trade};
use redis::{aio, AsyncCommands, RedisResult}; // RedisResult: Result type for Redis commands
use serde_json::from_str; // Deserialize JSON string to struct
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use rand::Rng;
use tokio::time::{sleep, Duration}; // to remove, check for computer resources used by this function

pub struct MarketDataGenrator {
    stocks: Vec<Stock>,
    client: redis::Client,
    // redis_conn: Arc<Mutex<aio::MultiplexedConnection>>,
}

impl MarketDataGenrator {
    pub async fn new(redis_url: &str) -> Self {
        println!("MarketDataGenrator: Connecting to Redis: {}", redis_url);
        let client = redis::Client::open(redis_url).unwrap();
        let mut redis_conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("MarketDataGenrator: Failed to connect to Redis");

        println!("MarketDataGenrator: Connected to Redis");

        // Fetch the initial list of stocks from Redis
        let stocks = fetch_stocks(&mut redis_conn).await;

        Self {
            stocks,
            client,
            // redis_conn: Arc::new(Mutex::new(redis_conn)),
        }
    }

    pub async fn start(&self, mut mdg_receiver: Receiver<Trade>, stock_sender: Sender<Stock>) {
        let redis_conn_task1 = &self.client.get_multiplexed_async_connection().await.expect("Failed to get redis connection");
        let redis_conn_task2 = &self.client.get_multiplexed_async_connection().await.expect("Failed to get redis connection");
        let redis_conn_task3 = &self.client.get_multiplexed_async_connection().await.expect("Failed to get redis connection");

        // Send inital stock prices
        for stock in &self.stocks {
            if let Err(e) = stock_sender.send(stock.clone()).await {
                eprintln!("Failed to send stock via stock_sender: {}", e);
            }
        }

        // // Clone the data to move into the async task
        // let redis_conn_clone = self.redis_conn.clone();

        // Task 1: Passive Update
        tokio::spawn({
            let mut redis_conn_clone = redis_conn_task1.clone();
            let stock_sender_clone = stock_sender.clone();
            async move {
                loop {
                    println!("First task");
                    let stocks = fetch_stocks(&mut redis_conn_clone).await;
                    let orders = fetch_orders(&mut redis_conn_clone).await.unwrap();
                    passive_update_stock_price(&mut redis_conn_clone, stocks, orders, stock_sender_clone.clone()).await;
                    sleep(Duration::from_secs(1)).await; // To remove, check for computer resources used by this function
                }
            }
        });

        // Task 2: Active Update of trade from channel
        tokio::spawn({
            let mut redis_conn_clone = redis_conn_task2.clone();
            let stock_sender_clone = stock_sender.clone();
            async move {
                while let Some(trade_received) = mdg_receiver.recv().await {
                    println!("Second Task");
                    // Update the stock price based on the trade
                    let stocks = fetch_stocks(&mut redis_conn_clone).await;
                    active_update_stock_price(&mut redis_conn_clone, stocks, &trade_received, stock_sender_clone.clone()).await;
                }
            }
        });

        // Task 3: Sector Performance
        // tokio::spawn({
        //     let mut redis_conn_clone = redis_conn_task3.clone();
        //     async move {
                // let mut conn = redis_conn_clone.lock().await;
                let mut redis_conn_clone = redis_conn_task3.clone();
                let stock_sector_string: RedisResult<Vec<(String, String)>> = redis_conn_clone.hgetall("stocks:sector").await;

                // let stock_sector_string: RedisResult<Vec<(String, String)>> = redis_conn_clone.hgetall("stocks:sector").await;
                let mut sector_hash: HashMap<String, f64> = HashMap::new();
                
                match stock_sector_string {
                    Ok(stock_sector_result) => {
                        loop {
                            println!("Third Task");
                            let start_sector_hash = sector_hash.clone();
                            let mut cumulate_sector_hash: HashMap<String, Vec<f64>> = HashMap::new();
                            for (stock, sector) in stock_sector_result.clone() {
                                // let mut conn = redis_conn_clone.lock().await;
                                let price = redis_conn_clone.hget("stocks:prices", stock).await.unwrap();
                                // drop(conn);
                                cumulate_sector_hash.entry(sector).or_insert(Vec::new()).push(price);
                            }
                            for (sector, prices) in cumulate_sector_hash.clone() {
                                let sum: f64 = prices.iter().sum();
                                let average: f64 = sum / prices.len() as f64;
                                sector_hash.insert(sector, average);
                            }

                            // Compare start_sector_hash with sector_hash, if sector_hash is different from start_sector_hash by 10%, then update the stock price of all the other stocks in the sector
                                // Iterate over the start_sector_hash and sector_hash, and compare the values
                                // If the difference is more than 10%, then update the stock price of all the other stocks in the sector
                            if start_sector_hash.is_empty() {
                                continue;
                            }
                            for (sector, price) in sector_hash.clone() {
                                let start_price = start_sector_hash.get(&sector).unwrap();
                                if (price - start_price).abs() / start_price > 0.0002564 {
                                    // let mut conn = redis_conn_clone.lock().await;
                                    let stock_sector_string: Vec<(String, String)> = redis_conn_clone.hgetall("stocks:sector").await.expect("Failed to fetch sector");
                                    // drop(conn);
                                    
                                    // Update the stock price of all the other stocks in the sector
                                    let stocks = fetch_stocks(&mut redis_conn_clone).await;

                                    for (stock, sector_db) in stock_sector_string {
                                        if sector_db == sector {
                                            let stock: &Stock = stocks.iter().find(|s| s.symbol == stock).expect("Stock not found");
                                            let new_price = match price > *start_price {
                                                true => {
                                                    stock.price * rand::thread_rng().gen_range(1.0..1.1)
                                                }
                                                false => {
                                                    stock.price * rand::thread_rng().gen_range(0.9..1.0)
                                                }
                                            };
                                            update_stock_price(&mut redis_conn_clone, stock, new_price, stock_sender.clone()).await.unwrap();
                                        }
                                    }
                                }
                            }
    
                            sleep(Duration::from_secs(60)).await; // Every 60 seconds
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch sector: {}", e);
                    }
                }
        //     }
        // });
    }
}

async fn passive_update_stock_price(
    redis_conn: &mut aio::MultiplexedConnection,
    latest_stock: Vec<Stock>,
    orders: HashMap<String, (Vec<Order>, Vec<Order>)>,
    stock_sender: Sender<Stock>,
) {        
    for order in orders {
        // Get the latest stock price
        let stock: &Stock = latest_stock.iter().find(|s| s.symbol == order.0).expect("Stock not found");

        let mut multiplier_vec: Vec<f64> = Vec::new();
        // Algorithm 1: buy vs sell demand in term of share (number of buy/sell * their no. of share)
        algorithm::algorithm_1(&order, &mut multiplier_vec);
        // Algorithm 2: Order Imbalance: buy vs sell demand in term of order amount ((total buy orders - total sell orders) / (total buy orders + total sell orders))
        algorithm::algorithm_2(&order, &mut multiplier_vec);
        // Algorithm 3: Cumulative Order Book Depth: buy vs sell demand in term of price level (total share of buy/sell orders @ price level)
        algorithm::algorithm_3(&order, &mut multiplier_vec);
        // Algorithm 4: Market Pressure from Large Orders (Iceberg Effect): buy vs sell in term of largest share of buy/sell orders that deviate from the current stock price
        algorithm::algorithm_4(&order, &mut multiplier_vec);
        // Algorithm 5: Order Flow Momentum: check the recent time window of buy/sell orders, and if there is a momentum in the order flow, then increase/decrease the stock price perspectively
        algorithm::algorithm_5(&order, &mut multiplier_vec);
        // Algorithm 6: Order Book Skewness: skew of the order book (buy/sell orders) in term of price
        algorithm::algorithm_6(&order, stock.price, &mut multiplier_vec);

        // calculate the average of the multiplier_vec
        let sum: f64 = multiplier_vec.iter().sum();
        let multiplier: f64 = sum / multiplier_vec.len() as f64;

        // Update the stock price
        let new_price = stock.price * multiplier;
        update_stock_price(redis_conn, stock, new_price, stock_sender.clone()).await.unwrap();
    }
}

async fn active_update_stock_price(
    redis_conn: &mut aio::MultiplexedConnection,
    stocks: Vec<Stock>,
    trade_received: &Trade,
    stock_sender: Sender<Stock>,
) {
    let stock: &Stock = stocks.iter().find(|s| s.symbol == trade_received.stock_symbol).expect("Stock not found");

    let new_price = algorithm::algorithm_trade(&trade_received, stock.price);

    update_stock_price(redis_conn, stock, new_price, stock_sender.clone()).await.unwrap();
}

async fn update_stock_price(
    redis_conn: &mut aio::MultiplexedConnection,
    stock: &Stock,
    new_price: f64,
    stock_sender: Sender<Stock>,
) -> RedisResult<()> {
    let updated_stock = Stock {
        symbol: stock.symbol.clone(),
        price: ((new_price * 10000.0).round() / 10000.0).max(0.0001),
    };

    // Send the updated stock price to redis
    // let mut conn = redis_conn.lock().await;
    redis_conn.hset("stocks:prices", &updated_stock.symbol, updated_stock.price).await?;

    // Send the updated stock price to channel
    if let Err(e) = stock_sender.send(updated_stock.clone()).await {
        eprintln!("Failed to send stock via stock_sender: {}", e);
    }

    Ok(())
}

async fn fetch_stocks(redis_conn: &mut aio::MultiplexedConnection) -> Vec<Stock> {
    // let mut conn = redis_conn.lock().await;
    let stocks_string: RedisResult<Vec<(String, String)>> = redis_conn.hgetall("stocks:prices").await;

    let mut stocks: Vec<Stock> = Vec::new();
    match stocks_string {
        Ok(stocks_result) => {
            for (symbol, price) in stocks_result {
                let stock = Stock {
                    symbol,
                    price: price.parse().unwrap(),
                };
                stocks.push(stock);
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch stocks: {}", e);
        }
    }

    stocks
}

// The return type of RedisResult<Vec<Order>, Vec<Order>> means there is a collection of each stock symbol's buy and sell orders
async fn fetch_orders(
    redis_conn: &mut aio::MultiplexedConnection,
) -> RedisResult<HashMap<String, (Vec<Order>, Vec<Order>)>> {
    // let mut conn = redis_conn.lock().await;
    let keys: Vec<String> = {
        let mut orders_book_iter: redis::AsyncIter<String> = redis_conn.scan_match("order_book:*").await.expect("Failed to scan order book");

        let mut keys = Vec::new();
        while let Some(key) = orders_book_iter.next_item().await {
            keys.push(key);
        }

        keys
    };

    let mut all_stock_orders: HashMap<String, (Vec<Order>, Vec<Order>)> = HashMap::new();

    for key in keys {
        // Use a block to isolate the mutable borrow
        let (buy_orders_string, sell_orders_string): (Option<String>, Option<String>) = {
            redis_conn.hget(&key, &["buy_orders", "sell_orders"]).await?
        };

        // Deserialize the buy and sell orders into Vec<Order>
        let buy_orders: Vec<Order> = match buy_orders_string {
            Some(buy_orders) => from_str(&buy_orders).expect("Failed to deserialize buy orders"),
            None => Vec::new(),
        };

        let sell_orders: Vec<Order> = match sell_orders_string {
            Some(sell_orders) => from_str(&sell_orders).expect("Failed to deserialize sell orders"),
            None => Vec::new(),
        };

        // Extract the stock symbol from the key
        let stock_symbol = key.split(":").last().unwrap().to_string();

        // Insert the buy and sell orders into the HashMap
        all_stock_orders.insert(stock_symbol, (buy_orders, sell_orders));
    }

    Ok(all_stock_orders)
}

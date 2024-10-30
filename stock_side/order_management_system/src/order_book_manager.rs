use redis::{aio, AsyncCommands, RedisResult};   // RedisResult: Result type for Redis commands
use common::models::{Order, OrderType, Trade};
use serde_json::{from_str, to_string};  // Deserialize JSON string to struct; Serialize struct to JSON string
// use std::sync::Arc;
// use tokio::sync::Mutex;  // Mutex: Mutual Exclusion, used to synchronize access to shared data

pub struct OrderBookManager {
    // ARC: Atomic Reference Counting, used to share ownership between threads
    // Mutex: Mutual Exclusion, used to synchronize access to shared data
    // So with arc and mutex, we can ensure that only one thread can access the Redis connection at a time
    redis_conn: aio::MultiplexedConnection,
}

impl OrderBookManager {
    pub async fn new(redis_url: &str) -> Self {
        println!("OrderBookManager: Connecting to Redis: {}", redis_url);
        let client = redis::Client::open(redis_url).unwrap();
        let redis_conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("OrderBookManager: Failed to connect to Redis");

        println!("OrderBookManager: Connected to Redis");

        Self {
            redis_conn,
        }
    }

    // return ok or not status
    pub async fn add_to_orderbook(&self, order: Order) -> RedisResult<()> {
        let mut conn = self.redis_conn.clone();

        // Key for the order book
        let order_book_key = format!("order_book:{}", order.stock_symbol);

        // If the order is a buy order, add it to the buy orders in Redis
        match order.order_type {
            OrderType::Buy => {
                // Get the buy orders from Redis, reason is just to sort it later
                let buy_orders_string: Option<String> = conn.hget(&order_book_key, "buy_orders").await?;
                let mut buy_orders: Vec<Order> = match buy_orders_string {
                    Some(buy_orders) => from_str(&buy_orders).expect("Failed to deserialize buy orders"),
                    None => Vec::new(),
                };

                buy_orders.push(order.clone());

                // Sort the buy orders in descending order of price
                buy_orders.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());

                // Serialize the buy orders and update the buy orders in Redis
                let to_redis_buy_orders_string = to_string(&buy_orders).expect("Failed to serialize buy orders");
                conn.hset(&order_book_key, "buy_orders", to_redis_buy_orders_string).await?;

                Ok(())
            }
            // If the order is a sell order, add it to the sell orders in Redis
            OrderType::Sell => {
                let sell_orders_string: Option<String> = conn.hget(&order_book_key, "sell_orders").await?;
                let mut sell_orders: Vec<Order> = match sell_orders_string {
                    Some(sell_orders) => from_str(&sell_orders).expect("Failed to deserialize sell orders"),
                    None => Vec::new(),
                };

                sell_orders.push(order.clone());

                // Sort the sell orders in ascending order of price
                sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

                // Serialize the sell orders and update the sell orders in Redis
                let to_redis_sell_orders_string = to_string(&sell_orders).expect("Failed to serialize sell orders");
                conn.hset(&order_book_key, "sell_orders", to_redis_sell_orders_string).await?;

                Ok(())
            }
        }
    }

    pub async fn process_order(&self) -> RedisResult<Option<Trade>> {
        // Get the different order books from Redis
        let mut conn = self.redis_conn.clone();
        let order_books: Vec<String> = conn.keys("order_book:*").await?;

        // Iterate over the order books
        for order_book_key in order_books {
            // Get the buy and sell orders from Redis
            let (buy_orders_string, sell_orders_string): (Option<String>, Option<String>) = conn
                .hget(&order_book_key, &["buy_orders", "sell_orders"])
                .await?;

            // Deserialize the buy and sell orders into Vec<Order>
            let mut buy_orders: Vec<Order> = match buy_orders_string {
                Some(buy_orders) => from_str(&buy_orders).expect("Failed to deserialize buy orders"),
                None => Vec::new(),
            };

            let mut sell_orders: Vec<Order> = match sell_orders_string {
                Some(sell_orders) => from_str(&sell_orders).expect("Failed to deserialize sell orders"),
                None => Vec::new(),
            };

            // Check the first buy order (largest) and the first sell order (smallest) to see if a trade can be made
            while let (Some(buy_order), Some(sell_order)) = (buy_orders.first(), sell_orders.first()) {
                if buy_order.price >= sell_order.price {
                    // Take the minimum quantity between the buy and sell orders
                    let trade_quantity = buy_order.quantity.min(sell_order.quantity);

                    // Create a trade
                    let trade = Trade {
                        buy_order_id: buy_order.id.clone(),
                        sell_order_id: sell_order.id.clone(),
                        stock_symbol: buy_order.stock_symbol.clone(),
                        quantity: trade_quantity,
                        price: sell_order.price,   // Take sell order because a trade should take the lower price
                        timestamp: std::cmp::max(buy_order.timestamp, sell_order.timestamp),
                    };

                    // Update the quantity of either the buy or sell order
                    if buy_order.quantity > trade_quantity {
                        buy_orders[0].quantity -= trade_quantity;
                        sell_orders.remove(0);
                    } else if sell_order.quantity > trade_quantity {
                        sell_orders[0].quantity -= trade_quantity;
                        buy_orders.remove(0);
                    } else {
                        buy_orders.remove(0);
                        sell_orders.remove(0);
                    }

                    // Update the buy and sell orders in Redis
                    let to_redis_buy_orders_string = to_string(&buy_orders).expect("Failed to serialize buy orders");
                    conn.hset(&order_book_key, "buy_orders", to_redis_buy_orders_string).await?;

                    let to_redis_sell_orders_string = to_string(&sell_orders).expect("Failed to serialize sell orders");
                    conn.hset(&order_book_key, "sell_orders", to_redis_sell_orders_string).await?;
                    
                    return Ok(Some(trade));
                } else {
                    // If the buy order price is less than the sell order price, then no trade can be made, meaning the highest buy order price is less than the lowest sell order price
                    break;
                }
            }
        }

        Ok(None)
    }

    // pub async fn process_order(&self, order: Order) -> RedisResult<Option<Trade>> {
    //     let order_book_key = format!("order_book:{}", order.stock_symbol);
    //     let mut conn = self.redis_conn.clone();

    //     // Get the buy and sell orders from Redis
    //     let (buy_orders_string, sell_orders_string): (Option<String>, Option<String>) = conn
    //         .hget(&order_book_key, &["buy_orders", "sell_orders"])
    //         .await?;

    //     // Deserialize the buy and sell orders into Vec<Order>
    //     let buy_orders: Vec<Order> = match buy_orders_string {
    //         Some(buy_orders) => from_str(&buy_orders).expect("Failed to deserialize buy orders"),
    //         None => Vec::new(),
    //     };

    //     let sell_orders: Vec<Order> = match sell_orders_string {
    //         Some(sell_orders) => from_str(&sell_orders).expect("Failed to deserialize sell orders"),
    //         None => Vec::new(),
    //     };

    //     // Get the order type to check if it is a buy or sell order
    //     let order_type: OrderType = order.order_type.clone();

    //     // Trade is an Option<Trade> because it may or may not be generated
    //     let trade: Option<Trade> = match order_type {
    //         OrderType::Buy => {
    //             handle_buy_order(&mut conn, &order_book_key, order.clone(), buy_orders, sell_orders).await?
    //         }
    //         OrderType::Sell => {
    //             handle_sell_order(&mut conn, &order_book_key, order.clone(), sell_orders, buy_orders).await?
    //         }
    //     };
    //     Ok(trade)
    // }
}

// async fn handle_buy_order(
//     conn: &mut aio::MultiplexedConnection,
//     order_book_key: &str,
//     mut order: Order,
//     mut buy_orders: Vec<Order>,
//     mut sell_orders: Vec<Order>,
// ) -> RedisResult<Option<Trade>> {
//     if let Some((index, matching_order)) = sell_orders
//         .iter()  // Iterate over the sell orders
//         .enumerate()  // Enumerate the sell orders
//         .find(|(_, sell_order)| {
//             sell_order.price <= order.price &&  // Find the first sell order with a price less than or equal to the buy order price
//             (order.partial_fill || sell_order.quantity >= order.quantity)  // Additional filter: check quantity only if partial_fill is false; must make sure that the sell order quantity is greater than or equal to the buy order quantity
//         })
//         .map(|(index, sell_order)| (index, sell_order.clone())) // Clone the matching order to avoid borrowing issues
//     {
//         // Update the sell orders in Redis
//         let mut trade_quantity = order.quantity;
//             // sell order more than buy order quantity, remaining sell quantity to add to order book (redis)
//         if sell_orders[index].quantity > order.quantity {
//             sell_orders[index].quantity -= order.quantity;
//         }
//             // sell order equal to buy order quantity, remove sell order from order book (redis) 
//         else if sell_orders[index].quantity == order.quantity {
//             sell_orders.remove(index);
//         }
//             // sell order less than buy order quantity, remaining buy quantity to add to order book (redis); if partial_fill is false, then this should not be able to happen
//         else if sell_orders[index].quantity < order.quantity {
//             order.quantity -= sell_orders[index].quantity;
//             buy_orders.push(order.clone());
//             // sort the buy orders in descending order of price
//             buy_orders.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
//             // update the buy orders in Redis
//             let to_redis_buy_orders_string = to_string(&buy_orders).expect("Failed to serialize buy orders");
//             conn.hset(order_book_key, "buy_orders", to_redis_buy_orders_string).await?;
            
//             trade_quantity = sell_orders[index].quantity;  // Because the sell order quantity is less than the buy order quantity
            
//             sell_orders.remove(index);
//         }

//         // Create a trade
//         let trade = Trade {
//             buy_order_id: order.id.clone(),
//             sell_order_id: matching_order.id.clone(),
//             stock_symbol: order.stock_symbol.clone(),
//             quantity: trade_quantity,
//             price: order.price,   // Take order because it is the buy order with the higher price
//             timestamp: order.timestamp,
//         };

//         // Update the sell orders in Redis
//         let to_redis_sell_orders_string = to_string(&sell_orders).expect("Failed to serialize sell orders");
//         conn.hset(order_book_key, "sell_orders", to_redis_sell_orders_string).await?;

//         return Ok(Some(trade));
//     } else {
//         buy_orders.push(order.clone());

//         // Sort the buy orders in descending order of price
//         buy_orders.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());

//         // Update the buy orders in Redis
//         let to_redis_buy_orders_string = to_string(&buy_orders).expect("Failed to serialize buy orders");
//         conn.hset(order_book_key, "buy_orders", to_redis_buy_orders_string).await?;

//         return Ok(None);
//     }
// }

// async fn handle_sell_order(
//     conn: &mut aio::MultiplexedConnection,
//     order_book_key: &str,
//     mut order: Order,
//     mut sell_orders: Vec<Order>,
//     mut buy_orders: Vec<Order>,
// ) -> RedisResult<Option<Trade>> {
//     if let Some((index, matching_order)) = buy_orders
//         .iter()  // Iterate over the buy orders
//         .enumerate()  // Enumerate the buy orders
//         .find(|(_, buy_order)| {
//             buy_order.price >= order.price &&  // Find the first buy order with a price greater than or equal to the sell order price
//             (order.partial_fill || buy_order.quantity >= order.quantity)  // Additional filter: check quantity only if partial_fill is false; must make sure that the buy order quantity is greater than or equal to the sell order quantity
//         })
//         .map(|(index, buy_order)| (index, buy_order.clone())) // Clone the matching order to avoid borrowing issues
//     {
//         // Update the buy orders in Redis
//         let mut trade_quantity = order.quantity;
//             // buy order more than sell order quantity, remaining buy quantity to add to order book (redis)
//         if buy_orders[index].quantity > order.quantity {
//             buy_orders[index].quantity -= order.quantity;
//         }
//             // buy order equal to sell order quantity, remove buy order from order book (redis) 
//         else if buy_orders[index].quantity == order.quantity {
//             buy_orders.remove(index);
//         }
//             // buy order less than sell order quantity, remaining sell quantity to add to order book (redis); if partial_fill is false, then this should not be able to happen
//         else if buy_orders[index].quantity < order.quantity {
//             order.quantity -= buy_orders[index].quantity;
//             sell_orders.push(order.clone());
//             // sort the sell orders in ascending order of price
//             sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
//             // update the sell orders in Redis
//             let to_redis_sell_orders_string = to_string(&sell_orders).expect("Failed to serialize sell orders");
//             conn.hset(order_book_key, "sell_orders", to_redis_sell_orders_string).await?;
            
//             trade_quantity = buy_orders[index].quantity;  // Because the buy order quantity is less than the sell order quantity
            
//             buy_orders.remove(index);
//         }

//         // Create a trade
//         let trade = Trade {
//             buy_order_id: matching_order.id.clone(),
//             sell_order_id: order.id.clone(),
//             stock_symbol: order.stock_symbol.clone(),
//             quantity: trade_quantity,
//             price: matching_order.price,   // Take matching order because it is the buy order with the higher price
//             timestamp: order.timestamp,
//         };

//         // Update the buy orders in Redis
//         let to_redis_buy_orders_string = to_string(&buy_orders).expect("Failed to serialize buy orders");
//         conn.hset(order_book_key, "buy_orders", to_redis_buy_orders_string).await?;

//         return Ok(Some(trade));
//     } else {
//         sell_orders.push(order.clone());

//         // Sort the sell orders in ascending order of price
//         sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

//         // Update the sell orders in Redis
//         let to_redis_sell_orders_string = to_string(&sell_orders).expect("Failed to serialize sell orders");
//         conn.hset(order_book_key, "sell_orders", to_redis_sell_orders_string).await?;

//         return Ok(None);
//     }
// }


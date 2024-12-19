use common::models::{Order, Trade};
use chrono::Local;
        
// Algorithm 1: buy vs sell demand in term of share (number of buy/sell * their no. of share)
pub fn algorithm_1(order: &(String, (Vec<Order>, Vec<Order>)), multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the total number of share for buy and sell orders
    let total_buy_share: u32 = buy_orders.iter().map(|o| o.quantity).sum();
    let total_sell_share: u32 = sell_orders.iter().map(|o| o.quantity).sum();


    // Calculate the multiplier
    let imbalance: i32 = total_buy_share as i32 - total_sell_share as i32;
    // for every 100 imbalance, the stock price will increase/decrease by 0.001
    let multiplier = 1.0 + (imbalance as f64 / 100.0) * 0.001;
    multiplier_vec.push(multiplier);


    let log_message = format!(
        "{}\nAlgorithm 1: Buy /Sell action affects the stock price\nStock symbol: {}\nThe rule is for every 100 imbalance, the stock price will increase/decrease by 0.001\nTotal buy share: {}\nTotal sell share: {}\nImbalance: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, total_buy_share, total_sell_share, imbalance, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 2: Order Imbalance: buy vs sell demand in term of order amount ((total buy orders - total sell orders) / (total buy orders + total sell orders))
pub fn algorithm_2(order:  &(String, (Vec<Order>, Vec<Order>)), multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the total order amount for buy and sell orders
    let num_buy_orders = buy_orders.len() as f64;
    let num_sell_orders = sell_orders.len() as f64;

    // Calculate the multiplier
    let imbalance = num_buy_orders - num_sell_orders;
    // for every total order amount, the stock price will increase/decrease by 0.01
    let multiplier = 1.0 + (imbalance / (num_buy_orders + num_sell_orders)) * 0.01;
    multiplier_vec.push(multiplier);

    let log_message = format!(
        "{}\nAlgorithm 2: Order Imbalance: buy vs sell demand in term of order amount\nStock symbol: {}\nThe rule is for every total order amount, the stock price will increase/decrease by 0.01\nTotal buy orders: {}\nTotal sell orders: {}\nImbalance: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, num_buy_orders, num_sell_orders, imbalance, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 3: Cumulative Order Book Depth: buy vs sell demand in term of price level (total share of buy/sell orders @ price level)
pub fn algorithm_3(order:  &(String, (Vec<Order>, Vec<Order>)), multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the total order amount for buy and sell orders
        // Get the price level for buy and sell orders, by taking the top 20% of the orders
    let percentile_buy = (buy_orders.len() as f64 * 0.2).ceil() as usize;
    let total_buy_share: u32 = buy_orders.iter().take(percentile_buy).map(|o| o.quantity).sum();

    let percentile_sell = (sell_orders.len() as f64 * 0.2).ceil() as usize;
    let total_sell_share: u32 = sell_orders.iter().take(percentile_sell).map(|o| o.quantity).sum();

    // Calculate the multiplier
    let imbalance = total_buy_share as i32 - total_sell_share as i32;
    // for every 50 imbalance, the stock price will increase/decrease by 0.01
    let multiplier = 1.0 + (imbalance as f64 / 50.0) * 0.01;
    multiplier_vec.push(multiplier);

    let log_message = format!(
        "{}\nAlgorithm 3: Cumulative Order Book Depth: buy vs sell demand in term of price level\nStock symbol: {}\nThe rule is for every 50 imbalance, the stock price will increase/decrease by 0.01\nTotal buy share: {}\nTotal sell share: {}\nImbalance: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, total_buy_share, total_sell_share, imbalance, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 4: Market Pressure from Large Orders (Iceberg Effect): buy vs sell in term of largest share of buy/sell orders that deviate from the current stock price
pub fn algorithm_4(order:  &(String, (Vec<Order>, Vec<Order>)), multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the average share of the buy and sell orders
    let total_buy_share: u32 = buy_orders.iter().map(|o| o.quantity).sum();
    let avg_buy_share: u32 = total_buy_share / buy_orders.len().max(1) as u32;
    let total_sell_share: u32 = sell_orders.iter().map(|o| o.quantity).sum();
    let avg_sell_share: u32 = total_sell_share / sell_orders.len().max(1) as u32;
    
    // Set the large amount threashold
    let large_amount: u32 = (avg_buy_share + avg_sell_share) / 2;

    // Get the number of buy and sell orders that are larger than the large amount threashold
    let num_large_buy_orders = buy_orders.iter().filter(|o| o.quantity > large_amount).count();
    let num_large_sell_orders = sell_orders.iter().filter(|o| o.quantity > large_amount).count();

    // Calculate the multiplier
    let imbalance: i64 = num_large_buy_orders as i64 - num_large_sell_orders as i64;
    // for every 15 large order imbalance, the stock price will increase/decrease by 0.02
    let multiplier = 1.0 + (imbalance as f64 / 15.0) * 0.02;
    multiplier_vec.push(multiplier);

    let log_message = format!(
        "{}\nAlgorithm 4: Market Pressure from Large Orders (Iceberg Effect): buy vs sell in term of largest share of buy/sell orders that deviate from the current stock price\nStock symbol: {}\nThe rule is for every 15 large order imbalance, the stock price will increase/decrease by 0.05\nAverage buy share: {}\nAverage sell share: {}\nLarge amount threashold: {}\nNumber of large buy orders: {}\nNumber of large sell orders: {}\nImbalance: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, avg_buy_share, avg_sell_share, large_amount, num_large_buy_orders, num_large_sell_orders, imbalance, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 5: Order Flow Momentum: check the recent time window of buy/sell orders, and if there is a momentum in the order flow, then increase/decrease the stock price perspectively
pub fn algorithm_5(order:  &(String, (Vec<Order>, Vec<Order>)), multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the recent time window of buy and sell orders
    let time_window = 5; // 5 seconds
    let recent_buy_orders = buy_orders.iter().filter(|o| Local::now().timestamp() - o.timestamp as i64 <= time_window).count() as f64;
    let recent_sell_orders = sell_orders.iter().filter(|o| Local::now().timestamp() - o.timestamp as i64 <= time_window).count() as f64;

    // Calculate the multiplier
    let imbalance = recent_buy_orders - recent_sell_orders;
    // for every 10 recent order imbalance, the stock price will increase/decrease by 0.1
    let multiplier = 1.0 + (imbalance / 10.0) * 0.1;
    multiplier_vec.push(multiplier);

    let log_message = format!(
        "{}\nAlgorithm 5: Order Flow Momentum: check the recent {} seconds time window of buy/sell orders, and if there is a momentum in the order flow, then increase/decrease the stock price perspectively\nStock symbol: {}\nThe rule is for every 10 recent order imbalance, the stock price will increase/decrease by 0.1\nRecent buy orders: {}\nRecent sell orders: {}\nImbalance: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), time_window, &order.0, recent_buy_orders, recent_sell_orders, imbalance, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 6: Order Book Skewness: skew of the order book (buy/sell orders) in term of price
pub fn algorithm_6(order:  &(String, (Vec<Order>, Vec<Order>)), current_market_price: f64, multiplier_vec: &mut Vec<f64>) {
    let buy_orders = &order.1 .0;
    let sell_orders = &order.1 .1;

    // Get the highest buy and lowest sell price
    let highest_buy_price = buy_orders.first().map(|o| o.price).unwrap_or(0.0);   // The highest buy price is the first buy order in the buy orders
    let lowest_sell_price = sell_orders.first().map(|o| o.price).unwrap_or(0.0);    // The lowest sell price is the first sell order in the sell orders

    // Calculate the multiplier
    let imbalance = highest_buy_price - lowest_sell_price;
    let skewness = (imbalance / current_market_price) * 100.0; // The skewness is in percentage
    // for every 10% skewness, the stock price will increase/decrease by 0.01
    let multiplier = 1.0 + (skewness / 10.0) * 0.01;
    multiplier_vec.push(multiplier);

    let log_message = format!(
        "{}\nAlgorithm 6: Order Book Skewness: skew of the order book (buy/sell orders) in term of price\nStock symbol: {}\nThe rule is for every 10% skewness, the stock price will increase/decrease by 0.01\nHighest buy price: {}\nLowest sell price: {}\nImbalance: {}\nCurrent Market Price: {}\nSkewness: {}\nMultiplier: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, highest_buy_price, lowest_sell_price, imbalance, current_market_price, skewness, multiplier
    );
    log_to_file(log_message)
}

// Algorithm 7: Industry Sector Performance: check the sector of the stock and adjust the stock price based on the sector performance
pub fn algorithm_7(_order: &(String, (Vec<Order>, Vec<Order>)), sector: &str, multiplier_vec: &mut Vec<f64>) {
    // Get the sector performance
    let sector_performance = match sector {
        "Technology" => 0.02, // 2% increase in stock price
        "Finance" => -0.01, // 1% decrease in stock price
        "Healthcare" => 0.03, // 3% increase in stock price
        "Consumer Goods" => 0.01, // 1% increase in stock price
        "Energy" => -0.02, // 2% decrease in stock price
        "Real Estate" => 0.02, // 2% increase in stock price
        "Utilities" => 0.01, // 1% increase in stock price
        _ => 0.0, // No change in stock price
    };

    // Calculate the multiplier
    let multiplier = 1.0 + sector_performance;
    multiplier_vec.push(multiplier);

    // let log_message = format!(
    //     "{}\nAlgorithm 7: Industry Sector Performance: check the sector of the stock and adjust the stock price based on the sector performance\nStock symbol: {}\nSector: {}\nSector Performance: {}\nMultiplier: {}\n",
    //     Local::now().format("%H.%M.%S %d-%m-%y").to_string(), &order.0, sector, sector_performance, multiplier
    // );
    // log_to_file(log_message)
    
}

// Algorithm For Active Trader: check the number of active traders and adjust the stock price based on the number of active traders
pub fn algorithm_trade(trade_received: &Trade, stock_price: f64) -> f64 {
    // Update the stock price based on the trade price; 
    let imbalance = (trade_received.quantity as f64) * (trade_received.price - stock_price);
    let multiplier = 1.0 + (imbalance / stock_price) * 0.01; // 0.01 is the sensitivity factor
    let new_price = (stock_price * multiplier * 10000.0).round() / 10000.0; // Round the new price to 4 decimal places

    let log_message = format!(
        "{}\nAlgorithm Trade: Update the stock price based on the trade price\nStock symbol: {}\nTrade price: {}\nTrade quantity: {}\nStock price: {}\nImbalance: {}\nMultiplier: {}\nNew Price: {}\n",
        Local::now().format("%H.%M.%S %d-%m-%y").to_string(), trade_received.stock_symbol, trade_received.price, trade_received.quantity, stock_price, imbalance, multiplier, new_price
    );
    log_to_file(log_message);

    new_price
}


//* --------------------- Helper Functions ---------------------  *//
use std::fs::OpenOptions;
use std::io::Write;
use once_cell::sync::Lazy;

static TIMESTAMP: Lazy<String> = Lazy::new(|| Local::now().format("%H.%M.%S %d-%m-%y").to_string());
fn log_to_file(message: String) {
    let file_path = format!("./log/algorithm_log/{}.txt", *TIMESTAMP);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .expect("Failed to open log file");

    if let Err(e) = writeln!(file, "{}", message) {
        eprintln!("Failed to write to log file: {}", e);
    }
}
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: String,
    pub stock_symbol: String,
    pub order_type: OrderType,
    pub quantity: u32,
    pub price: f64,
    pub timestamp: u64,
    pub partial_fill: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderType {
    Buy,
    Sell,
}


impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::Buy => write!(f, "Buy"),
            OrderType::Sell => write!(f, "Sell"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub stock_symbol: String,
    pub quantity: u32,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
    pub symbol: String,
    pub price: f64,
}
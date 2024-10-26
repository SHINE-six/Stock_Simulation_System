use serde::{Deserialize, Serialize};

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

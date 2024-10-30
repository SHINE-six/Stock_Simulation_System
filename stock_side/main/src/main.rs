use common::models::{Order, Stock, Trade};
use communication_layer::consumer::OrderConsumer;
use communication_layer::producer::StockProducer;
use market_data_generator::price_updater::MarketDataGenrator;
use order_management_system::order_book_manager::OrderBookManager;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let (oms_sender, mut oms_receiver): (Sender<Order>, Receiver<Order>) = channel(100);
    let (mdg_sender, mdg_receiver): (Sender<Trade>, Receiver<Trade>) = channel(100);
    let (stock_sender, mut stock_receiver): (Sender<Stock>, Receiver<Stock>) = channel(100);

    // ------------- Order Management System -------------
    const BROKERS: &str = "localhost:19092"; // redpanda-0:9092
    const TO_CONSUME_TOPIC: &str = "broker-orders";
    const GROUP_ID: &str = "oms_consumer_group";
    const TO_PRODUCE_TOPIC: &str = "stock-prices";
    const REDIS_URL: &str = "redis://localhost:6379";

    let consumer = OrderConsumer::new(BROKERS, TO_CONSUME_TOPIC, GROUP_ID);

    let order_book_manager = OrderBookManager::new(REDIS_URL).await;

    let consumer_handle = tokio::spawn(async move {
        consumer.consume_messages(oms_sender).await;
        println!("Consumer stopped");
    });

    let order_book_manager_handle = tokio::spawn(async move {
        while let Some(order_received) = oms_receiver.recv().await {
            // Process the order
            if let Some(trade) = order_book_manager
                .process_order(order_received)
                .await
                .unwrap()
            {
                // Send the trade to the trading side
                if let Err(e) = mdg_sender.send(trade).await {
                    eprintln!("Failed to send trade via mdg_sender: {}", e);
                }
            }
        }
        panicln!("Order Book Manager stopped");
    });

    // Catch the channel receiver and send to market data generator
    let market_data_generator = MarketDataGenrator::new(REDIS_URL).await;
    let market_data_generator_handle = task::spawn(async move {
        market_data_generator.start(mdg_receiver, stock_sender).await;

        panicln!("Market Data Generator stopped");
    });

    let producer = StockProducer::new(BROKERS);

    let producer_handle = task::spawn(async move {
        // Produce stock prices from channel
        while let Some(stock) = stock_receiver.recv().await {
            producer.produce_stock(stock, TO_PRODUCE_TOPIC).await;
        }
    });

    let _ = tokio::join!(
        consumer_handle,
        order_book_manager_handle,
        market_data_generator_handle,
        producer_handle
    );
}

use common::models::{Order, Stock, Trade};
use communication_layer::consumer::OrderConsumer;
use communication_layer::producer::StockProducer;
use market_data_generator::price_updater::MarketDataGenrator;
use order_management_system::order_book_manager::OrderBookManager;
use std::sync::Arc;

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
    
    let consumer_handle = tokio::spawn(async move {
        consumer.consume_messages(oms_sender).await;
        println!("Consumer stopped");
    });

    // This need ARC, because it will be shared between the 2 thread (process_order and add_to_orderbook)
        // No need Mutex, because they use different function, do different thing
    let order_book_manager = OrderBookManager::new(REDIS_URL).await;
    let order_book_manager_clone = Arc::new(order_book_manager);

    // every 500ms, check the order book, to see any trades can be made
    let order_book_manager_process_order_handle = tokio::spawn({
        let order_book_manager = order_book_manager_clone.clone();
        async move {
            loop {
                // Process the order
                match order_book_manager.process_order().await {
                    Ok(Some(trade)) => {
                        // Send the trade to the trading side
                        println!("Trade: {:?}", trade);
                        if let Err(e) = mdg_sender.send(trade).await {
                            eprintln!("Failed to send trade via mdg_sender: {}", e);
                        }
                    },
                    Ok(None) => {},
                    Err(e) => {
                        eprintln!("Failed to process order: {}", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    });

    let order_book_manager_add_to_order_handle = tokio::spawn({
        let order_book_manager = order_book_manager_clone.clone();
        async move {
            // Everytime receive an order, add to order book
            while let Some(order_received) = oms_receiver.recv().await {
                // Add the order to the order book
                order_book_manager.add_to_orderbook(order_received).await.expect("Failed to add order to order book");

                // if let Some(trade) = order_book_manager
                //     .add_to_orderbook(order_received)
                //     .await
                //     .unwrap()
                // {
                //     // Send the trade to the trading side
                //     if let Err(e) = mdg_sender.send(trade).await {
                //         eprintln!("Failed to send trade via mdg_sender: {}", e);
                //     }
                // }
            }
            panic!("Order Book Manager stopped");
        }
    });

    // Catch the channel receiver and send to market data generator
    let market_data_generator = MarketDataGenrator::new(REDIS_URL).await;
    let market_data_generator_handle = task::spawn(async move {
        market_data_generator.start(mdg_receiver, stock_sender).await;

        panic!("Market Data Generator stopped");
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
        order_book_manager_process_order_handle,
        order_book_manager_add_to_order_handle,
        market_data_generator_handle,
        producer_handle
    );
}

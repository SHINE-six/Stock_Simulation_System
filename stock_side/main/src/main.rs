extern crate sys_info;

use common::models::{Order, Stock, Trade};
use communication_layer::consumer::OrderConsumer;
use communication_layer::producer::StockProducer;
use market_data_generator::price_updater::MarketDataGenrator;
use order_management_system::order_book_manager::OrderBookManager;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use tokio::signal;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use peak_alloc::PeakAlloc;


#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // Initialize the console subscriber to emit tracing data
    // console_subscriber::init();

    let (log_sender, mut log_receiver): (Sender<String>, Receiver<String>) = channel(100);

    // System info
    let cpu_num = sys_info::cpu_num().unwrap();
    let boot_time = sys_info::boottime().unwrap().tv_sec;
    let proc_total = sys_info::proc_total().unwrap();
    let load_avg = sys_info::loadavg().unwrap();
    let mem_info = sys_info::mem_info().unwrap();
    let disk_info = sys_info::disk_info().unwrap();
    let cpu_speed = sys_info::cpu_speed().unwrap();
    let os_info = sys_info::linux_os_release().unwrap();

    println!("Number of CPU: {}", cpu_num);
    println!("System Boot Time: {}", boot_time);
    println!("Number of Processes: {}", proc_total);
    println!("System Load Average: {:?}", load_avg);
    println!("Total Memory: {}", mem_info.total);
    println!("Free Memory: {}", mem_info.free);
    println!("Disk Info: {:?}", disk_info);
    println!("Cpu Speed: {}", cpu_speed);
    println!(
        "OS: {}, {}",
        os_info.name.clone().unwrap_or("Unknown".to_string()),
        os_info.version.clone().unwrap_or("Unknown".to_string())
    );

    // Log system info
    let system_info = format!(
        "Number of CPU: {}\nSystem Boot Time: {}\nNumber of Processes: {}\nSystem Load Average: {:?}\nTotal Memory: {}\nFree Memory: {}\nDisk Info: {:?}\nCpu Speed: {}\nOS: {}, {}\n",
        cpu_num, boot_time, proc_total, load_avg, mem_info.total, mem_info.free, disk_info, cpu_speed, os_info.name.unwrap_or("Unknown".to_string()), os_info.version.unwrap_or("Unknown".to_string())
    );
    let _ = log_sender.send(system_info).await;

    println!("\n------------------------------------------------------------------- Application Start -------------------------------------------------------------------\n");

    // Create channels
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
    
    let consumer_handle = tokio::spawn({
        let log_sender = log_sender.clone();

        async move {
            consumer.consume_messages(oms_sender, log_sender).await;
            panic!("Consumer stopped");
        }
    });

    // This need ARC, because it will be shared between the 2 thread (process_order and add_to_orderbook)
        // No need Mutex, because they use different function, do different thing
    let order_book_manager = OrderBookManager::new(REDIS_URL).await;
    // let order_book_manager_clone = Arc::new(order_book_manager);

    // // every 500ms, check the order book, to see any trades can be made
    // let order_book_manager_process_order_handle = tokio::spawn({
    //     let order_book_manager = order_book_manager_clone.clone();
    //     let log_sender = log_sender.clone();
    //     async move {
    //         loop {
    //             let start = std::time::Instant::now();
    //             // Process the order
    //             match order_book_manager.process_order().await {
    //                 Ok(Some(trade)) => {
    //                     // Send the trade to the trading side
    //                     println!("Trade: {:?}", trade);
    //                     if let Err(e) = mdg_sender.send(trade).await {
    //                         eprintln!("Failed to send trade via mdg_sender: {}", e);
    //                     }
    //                 },
    //                 Ok(None) => {},
    //                 Err(e) => {
    //                     eprintln!("Failed to process order: {}", e);
    //                 }
    //             }
    //             let elapsed = start.elapsed();
    //             let _ = log_sender.send(format!("OrderBookManager (Check for trade): {:?}", elapsed)).await;

    //             tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    //         }
    //     }
    // });

    let order_book_manager_add_to_order_handle = tokio::spawn({
        // let order_book_manager = order_book_manager_clone;
        let log_sender = log_sender.clone();
        async move {
            // Everytime receive an order, add to order book, then process the order
            while let Some(order_received) = oms_receiver.recv().await {
                let start = std::time::Instant::now();
                // Add the order to the order book
                order_book_manager.add_to_orderbook(order_received.clone()).await.expect("Failed to add order to order book");

                let elapsed = start.elapsed();
                let _ = log_sender.clone().send(format!("OrderBookManager (Add to order): {:?}", elapsed)).await;

                let start = std::time::Instant::now();
                // Then process the order
                match order_book_manager.process_order(order_received.stock_symbol).await {
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

                let elapsed = start.elapsed();
                let _ = log_sender.clone().send(format!("OrderBookManager (Check for trade): {:?}", elapsed)).await;
            }
            panic!("Order Book Manager stopped");
        }
    });

    // Catch the channel receiver and send to market data generator
    let market_data_generator = MarketDataGenrator::new(REDIS_URL).await;

    /*
    Allowing market data generator thread to end after sending initial stock prices and spawning 4 threads.
        Originally, the market data generator thread will only spawn 3 more thread, and take on the last thread task itself.
        However, using the console subscriber to monitor, it can be observed that if the market data generator take on the last thread task, the market data generator thread will have too much load.
    */
    let market_data_generator_handle = tokio::spawn({
        let log_sender = log_sender.clone();
        async move {
            market_data_generator.start(mdg_receiver, stock_sender, log_sender).await;

            println!("Market Data Generator finish spawning 3 threads and send initial stock prices");
            // panic!("Market Data Generator stopped");
        }
    });

    let producer = StockProducer::new(BROKERS);

    let producer_handle = tokio::spawn(async move {
        // Produce stock prices from channel
        while let Some(stock) = stock_receiver.recv().await {
            producer.produce_stock(stock, TO_PRODUCE_TOPIC).await;
        }
    });

    // Log Receiver and write to file ./log/log.txt
    let log_handle = tokio::spawn(async move {
        let mut file = tokio::fs::File::create("./log/log.txt").await.expect("Failed to create log file");
        while let Some(log) = log_receiver.recv().await {
            let log = format!("{}\n", log);
            file.write_all(log.as_bytes()).await.expect("Failed to write log to file");
        }
    });

    let start = std::time::Instant::now();

    loop {
        // Print current memory usage
        let _ = log_sender.send(format!("Current memory usage (KB) : {}", PeakAlloc.current_usage_as_kb())).await;
        
        // Check if ctrl_c is pressed without blocking the loop,
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {},
            _ = signal::ctrl_c() => {
                println!("Ctrl+C pressed, exiting loop.");
                break;
            },
        }
    }

    // Handle the ctrl_c signal
    // signal::ctrl_c().await.expect("Failed to listen for ctrl-c event");

    println!("\n------------------------------------------------------------------- Application Stop -------------------------------------------------------------------\n");
    
    let elapsed = start.elapsed();
    println!("Application run time: {:?}", elapsed);
    let _ = log_sender.send(format!("Application run time: {:?}", elapsed)).await;

    // Print peak memory usage
    println!("Peak memory usage: {} MB of RAM", PeakAlloc.peak_usage_as_mb());
    let _ = log_sender.send(format!("Peak memory usage: {} MB of RAM", PeakAlloc.peak_usage_as_mb())).await;

    // Sleep for 1 second to allow the log to be written to file
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // let _ = tokio::join!(
    //     consumer_handle,
    //     // order_book_manager_process_order_handle,
    //     order_book_manager_add_to_order_handle,
    //     market_data_generator_handle,
    //     producer_handle,
    //     log_handle,
    // );
}

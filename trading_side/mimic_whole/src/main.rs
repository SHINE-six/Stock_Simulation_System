mod models;
mod consumer;
mod producer;

use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use warp::Filter;
use serde_json::{Value, json};

use models::Stock;
use consumer::start_kafka_consumer;
use producer::OrderProducer;

#[tokio::main]
async fn main() {
    let brokers = "localhost:19092";

    let (tx, _rx) = broadcast::channel(16);

    // Kafka Stock consumer task
    tokio::spawn(start_kafka_consumer(brokers, "stock-prices", tx.clone()));

    // Kafka Order producer task
    let order_producer = OrderProducer::new(brokers, "broker-orders");
    // tokio::spawn({
    //     let order_producer = order_producer.clone();
    //     async move {
    //         order_producer.start_order_producer().await;
    // }});

    // Set up WebSocket route
    let tx_filter = warp::any().map(move || tx.clone());
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(tx_filter)
        .and_then(ws_handler);

    // Set up POST /order route
    let order_route = warp::path("order")
        .and(warp::post())
        .and(warp::body::json())
        .and_then({
            let order_producer = order_producer.clone();
            move |json_body: Value| {
                let order_producer = order_producer.clone();
                async move {
                    println!("Order Received");

                    let full_order_detail = order_producer.produce_custom_order(&json_body).await;

                    Ok::<_, warp::Rejection>(warp::reply::json(&json!({
                        "status": "ok",
                        "received_message": full_order_detail
                    })))
                }
            }
        });

    // Serve static files (HTML, JS, CSS)
    let static_files = warp::fs::dir("public");

    // Combine routes
    let routes = ws_route.or(order_route).or(static_files);

    println!("WebSocket server running on ws://localhost:3030/ws");
    println!("Static files server running on http://localhost:3030");

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn ws_handler(ws: warp::ws::Ws, tx: broadcast::Sender<Stock>) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(ws.on_upgrade(move |websocket| client_connection(websocket, tx)))
}

async fn client_connection(ws: warp::ws::WebSocket, tx: broadcast::Sender<Stock>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    // Spawn a task to forward messages from Kafka to WebSocket
    let forward_task = tokio::task::spawn(async move {
        while let Ok(stock) = rx.recv().await {
            let msg = serde_json::to_string(&stock).unwrap();
            if ws_tx
                .send(warp::ws::Message::text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Handle client messages if necessary
    while let Some(result) = ws_rx.next().await {
        if result.is_err() {
            break;
        }
        // We can process client messages here if needed
    }

    // Wait for the forward task to finish
    let _ = forward_task.await;
}

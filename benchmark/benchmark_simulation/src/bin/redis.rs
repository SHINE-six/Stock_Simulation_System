#[macro_use]
extern crate bma_benchmark;

use redis::{aio, AsyncCommands}; // RedisResult: Result type for Redis commands
use rand::Rng;

#[tokio::main]
async fn main() {
    warmup!();
    
    let n: u32 = 1000;
    
    redis(n).await;

    redis_multi(n).await;
    
    mutex(n).await;

    staged_benchmark_print_for!("redis");
}

async fn redis(n: u32) {
    const FIELDS: [&str; 3] = ["field1", "field2", "field3"];
    let redis_conn: aio::MultiplexedConnection = redis::Client::open("redis://127.0.0.1/").unwrap().get_multiplexed_async_connection().await.unwrap();
    let mut handles = vec![];

    staged_benchmark_start!("redis");
    for _ in 1..5 {
        let redis_conn = redis_conn.clone();

        let handle = tokio::spawn(async move {
            let mut con = redis_conn.clone();
            for _ in 0..n {
                let field = FIELDS[rand::thread_rng().gen_range(0..3)];
                let value = rand::thread_rng().gen_range(0..1000).to_string();

                let () = con.hset("my_key", field, value).await.unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    staged_benchmark_finish_current!(n);
}


async fn redis_multi(n: u32) {
    const FIELDS: [&str; 3] = ["field1", "field2", "field3"];
    let redis_conn: aio::MultiplexedConnection = redis::Client::open("redis://127.0.0.1/").unwrap().get_multiplexed_async_connection().await.unwrap();
    let mut handles = vec![];

    staged_benchmark_start!("redis");
    for _ in 1..5 {
        let redis_conn = redis_conn.clone();

        let handle = tokio::spawn(async move {
            let mut con = redis_conn.clone();
            for _ in 0..n {
                let field = FIELDS[rand::thread_rng().gen_range(0..3)];
                let value = rand::thread_rng().gen_range(0..1000).to_string();

                let () = con.hset("my_key", field, value).await.unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    staged_benchmark_finish_current!(n);
}



use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

async fn mutex(n: u32) {
    const FIELDS: [&str; 3] = ["field1", "field2", "field3"];
    let hashmap: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];

    
    staged_benchmark_start!("mutex");
    for _ in 1..5 {
        let hashmap = Arc::clone(&hashmap);
        
        let handle = tokio::spawn(async move {
            
            for _ in 0..n {
                let mut hashmap = hashmap.lock().await;
                // println!("{:?}", hashmap);
                let field = FIELDS[rand::thread_rng().gen_range(0..3)];
                let value = rand::thread_rng().gen_range(0..1000).to_string();

                // println!("Thread {} insert field: {}, value: {}", i, field, value);
                hashmap.insert(field.to_string(), value);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    staged_benchmark_finish_current!(n);
}

use tokio::sync:: Mutex;
use std::sync::Arc;
use tokio::time::{Duration, interval};

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for i in 1..4 {
        let counter = Arc::clone(&counter);
        let mut interval = interval(Duration::from_secs(1));
        let handle = tokio::spawn(async move {
            loop {
                interval.tick().await;
                let mut counter = counter.lock().await;
                *counter += 1;
                println!("Thread {} counter: {}", i, *counter);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {

    let handler =tokio::spawn(async {
        for _ in 0..10 {
            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Local> = now.into();
            println!("I am doing something, TIME: {}", datetime.format("%S:%3f"));
            // Sleep for 300ms, simulate some work, don't use tokio::time::sleep
            sleep(Duration::from_millis(300)).await;
            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Local> = now.into();
            println!("I am done, TIME: {}", datetime.format("%S:%3f"));
            println!();

            // Wait for 1 second
            sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    handler.await.unwrap();
}
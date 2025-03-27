use chrono;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    let handler = tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(1));

        for _ in 0..10 {
            interval.tick().await; // Wait until the next 1-second interval

            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Local> = now.into();
            println!("I am starting, TIME: {}", datetime.format("%S:%3f"));

            // Simulate some work, sleep for 500ms
            std::thread::sleep(std::time::Duration::from_millis(500));

            let now = std::time::SystemTime::now();
            let datetime: chrono::DateTime<chrono::Local> = now.into();
            println!("I am done, TIME: {}", datetime.format("%S:%3f"));
            println!();
        }
    });

    handler.await.unwrap();
}

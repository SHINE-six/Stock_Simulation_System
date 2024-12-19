#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
pub async fn multi_thread() {
    println!("Test from multi_thread");
}

#[tokio::main(flavor = "current_thread")]
pub async fn current_thread() {
    println!("Test from current_thread");
}

#[tokio::main]
pub async fn default() {
    println!("Test from default");
}
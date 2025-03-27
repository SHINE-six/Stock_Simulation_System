use std::time::Duration;
use tokio::time::sleep;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

// #[tokio::main]
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    // Initialize the console subscriber to emit tracing data
    console_subscriber::init();

    // Spawn a thread task every 2 seconds that will execute for within 1-3 seconds
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        tokio::spawn(async {
            // let mut rng = StdRng::from_entropy();
            // let sleep_time = Duration::from_secs(rng.gen_range(1..4));
            let sleep_time = Duration::from_secs(4);
            let start = Instant::now();
            println!("Spawning task that will sleep for {:?}", sleep_time);
            sleep(sleep_time).await;
            println!("Task completed after {:?}", sleep_time);
            println!("Task completed after {:?}", start.elapsed());
        });
    }

    // Get the current process
    // if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
    //     println!("Process: {:?}", process.session_id());
    //     let mut interval = tokio::time::interval(Duration::from_secs(1));
    //     loop {
    //         interval.tick().await;
    //         system.refresh_all();

    //         // Get the current process
    //         if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
    //             println!("Process: {:?}", process.session_id());
    //             if let Some(tasks) = process.tasks() {
    //                 println!("Listing tasks for process {:?}", process.pid());
    //                 for task_pid in tasks {
    //                     if let Some(task) = system.process(*task_pid) {
    //                         println!("Task {:?}: {:?}", task.pid(), task.name());
    //                     }
    //                 }
    //             }
    //         } else {
    //             println!("Could not find the current process.");
    //         }
    //     }
    //     // if let Some(tasks) = process.tasks() {
    //     //     println!("Listing tasks for process {:?}", process.pid());
    //     //     for task_pid in tasks {
    //     //         if let Some(task) = system.process(*task_pid) {
    //     //             println!("Task {:?}: {:?}", task.pid(), task.name());
    //     //         }
    //     //     }
    //     // }
    // } else {
    //     println!("Could not find the current process.");
    // }
}

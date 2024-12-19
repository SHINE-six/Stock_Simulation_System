use sysinfo::System;

// #[tokio::main(flavor = "multi_thread", worker_threads = 1)]
pub fn main() {
    let mut system = System::new_all();
    system.refresh_all();

    // Get the current process
    if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
        println!("Process: {:?}", process.session_id());
        if let Some(tasks) = process.tasks() {
            println!("Listing tasks for process {:?}", process.pid());
            for task_pid in tasks {
                if let Some(task) = system.process(*task_pid) {
                    println!("Task {:?}: {:?}", task.pid(), task.name());
                }
            }
        }
    } else {
        println!("Could not find the current process.");
    }
}


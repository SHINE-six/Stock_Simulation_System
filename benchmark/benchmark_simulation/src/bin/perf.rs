use perf_monitor::cpu::{ThreadStat, ProcessStat, processor_numbers};
use perf_monitor::mem::get_process_memory_info;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    // Spawn multiple thread and do some intensive work with shared data
    let shared_data = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..4 {
        let shared_data = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            let mut stat_t = ThreadStat::cur().unwrap();
            let usage_t = stat_t.cpu().unwrap() * 100f64;
            println!("[Thread] current thread usage: {:.2}%", usage_t);
            let start = std::time::Instant::now();
            for _ in 0..1_000_000 {
                {
                    let mut data = shared_data.lock().unwrap();
                    *data += 1;
                }
                // unlock the mutex to allow other thread to access the shared data
                thread::yield_now();
            }
            println!("[Thread] current thread usage: {:.2}%", usage_t);

            let elapsed = start.elapsed();
            println!("Thread {:?} finished in {:?}", thread::current().id(), elapsed);
        });
        handles.push(handle);
    }

    loop {
        // cpu
        // let core_num = processor_numbers().unwrap();
        // let mut stat_p = ProcessStat::cur().unwrap();
        // let mut stat_t = ThreadStat::cur().unwrap();

        // let usage_p = stat_p.cpu().unwrap() * 100f64;
        // let usage_t = stat_t.cpu().unwrap() * 100f64;

        // println!("[CPU] core Number: {}, process usage: {:.2}%, current thread usage: {:.2}%", core_num, usage_p, usage_t);

        // // mem
        // let mem_info = get_process_memory_info().unwrap();
        // println!("[Memory] memory used: {} bytes, virtual memory used: {} bytes", mem_info.resident_set_size, mem_info.virtual_memory_size);

        // // Sleep for a second before the next update
        // thread::sleep(Duration::from_secs(1));
    }

}
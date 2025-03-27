use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

use std::time::Duration;

fn main() {
    std::thread::spawn(|| {
        let mut v = Vec::new();
        for _ in 0..1_000_000 {
            v.push(0);
        }
    });
    
    loop {
        println!("Memory usage: {:?} KB of RAM", PEAK_ALLOC.current_usage_as_kb());
        std::thread::sleep(Duration::from_secs(1));
    }
}
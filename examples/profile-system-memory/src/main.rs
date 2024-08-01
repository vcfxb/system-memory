
use indicatif::HumanBytes;
use memory_stats::MemoryStats;
use system_memory;
use std::{hint::black_box, sync::atomic::{AtomicU64, AtomicUsize, Ordering}, thread, time::Instant};

static MAX_PHYSICAL_MEM: AtomicUsize = AtomicUsize::new(0);
static SAMPLES: AtomicU64 = AtomicU64::new(0);

fn main() {
    let start = Instant::now();
    
    // Spawn a thread to profile this process
    thread::spawn(|| loop {
        let MemoryStats { physical_mem, .. } = memory_stats::memory_stats().unwrap();

        SAMPLES.fetch_add(1, Ordering::Relaxed);
        
        if physical_mem > MAX_PHYSICAL_MEM.load(Ordering::SeqCst) {
            MAX_PHYSICAL_MEM.store(physical_mem, Ordering::SeqCst);
        }
    });

    let (total, available) = (black_box(system_memory::total()), black_box(system_memory::available()));

    println!("Sys mem: {}/{}", HumanBytes(available), HumanBytes(total));

    println!("Max mem usage {} over {} samples", HumanBytes(MAX_PHYSICAL_MEM.load(Ordering::SeqCst) as u64), SAMPLES.load(Ordering::Relaxed));

    println!("Elapsed time: {:?}", start.elapsed());
}

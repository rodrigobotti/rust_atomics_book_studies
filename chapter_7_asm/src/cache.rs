use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::{hint::black_box, sync::atomic::AtomicU64, time::Instant};

static A: AtomicU64 = AtomicU64::new(0);

pub fn perf_atomic() {
    // backbox: make the compiler treat it as a blackbox
    // that way it won't perform optimizations
    // e.g. remove the loop since we're not using the value
    // e.g. assume the value of A is always zero

    black_box(&A);
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

pub fn perf_atomic_background_read() {
    // The background thread has no significant effect on the main thread.
    // They presumably each run on a separate processor core, but the caches of both cores contain a copy of A,
    // allowing for very fast access.

    black_box(&A);

    thread::spawn(|| {
        // New!
        loop {
            black_box(A.load(Relaxed));
        }
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

pub fn perf_atomic_background_store() {
    // This time, we do see a significant difference
    black_box(&A);
    thread::spawn(|| {
        loop {
            A.store(0, Relaxed); // New!
        }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

pub fn perf_atomic_background_cmp_exc() {
    // also affects cache even if the store on the compare exchange never hapens
    // the instruction(s) of compare_exchange will claim exclusive access of the relevant cache line regardless of whether the comparison succeeds or not.
    black_box(&A);
    thread::spawn(|| {
        loop {
            // Never succeeds, because A is never 10.
            black_box(A.compare_exchange(10, 20, Relaxed, Relaxed).is_ok());
        }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

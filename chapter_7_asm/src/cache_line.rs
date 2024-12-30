use std::sync::atomic::Ordering::Relaxed;
use std::{hint::black_box, sync::atomic::AtomicU64, thread, time::Instant};

static A: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];

pub fn perf_atomic_cache_line_shared() {
    // claims exclusive access to the cache line(s) containing A[0] and A[2],
    // which also contains A[1], slowing down "unrelated" operations on A[1].
    // This effect is called false sharing

    black_box(&A);
    thread::spawn(|| loop {
        A[0].store(0, Relaxed);
        A[2].store(0, Relaxed);
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

#[repr(align(64))] // This struct must be 64-byte aligned.
struct Aligned(AtomicU64);

static B: [Aligned; 3] = [
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
];

pub fn perf_atomic_cache_lines() {
    // each element occupies a separate cache line

    black_box(&A);
    thread::spawn(|| loop {
        B[0].0.store(1, Relaxed);
        B[2].0.store(1, Relaxed);
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(B[1].0.load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}

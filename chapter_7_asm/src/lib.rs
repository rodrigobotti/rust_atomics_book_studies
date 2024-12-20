use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[no_mangle]
pub fn simple_add_ten(num: &mut i32) {
    *num += 10;
}

#[no_mangle]
pub fn simple_store(x: &mut i32) {
    *x = 0;
}

#[no_mangle]
pub fn relaxed_atomic_store(x: &AtomicI32) {
    x.store(0, Relaxed);
}

#[no_mangle]
pub fn simple_load(x: &i32) -> i32 {
    *x
}

#[no_mangle]
pub fn relaxed_atomic_load(x: &AtomicI32) -> i32 {
    x.load(Relaxed)
}

#[no_mangle]
pub fn relaxed_atomic_add_ten(x: &AtomicI32) -> i32 {
    x.fetch_add(10, Relaxed)
}

#[no_mangle]
pub fn relaxed_atomic_fetch_or(x: &AtomicI32) -> i32 {
    // compiled in x86-64 as a lock-prefixed compare-exchange loop
    // because x86-64 has no lock-prefixed equivalent for multiple bits or
    x.fetch_or(10, Relaxed)
}

// equivalent to `relaxed_atomic_fetch_or`
// generates the same x86-64 assembly
#[no_mangle]
pub fn relaxed_atomic_compare_exchange(x: &AtomicI32) -> i32 {
    let mut current = x.load(Relaxed);
    loop {
        let new = current | 10;
        match x.compare_exchange(current, new, Relaxed, Relaxed) {
            Ok(v) => return v,
            Err(v) => current = v,
        }
    }
}

#[no_mangle]
pub fn atomic_relaxed_compare_exchange_weak(x: &AtomicI32) {
    let _ = x.compare_exchange_weak(5, 6, Relaxed, Relaxed);
}

#[no_mangle]
pub fn atomic_relaxed_compare_exchange_strong(x: &AtomicI32) {
    let _ = x.compare_exchange(5, 6, Relaxed, Relaxed);
}

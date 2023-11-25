use std::thread;

use chapter_4_spinlock::SpinLock;

fn main() {
    let lock = SpinLock::new(Vec::new());

    thread::scope(|s| {
        s.spawn(|| {
            lock.lock().push(1)
            // guard is dropped here: lock is unlocked
        });
        s.spawn(|| {
            let mut guard = lock.lock();
            guard.push(2);
            guard.push(2);
            // guard is dropped here: lock is unlocked
        });
    });

    let guard = lock.lock();
    let slice = guard.as_slice();
    assert!(slice == [1, 2, 2] || slice == [2, 2, 1]);

    println!("{:?}", slice);
}

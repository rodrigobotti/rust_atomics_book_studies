use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::{cell::UnsafeCell, sync::atomic::AtomicBool};

pub struct SpinLock<T> {
    value: UnsafeCell<T>,
    locked: AtomicBool,
}

impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        SpinLock {
            value: UnsafeCell::new(value),
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> LockGuard<T> {
        // equivalent to `self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err()``
        while self.locked.swap(true, Acquire) {
            // can be used to inform the processor of a spin loop, which might increase its efficiency
            std::hint::spin_loop();
        }
        LockGuard { lock: self }
    }
}

// so we can share references between threads
// it is ok to do so as long as the value can be transfered between threads
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

// SpinLock lives for >= LockGuard
pub struct LockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the very existence of this guard
        // guarantees we've exclusively locked the lock
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for LockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: the very existence of this guard
        // guarantees we've exclusively locked the lock
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release)
    }
}

use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>, // unsafe barebones alternative to Option<T>
    ready: AtomicBool,                   // has a message to be received
    in_use: AtomicBool,                  // already wrote a message
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
            in_use: AtomicBool::new(false),
        }
    }

    /// Panics when trying to send more than one message.
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Relaxed) {
            panic!("can't send more than one message!");
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Relaxed)
    }

    //// Panics if no message is available yet,
    /// or if the message was already consumed.
    ///
    /// Tip: Use `is_ready` to check first.
    pub fn receive(&self) -> T {
        // better to panic than lead to undefined behavior

        // swap makes sure release can only be called once
        // if called more than once, it panics
        if !self.ready.swap(false, Acquire) {
            panic!("no message available!");
        }
        // Safety: We've just checked (and reset) the ready flag.
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

// why implement drop?
// to drop the message if it was sent but never received avoiding leaks
// (MaybeUninit does not drop the writen value when dropped)
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        // don't need atomic operation:
        // object can only be dropped if it is fully owned by whichever thread is dropping it, with no outstanding borrows
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

impl<T> Default for Channel<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

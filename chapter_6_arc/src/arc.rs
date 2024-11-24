use std::sync::atomic::fence;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{ops::Deref, ptr::NonNull, sync::atomic::AtomicUsize};

use crate::utils::non_null_from;

struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

pub struct Arc<T> {
    // raw pointer.
    // can't use Box: exclusive ownership, not shared
    // can't use reference: not borowing data owned by something else + can't represent the lifetime (until last clone of Arc is dropped).
    ptr: NonNull<ArcData<T>>,
}

// Sending an Arc<T> across threads results in a T object being shared, requiring T to be Sync.
// Similarly, sending an Arc<T> across threads could result in another thread dropping that T,
// effectively transferring it to the other thread, requiring T to be Send.
// In other words, Arc<T> should be Send if and only if T is both Send and Sync.
// The exact same holds for Sync, since a shared &Arc<T> can be cloned into a new Arc<T>.
unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> ArcData<T> {
    pub fn new(data: T) -> Self {
        ArcData {
            ref_count: AtomicUsize::new(1),
            data,
        }
    }
}
impl<T> Default for ArcData<T>
where
    T: Default,
{
    fn default() -> Self {
        ArcData {
            ref_count: AtomicUsize::new(1),
            data: Default::default(),
        }
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Arc {
            ptr: non_null_from(ArcData::new(data)),
        }
    }

    fn data(&self) -> &ArcData<T> {
        // We know the pointer will always point to a valid ArcData<T> as long as the Arc object exists.
        // However, this is not something the compiler knows or checks for us,
        // so accessing the ArcData through the pointer requires unsafe code.
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(Relaxed) == 1 {
            fence(Acquire);
            // Safety: Nothing else can access the data, since
            // there's only one Arc, to which we have exclusive access.
            unsafe { Some(&mut arc.ptr.as_mut().data) }
        } else {
            None
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // increment atomic reference count
        // abort process if we get close to an overflow
        if self.data().ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        // every single drop of one of the former Arc clones must have happened before the final drop.
        // So, the final fetch_sub must establish a happens-before relationship with every previous fetch_sub operation,
        // which we can do using release and acquire ordering

        // decrement atomic reference counter
        if self.data().ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            // if it's the last Arc: reclaim ownership of the pointer using a Box
            // and drop it
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DetectDrop;

        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Relaxed);
            }
        }

        // Create two Arcs sharing an object containing a string
        // and a DetectDrop, to detect when it's dropped.
        let x = Arc::new(("hello", DetectDrop));
        let y = x.clone();

        // Send x to another thread, and use it there.
        let t = std::thread::spawn(move || {
            assert_eq!(x.0, "hello");
        });

        // In parallel, y should still be usable here.
        assert_eq!(y.0, "hello");

        // Wait for the thread to finish.
        t.join().unwrap();

        // One Arc, x, should be dropped by now.
        // We still have y, so the object shouldn't have been dropped yet.
        assert_eq!(NUM_DROPS.load(Relaxed), 0);

        // Drop the remaining `Arc`.
        drop(y);

        // Now that `y` is dropped too,
        // the object should've been dropped.
        assert_eq!(NUM_DROPS.load(Relaxed), 1);
    }
}

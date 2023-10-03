#![allow(unused)]

fn main() {
    //
}

mod scoped {
    use std::thread;

    pub fn scoped() {
        let numbers = vec![1, 2, 3];

        thread::scope(|s| {
            // 1
            s.spawn(|| {
                // 2
                println!("length: {}", numbers.len());
                // even if numbers was mutable, mutating it won't compile:
                // mutable borrow here + immutable borrow in the thread below
            });
            s.spawn(|| {
                // 2
                for n in &numbers {
                    println!("{n}");
                }
            });
        }); // 3

        // 1 - We call the std::thread::scope function with a closure. Our closure is directly executed and gets an argument, s, representing the scope.
        // 2 - We use s to spawn threads. The closures can borrow local variables like numbers.
        // 3 - When the scope ends, all threads that haven’t been joined yet are automatically joined.

        // This pattern guarantees that none of the threads spawned in the scope can outlive the scope.
        // Because of that, this scoped spawn method does not have a 'static bound on its argument type,
        // allowing us to reference anything as long as it outlives the scope, such as numbers.
    }
}

mod sharing {
    // When sharing data between two threads where neither thread is guaranteed to outlive the other, neither of them can be the owner of that data.
    // Any data shared between them will need to live as long as the longest living thread.
    use std::thread;

    static X: [i32; 3] = [1, 2, 3]; // "owned" by the program. every thread can borrow it: guaranteed to ALWAYS exist
    pub fn statics() {
        thread::spawn(|| dbg!(&X));
        thread::spawn(|| dbg!(&X));
    }

    pub fn leaking() {
        // Using Box::leak, one can release ownership of a Box, promising to never drop it.
        // From that point on, the Box will live forever, without an owner,
        // allowing it to be borrowed by any thread for as long as the program runs.
        let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3]));
        thread::spawn(move || dbg!(x));
        thread::spawn(move || dbg!(x));
    }
}

mod reference_counting {
    use std::{rc::Rc, sync::Arc, thread};

    pub fn reference_counting() {
        let a = Rc::new([1, 2, 3]);
        let b = a.clone();
        // Both the original and cloned Rc will refer to the same allocation; they share ownership.
        // When cloned, internal reference counter incremented.
        // When dropped, internal reference counter decremented.
        // Deallocated when all Rcs drop i.e. count=0

        assert_eq!(a.as_ptr(), b.as_ptr()); // Same allocation

        // thread::spawn(move || dbg!(b)); --> compile error
        // If multiple threads had an Rc to the same allocation, they might try to modify the reference counter at the same time, which can give unpredictable results.
    }

    pub fn atomic_reference_counting() {
        let a = Arc::new([1, 2, 3]); // 1
        let b = a.clone(); // 2

        thread::spawn(move || dbg!(a)); // 3
        thread::spawn(move || dbg!(b)); // 3

        // 1 - We put an array in a new allocation together with a reference counter, which starts at one.
        // 2 - Cloning the Arc increments the reference count to two and provides us with a second Arc to the same allocation.
        // 3 - Both threads get their own Arc through which they can access the shared array.
        // Both decrement the reference counter when they drop their Arc.
        // The last thread to drop its Arc will see the counter drop to zero and will be the one to drop and deallocate the array.

        // naming conventions: favor shadowing over naming clone outside
        let c = Arc::new([1, 2, 3]);
        // instead of let d = c.clone(); spawn(move || dbg!(d));
        thread::spawn({
            let c = c.clone();
            move || {
                dbg!(c);
            }
        });
        dbg!(c);
    }

    // Because ownership is shared, reference counting pointers (Rc<T> and Arc<T>) have the same restrictions as shared references (&T).
    // They do not give you mutable access to their contained value, since the value might be borrowed by other code at the same time.
}

mod interior_mutability {
    use std::cell::{Cell, RefCell};

    pub fn cell(a: &Cell<i32>, b: &Cell<i32>) {
        let before = a.get();
        b.set(b.get() + 1);
        let after = a.get();
        if before != after {
            dbg!(a, b); // might happen
        }

        /*
        fn f(a: &i32, b: &mut i32) {
            let before = *a;
            *b += 1;
            let after = *a;
            if before != after {
                x(); // never happens
            }
        }
        */
    }

    pub fn cell_mutable_borrow(v: &Cell<Vec<i32>>) {
        let mut v2 = v.take(); // Replaces the contents of the Cell with an empty Vec. Can't borrow interior value.
        v2.push(1);
        v.set(v2); // Put the modified Vec back
    }

    pub fn ref_cell(v: &RefCell<Vec<i32>>) {
        v.borrow_mut().push(1); // We can modify the `Vec` directly.

        // If you try to borrow it while it is already mutably borrowed (or vice-versa), it will panic, which avoids undefined behavior.
        // Just like a Cell, a RefCell can only be used within a single thread.
    }
}

mod locking {
    use std::{sync::Mutex, thread, time::Duration};

    pub fn mutex() {
        let n = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    // <-- guard is dropped, mutex is unlocked
                });
            }
        });
        assert_eq!(
            n.into_inner().unwrap(), // takes ownership of the mutex data
            1000
        );
    }

    pub fn mutex_long_lock() {
        let n = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });
        // takes about 10s to complete: each thread is waiting the 1s for the mutex to be unlocked
    }

    pub fn mutex_sleep_after_drop() {
        let n = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    drop(guard); // unlocking the mutex before sleeping
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });
        // takes about 1s to complete; sleeps can run in parallel
        // This shows the importance of keeping the amount of time a mutex is locked as short as possible.
        // Keeping a mutex locked longer than necessary can completely nullify any benefits of parallelism,
        // effectively forcing everything to happen serially instead.
    }

    // Mutex::lock returns a Result due to lock poisoning
    // if another thread that held the lock panicked, lock returns an Error
    // This is a mechanism to protect against leaving the data that’s protected by a mutex in an inconsistent state.
    // Trying to recover is rarely done in practice --> letting it panic is more common

    pub fn mutex_pitfalls() {
        let list = Mutex::new(vec![1, 2, 3]);

        // single statement: lock + mutate + unlock
        // unlock happens because the temporary MutexGuard is dropped
        list.lock().unwrap().push(1);
        //                  ^ guard is dropped here

        if let Some(item) = list.lock().unwrap().pop() {
            dbg!(item);
            // if the intetion was: lock + pop + unlock then process the item, the code is incorrect
            // the lock persists until the end of the `if let` block

            // <-- MutexGuard is dropped here
        }

        if list.lock().unwrap().pop() == Some(1) {
            // MutexGuard is already dropped: regular if statements do not borrow values
            // can only produce booleans
            println!("do something");
        }

        // correct implementation of: lock + pop + unlock then process
        let item = list.lock().unwrap().pop();
        //                                          ^ guard is dropped here
        if let Some(item) = item {
            dbg!(item);
        }
    }
}

mod waiting {
    use std::{
        collections::VecDeque,
        sync::{Condvar, Mutex},
        thread,
        time::Duration,
    };

    pub fn parking() {
        let queue = Mutex::new(VecDeque::new());

        thread::scope(|s| {
            // Consuming thread
            let t = s.spawn(|| loop {
                let item = queue.lock().unwrap().pop_front();
                if let Some(item) = item {
                    dbg!(item);
                } else {
                    thread::park();
                }
                // rare: spurious wake-ups regardless of .unpark
            });

            // Producing thread
            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                t.thread().unpark();
                // call to unpark is not lost, it is "recorded"
                // if t attempts to park and there is a recorded unpack request, it won't park
                // unpark calls DO NOT STACK though e.g. unpark, unpark, park (cancelled by recorded unpark, clears unpark request), park (still parks)

                // Unfortunately, this does mean that if unpark() is called right after park() returns,
                // but before the queue gets locked and emptied out,
                // the unpark() call was unnecessary but still causes the next park() call to instantly return.
                // This results in the (empty) queue getting locked and unlocked an extra time.
                // While this doesn’t affect the correctness of the program, it does affect its efficiency and performance.
                thread::sleep(Duration::from_secs(1));
            }
        });

        // thread::park_timeout(duration)
    }

    pub fn condition_variable() {
        let queue = Mutex::new(VecDeque::new());
        let not_empty = Condvar::new();

        thread::scope(|s| {
            s.spawn(|| loop {
                let mut q = queue.lock().unwrap();
                let item = loop {
                    if let Some(item) = q.pop_front() {
                        break item;
                    } else {
                        q = not_empty.wait(q).unwrap();
                    }
                };
                drop(q); // droping before processing
                dbg!(item);
            });

            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                not_empty.notify_one();
                thread::sleep(Duration::from_secs(1));
            }
        });

        // CondVar::wait_timeout(duration)
    }
}

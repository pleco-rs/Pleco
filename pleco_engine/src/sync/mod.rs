//! Useful synchronization primitives.

use std::sync::{Condvar, Mutex};

/// A `LockLatch` starts as false and eventually becomes true. You can block
/// until it becomes true.
pub struct LockLatch {
    m: Mutex<bool>,
    v: Condvar,
}

impl LockLatch {
    #[inline]
    pub fn new() -> LockLatch {
        LockLatch {
            m: Mutex::new(false),
            v: Condvar::new(),
        }
    }

    /// Block until latch is set.
    #[inline]
    pub fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while !*guard {
            guard = self.v.wait(guard).unwrap();
        }
    }

    // Sets the lock to true and notifies any threads waiting on it.
    #[inline]
    pub fn set(&self) {
        let mut guard = self.m.lock().unwrap();
        *guard = true;
        self.v.notify_all();
    }

    // Locks the latch, causing threads to await its unlocking.
    #[inline]
    pub fn lock(&self) {
        let mut guard = self.m.lock().unwrap();
        *guard = false;
    }

    #[inline]
    fn new_value(value: bool) -> LockLatch {
        LockLatch {
            m: Mutex::new(value),
            v: Condvar::new(),
        }
    }

    #[inline]
    fn set_value(&self, value: bool) {
        let mut guard = self.m.lock().unwrap();
        *guard = value;
        self.v.notify_all();
    }

    #[inline]
    fn await_value(&self, value: bool) {
        let mut guard = self.m.lock().unwrap();
        while *guard != value {
            guard = self.v.wait(guard).unwrap();
        }
    }
}

/// A `GuardedBool` allows for waiting on a specific bool value.
pub struct GuardedBool {
    a: LockLatch,
}

impl GuardedBool {
    #[inline]
    pub fn new(value: bool) -> GuardedBool {
        GuardedBool {
            a: LockLatch::new_value(value),
        }
    }

    /// Sets the value.
    #[inline]
    pub fn set(&self, value: bool) {
        self.a.set_value(value);
    }

    /// Awaits a value.
    #[inline]
    pub fn wait(&self, value: bool) {
        self.a.await_value(value);
    }
}

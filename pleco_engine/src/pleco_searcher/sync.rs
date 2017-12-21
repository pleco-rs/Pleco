use std::sync::{Mutex,Condvar};

/// A Latch starts as false and eventually becomes true. You can block
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

    #[inline]
    pub fn set(&self) {
        let mut guard = self.m.lock().unwrap();
        *guard = true;
        self.v.notify_all();
    }

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

pub struct GuardedBool {
    a: LockLatch
}

impl GuardedBool {
    #[inline]
    pub fn new(value: bool) -> GuardedBool {
        GuardedBool {
           a: LockLatch::new_value(value)
        }
    }

    #[inline]
    pub fn set(&self, value: bool) {
        self.a.set_value(value);
    }

    #[inline]
    pub fn await(&self, value: bool) {
        self.a.await_value(value);
    }
}
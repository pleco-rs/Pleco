//! A faster version of `std::sync::Arc`.
//!
//! This is mostly copied from [servo_arc](https://doc.servo.org/servo_arc/index.html), so see
//! that documentation for more information.

use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
#[allow(unused_imports)]
use std::sync::atomic;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// The Inner structure of an `Arc`.
pub struct ArcInner<T: ?Sized> {
    count: atomic::AtomicUsize,
    data: T,
}

/// An `Arc` that ensures a single reference to it. Allows for modification to the
/// state inside, and also transformation into an `Arc`.
pub struct UniqueArc<T: ?Sized>(Arc<T>);

unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}

impl<T> UniqueArc<T> {
    #[inline]
    /// Construct a new UniqueArc
    pub fn new(data: T) -> Self {
        UniqueArc(Arc::new(data))
    }

    #[inline]
    /// Convert to a shareable Arc<T> once we're done using it
    pub fn shareable(self) -> Arc<T> {
        self.0
    }
}

impl<T> Deref for UniqueArc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for UniqueArc<T> {
    fn deref_mut(&mut self) -> &mut T {
        // We know this to be uniquely owned
        unsafe { &mut (*self.0.ptr()).data }
    }
}

/// Reference counting pointer, shareable between threads.
pub struct Arc<T: ?Sized> {
    p: NonNull<ArcInner<T>>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Creates a new `Arc`.
    #[inline]
    pub fn new(data: T) -> Self {
        let x = Box::new(ArcInner {
            count: atomic::AtomicUsize::new(1),
            data,
        });
        unsafe {
            Arc {
                p: NonNull::new_unchecked(Box::into_raw(x)),
            }
        }
    }
}

impl<T: ?Sized> Arc<T> {
    /// Returns a pointer to the inner Arc.
    #[inline]
    fn ptr(&self) -> *mut ArcInner<T> {
        self.p.as_ptr()
    }

    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { &*self.ptr() }
    }

    /// Gets a `& mut T` of the inner value if there is only one reference.
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            unsafe {
                // See make_mut() for documentation of the threadsafety here.
                Some(&mut (*this.ptr()).data)
            }
        } else {
            None
        }
    }

    /// Allows for determining if the reference count is zero.
    #[inline]
    pub fn is_unique(&self) -> bool {
        // We can use Relaxed here, but the justification is a bit subtle.
        //
        // The reason to use Acquire would be to synchronize with other threads
        // that are modifying the refcount with Release, i.e. to ensure that
        // their writes to memory guarded by this refcount are flushed. However,
        // we know that threads only modify the contents of the Arc when they
        // observe the refcount to be 1, and no other thread could observe that
        // because we're holding one strong reference here.
        self.inner().count.load(Relaxed) == 1
    }

    // Non-inlined part of `drop`. Just invokes the destructor.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        let _ = Box::from_raw(self.ptr());
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.inner().count.fetch_add(1, Relaxed);
        unsafe {
            Arc {
                p: NonNull::new_unchecked(self.ptr()),
            }
        }
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    #[inline]
    fn drop(&mut self) {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other threads unless we are going to delete the object.
        if self.inner().count.fetch_sub(1, Release) != 1 {
            return;
        }

        // This load is needed to prevent reordering of use of the data and
        // deletion of the data.  Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` load. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this load, which happens before the
        // deletion of the data.
        //
        // As explained in the [Boost documentation][1],
        //
        // > It is important to enforce any possible access to the object in one
        // > thread (through an existing reference) to *happen before* deleting
        // > the object in a different thread. This is achieved by a "release"
        // > operation after dropping a reference (any access to the object
        // > through this reference must obviously happened before), and an
        // > "acquire" operation before deleting the object.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        // [2]: https://github.com/rust-lang/rust/pull/41714
        self.inner().count.load(Acquire);

        unsafe {
            self.drop_slow();
        }
    }
}

//! Miscellaneous tools for used for Searching. Most notably this module
//! contains the `TranspositionTable`, a fast lookup table able to be accessed by
//! multiple threads. Other useful objects are the `UciLimit` enum and `Searcher` trait
//! for building bots.

pub mod prng;
pub mod tt;
pub mod timer;
mod pawn_table;

use std::ptr::Unique;
use std::heap::{Alloc, Layout, Heap};
use std::cell::UnsafeCell;

use core::piece_move::BitMove;
use tools::timer::Timer;
use board::Board;

/// Defines an object that can play chess.
pub trait Searcher {
    fn name() -> &'static str where Self: Sized;

    fn best_move(board: Board, limit: UCILimit) -> BitMove
        where
            Self: Sized;

    fn best_move_depth(board: Board, max_depth: u16) -> BitMove
        where
            Self: Sized {
        Self::best_move(board, UCILimit::Depth(max_depth))
    }
}

// TODO: Decrement this
/// Defines a Limit for a Searcher. e.g., when a searcher should stop
/// searching.
#[derive(Clone)]
pub enum UCILimit {
    Infinite,
    Depth(u16),
    Nodes(u64),
    Time(Timer),
}

impl UCILimit {
    /// Returns if time management should be used.
    pub fn use_time(&self) -> bool {
        if let UCILimit::Time(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns if the limit is depth.
    pub fn is_depth(&self) -> bool {
        if let UCILimit::Depth(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns the depth limit if there is one, otherwise returns 10000.
    pub fn depth_limit(&self) -> u16 {
        if let UCILimit::Depth(depth) = *self {
            depth
        } else {
            10_000
        }
    }

    /// Returns the Timer for the UCILimit, if there is one to be sent.
    pub fn timer(&self) -> Option<Timer> {
        if let UCILimit::Time(timer) = *self {
            Some(timer.clone())
        } else {
            None
        }
    }
}

// TODO: Performance increase awaiting with const generics: https://github.com/rust-lang/rust/issues/44580

/// Generic Heap-stored array of entries.
pub struct TableBase<T: Sized> {
    table: UnsafeCell<Unique<T>>,
    size: UnsafeCell<usize>
}

impl<T: Sized> TableBase<T> {
    /// Constructs a new `TableBase`. The size must be a power of 2, or else `None` is
    /// returned.
    pub fn new(size: usize) -> Option<TableBase<T>> {
        if size.count_ones() != 1 {
            None
        } else {
            unsafe  {
                let table = TableBase {
                    table: UnsafeCell::new(TableBase::alloc(size)),
                    size: UnsafeCell::new(size)
                };
                Some(table)
            }
        }
    }

    /// Returns the size of the Table.
    #[inline]
    pub fn size(&self) -> usize {
        unsafe {*self.size.get()}
    }

    /// Gets a mutable reference to an entry with a certain key.
    #[inline]
    pub fn get_mut(&self, key: u64) -> &mut T {
        let ptr: *mut T = self.get_ptr(key);
        unsafe {
            &mut *ptr
        }
    }

    /// Gets a mutable pointer to an entry with a certain key.
    #[inline(always)]
    pub fn get_ptr(&self, key: u64) -> *mut T {
        let index: usize = (key & (self.size() as u64 - 1)) as usize;
        unsafe {
            (*self.table.get()).as_ptr().offset(index as isize)
        }
    }

    /// Re-sizes the table to a particular size, which must be a power of 2.
    pub unsafe fn resize(&self, size: usize) {
        assert_eq!(size.count_ones(), 1);
        self.de_alloc();
        *self.table.get() = TableBase::alloc(size);
        *self.size.get() = size;
    }

    unsafe fn alloc(size: usize) -> Unique<T> {
        let ptr = Heap.alloc_zeroed(Layout::array::<T>(size).unwrap());let new_ptr = match ptr {
            Ok(ptr) => ptr,
            Err(err) => Heap.oom(err),
        };
        Unique::new(new_ptr as *mut T).unwrap()
    }

    unsafe fn de_alloc(&self) {
        Heap.dealloc(self.table.get() as *mut _,
                     Layout::array::<T>(*self.size.get()).unwrap());
    }
}

impl<T: Sized> Drop for TableBase<T> {
    fn drop(&mut self) {
        unsafe {
            self.de_alloc();
        }
    }
}

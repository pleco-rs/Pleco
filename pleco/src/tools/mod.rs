//! Miscellaneous tools for used for Searching. Most notably this module
//! contains the `TranspositionTable`, a fast lookup table able to be accessed by
//! multiple threads. Other useful objects are the `UciLimit` enum and `Searcher` trait
//! for building bots.

pub mod prng;
pub mod eval;
pub mod tt;
pub mod timer;
pub mod pawn_table;

use std::ptr::NonNull;
use std::heap::{Alloc, Layout, Heap};

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

/// Generic Heap-stored array of entries. Used for building more specific abstractions.
///
/// Objects placed inside must not implement `Drop`, or else undefined behavior follows. Indexing is done
/// with `u64`s, and returns a value using a mask of the lower log<sub>2</sub>(table size) bits. Collisions
/// are possible using this structure, although very rare.
pub struct TableBase<T: Sized> {
    table: NonNull<T>,
    size: usize
}

impl<T: Sized> TableBase<T> {
    /// Constructs a new `TableBase`. The size must be a power of 2, or else `None` is
    /// returned.
    ///
    /// # Safety
    ///
    /// Size must be a power of 2/
    pub fn new(size: usize) -> Option<TableBase<T>> {
        if size.count_ones() != 1 {
            None
        } else {
            unsafe  {
                let table = TableBase {
                    table: TableBase::alloc(size),
                    size: size
                };
                Some(table)
            }
        }
    }

    /// Returns the size of the Table.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets a mutable reference to an entry with a certain key.
    #[inline]
    pub fn get_mut(&mut self, key: u64) -> &mut T {
        let index: usize =  (key & (self.size as u64 - 1)) as usize;
        unsafe {
            &mut *self.table.as_ptr().offset(index as isize)
        }
    }

    /// Gets a mutable pointer to an entry with a certain key.
    ///
    /// # Safety
    ///
    /// Unsafe due to returning a raw pointer that may dangle if the `TableBase` is
    /// dropped prematurely.
    #[inline]
    pub unsafe fn get_ptr(&self, key: u64) -> *mut T {
        let index: usize = (key & (self.size() as u64 - 1)) as usize;
        self.table.as_ptr().offset(index as isize)
    }

    /// Re-sizes the table to a particular size, which must be a power of 2. Also clears all
    /// the entries inside of the table.
    ///
    /// # Safety
    ///
    /// Panics if `size` is not a power of 2.
    pub fn resize(&mut self, size: usize) {
        assert_eq!(size.count_ones(), 1);
        unsafe {
            self.de_alloc();
            self.table = TableBase::alloc(size);
        }
        self.size = size;
    }

    // allocates space.
    unsafe fn alloc(size: usize) -> NonNull<T> {
        let ptr = Heap.alloc_zeroed(Layout::array::<T>(size).unwrap());
        let new_ptr = match ptr {
            Ok(ptr) => ptr,
            Err(err) => Heap.oom(err),
        };
        NonNull::new(new_ptr as *mut T).unwrap()
    }

    /// de-allocates the current table.
    unsafe fn de_alloc(&mut self) {
        Heap.dealloc(self.table.as_ptr() as *mut _,
                     Layout::array::<T>(self.size).unwrap());
    }
}

impl<T: Sized> Drop for TableBase<T> {
    fn drop(&mut self) {
        unsafe {
            self.de_alloc();
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn table_base_allocs() {
        for i in 0..14 {
            let size: usize = 1 << i;
            let mut t = TableBase::<u64>::new(size).unwrap();
            for x in 0..(3*size) {
                *t.get_mut(x as u64) += 1 as u64;
            }
        }
    }
}
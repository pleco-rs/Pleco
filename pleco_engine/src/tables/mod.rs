pub mod butterfly;
pub mod capture_piece_history;
pub mod continuation;
pub mod counter_move;
pub mod material;
pub mod pawn_table;

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::mem;
use std::ops::*;
use std::ptr;
use std::ptr::NonNull;

pub mod prelude {
    // easier exporting :)
    pub use super::butterfly::ButterflyHistory;
    pub use super::capture_piece_history::CapturePieceToHistory;
    pub use super::continuation::{ContinuationHistory, PieceToHistory};
    pub use super::counter_move::CounterMoveHistory;
    pub use super::{NumStatBoard, NumStatCube, StatBoard};
}

// TODO: Create StatBoards using const generics: https://github.com/rust-lang/rust/issues/44580
// TODO: Create 3DBoard using const generics: https://github.com/rust-lang/rust/issues/44580

pub trait StatBoard<T, IDX>: Sized + IndexMut<IDX, Output = T>
where
    T: Copy + Clone + Sized,
{
    const FILL: T;

    fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    fn clear(&mut self) {
        self.fill(Self::FILL);
    }

    fn fill(&mut self, val: T) {
        let num: usize = mem::size_of::<Self>() / mem::size_of::<T>();

        unsafe {
            let ptr: *mut T = self as *mut Self as *mut T;
            for i in 0..num {
                ptr::write(ptr.add(i), val);
            }
        }
    }
}

pub trait NumStatBoard<IDX>: StatBoard<i16, IDX> {
    const D: i16;
    fn update(&mut self, idx: IDX, bonus: i16) {
        assert!(bonus.abs() <= Self::D); // Ensure range is [-32 * D, 32 * D]
        let entry = self.index_mut(idx);
        *entry += bonus * 32 - (*entry) * bonus.abs() / Self::D;
    }
}

pub trait NumStatCube<IDX>: StatBoard<i16, IDX> {
    const D: i32;
    const W: i32;

    fn update(&mut self, idx: IDX, bonus: i32) {
        assert!(bonus.abs() <= Self::D);
        let entry = self.index_mut(idx);
        *entry += (bonus * Self::W - (*entry) as i32 * bonus.abs() / Self::D) as i16;
        assert!(((*entry) as i32).abs() <= Self::D * Self::W);
    }
}

// TODO: Performance increase awaiting with const generics: https://github.com/rust-lang/rust/issues/44580

/// Generic Heap-stored array of entries. Used for building more specific abstractions.
///
/// Objects placed inside must not implement `Drop`, or else undefined behavior follows. Indexing is done
/// with `u64`s, and returns a value using a mask of the lower log<sub>2</sub>(table size) bits. Collisions
/// are possible using this structure, although very rare.
pub struct TableBase<T: Sized + TableBaseConst> {
    table: NonNull<T>,
}

pub trait TableBaseConst {
    const ENTRY_COUNT: usize;
}

impl<T: Sized + TableBaseConst> TableBase<T> {
    /// Constructs a new `TableBase`. The size must be a power of 2, or else `None` is
    /// returned.
    ///
    /// # Safety
    ///
    /// Size must be a power of 2/
    pub fn new() -> Option<TableBase<T>> {
        if T::ENTRY_COUNT.count_ones() != 1 {
            None
        } else {
            unsafe {
                let table = TableBase {
                    table: TableBase::alloc(),
                };
                Some(table)
            }
        }
    }

    /// Gets a mutable reference to an entry with a certain key.
    #[inline(always)]
    pub fn get_mut(&mut self, key: u64) -> &mut T {
        unsafe { &mut *self.get_ptr(key) }
    }

    /// Gets a mutable pointer to an entry with a certain key.
    ///
    /// # Safety
    ///
    /// Unsafe due to returning a raw pointer that may dangle if the `TableBase` is
    /// dropped prematurely.
    #[inline(always)]
    pub unsafe fn get_ptr(&self, key: u64) -> *mut T {
        let index: usize = (key & (T::ENTRY_COUNT as u64 - 1)) as usize;
        self.table.as_ptr().offset(index as isize)
    }

    pub fn clear(&mut self) {
        unsafe {
            let t_ptr = self.get_ptr(0);
            ptr::write_bytes(t_ptr, 0, T::ENTRY_COUNT);
        }
    }

    // allocates space.
    unsafe fn alloc() -> NonNull<T> {
        let layout = Layout::array::<T>(T::ENTRY_COUNT).unwrap();
        let ptr = alloc_zeroed(layout);
        let new_ptr = ptr.cast();
        NonNull::new(new_ptr as *mut T).unwrap()
    }

    /// de-allocates the current table.
    unsafe fn de_alloc(&mut self) {
        let ptr: NonNull<u8> = mem::transmute(self.table);
        dealloc(ptr.as_ptr(), Layout::array::<T>(T::ENTRY_COUNT).unwrap());
    }
}

impl<T: Sized + TableBaseConst> Drop for TableBase<T> {
    fn drop(&mut self) {
        unsafe {
            self.de_alloc();
        }
    }
}

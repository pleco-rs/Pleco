
pub mod pawn_table;
pub mod material;

use std::ptr::NonNull;
use std::heap::{Alloc, Layout, Heap};

use pleco::core::masks::*;


// TODO: Create StatBoards using const generics: https://github.com/rust-lang/rust/issues/44580
// TODO: Create 3DBoard using const generics: https://github.com/rust-lang/rust/issues/44580

// ButterflyHistory
// CapturePieceToHistory
// CounterMoveHistory
// ContinuationHistory
pub struct ButterflyHistory {
    d: [[i16; SQ_CNT * SQ_CNT]; PLAYER_CNT]
}

pub struct PieceToBoards {
    d: [[[i16; PIECE_TYPE_CNT]; PLAYER_CNT]; SQ_CNT]
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
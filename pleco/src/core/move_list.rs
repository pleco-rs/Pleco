//! Contains the `MoveList` structure, akin to a `Vec<BitMove>` but faster for our purposes.
//!
//! A [`MoveList`] structure is guaranteed to be exactly 512 bytes long, containing a maximum of 256
//! moves. This number was chosen as no possible chess position has been found to contain more than
//! 232 possible moves.
//!
//! This structure is intended to mainly be used for generation of moves for a certain position. If
//! you need to a more versatile collection of moves to manipulate, considering using a `Vec<BitMove>`
//! instead.
//!
//! [`MoveList`]: struct.MoveList.html

use super::piece_move::BitMove;

use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator};


const MAX_MOVES: usize = 256;

/// This is the list of possible moves for a current position. Think of it alike a faster
/// version of `Vec<BitMove>`, as all the data is stored in the Stack rather than the Heap.
pub struct MoveList {
    inner: [BitMove; 256],
    len: usize,
}

impl Default for MoveList {
    #[inline]
    fn default() -> Self {
        MoveList {
            inner: [BitMove::null(); 256],
            len: 0,
        }
    }
}

impl From<Vec<BitMove>> for MoveList {
    fn from(vec: Vec<BitMove>) -> Self {
        let mut list = MoveList::default();
        vec.iter().for_each(|m| list.push(*m));
        list
    }
}

impl Into<Vec<BitMove>> for MoveList {
    #[inline]
    fn into(self) -> Vec<BitMove> {
        self.vec()
    }
}

impl MoveList {
    /// Adds a `BitMove` to the end of the list.
    #[inline]
    pub fn push(&mut self, mov: BitMove) {
        unsafe {
            let end = self.inner.get_unchecked_mut(self.len);
            *end = mov;
        }
        self.len += 1;
    }

    /// Creates a vector from this `MoveList`.
    pub fn vec(&self) -> Vec<BitMove> {
        let mut vec = Vec::with_capacity(self.len);
        for mov in self.iter() {
            vec.push(*mov);
        }
        assert_eq!(vec.len(),self.len);
        vec
    }

    /// Returns the number of moves inside the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the `MoveList` as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[BitMove] {
        self
    }
}

impl Deref for MoveList {
    type Target = [BitMove];

    #[inline]
    fn deref(&self) -> &[BitMove] {
        unsafe {
            let p = self.inner.as_ptr();
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [BitMove] {
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Index<usize> for MoveList {
    type Output = BitMove;

    #[inline]
    fn index(&self, index: usize) -> &BitMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for MoveList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut BitMove {
        &mut (**self)[index]
    }
}

pub struct MoveIter<'a> {
    movelist: &'a MoveList,
    idx: usize
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = BitMove;
    type IntoIter = MoveIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MoveIter {
            movelist: &self,
            idx: 0,
        }
    }
}

impl<'a> ExactSizeIterator for MoveIter<'a> {}

impl<'a> FusedIterator for MoveIter<'a> {}

unsafe impl<'a> TrustedLen for MoveIter<'a> {}

// Iterator for the `MoveList`.
pub struct MoveIntoIter {
    movelist: MoveList,
    idx: usize
}

impl Iterator for MoveIntoIter {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl ExactSizeIterator for MoveIntoIter {}

impl FusedIterator for MoveIntoIter {}

unsafe impl TrustedLen for MoveIntoIter {}

impl IntoIterator for MoveList {
    type Item = BitMove;
    type IntoIter = MoveIntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MoveIntoIter {
            movelist: self,
            idx: 0,
        }
    }
}
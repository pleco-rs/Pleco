use super::{RootMove, MAX_MOVES};

use pleco::MoveList;

use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator};
use std::ptr;

#[repr(C)]
pub struct RawRootMoveList {
    len: u32, // 4 bytes
    pad: [u8; 12], // 12 bytes
    moves: [RootMove; MAX_MOVES], // 4096 bytes
    bottom_pad: [u8; 54] // 48 bytes
}

pub struct RootMoveList {
    pub moves: *mut RawRootMoveList
}

impl RootMoveList {
    pub fn len(&self) -> usize {
        unsafe {(*self.moves).len as usize}
    }

    pub fn clone_from(&mut self, other: &RootMoveList) {
        unsafe {
            ptr::copy_nonoverlapping(other.moves, self.moves, 1);
        }
    }

    pub fn replace(&mut self, moves: &MoveList) {
        unsafe {
            (*self.moves).len = moves.len() as u32;
            for (i, mov) in moves.iter().enumerate() {
                self[i] = RootMove::new(*mov);
            }
        }
    }
}

impl Deref for RootMoveList {
    type Target = [RootMove];

    #[inline]
    fn deref(&self) -> &[RootMove] {
        unsafe {
            let p = (*self.moves).moves.as_ptr();
            slice::from_raw_parts(p, self.len())
        }
    }
}

impl DerefMut for RootMoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [RootMove] {
        unsafe {
            let p = (*self.moves).moves.as_mut_ptr();
            slice::from_raw_parts_mut(p, self.len())
        }
    }
}

impl Index<usize> for RootMoveList {
    type Output = RootMove;

    #[inline]
    fn index(&self, index: usize) -> &RootMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for RootMoveList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut RootMove {
        &mut (**self)[index]
    }
}

pub struct MoveIter<'a> {
    movelist: &'a RootMoveList,
    idx: usize,
    len: usize
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = RootMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.idx, Some(self.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a RootMoveList {
    type Item = RootMove;
    type IntoIter = MoveIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        MoveIter {
            movelist: &self,
            idx: 0,
            len: self.len()
        }
    }
}

impl<'a> ExactSizeIterator for MoveIter<'a> {}

impl<'a> FusedIterator for MoveIter<'a> {}

unsafe impl<'a> TrustedLen for MoveIter<'a> {}
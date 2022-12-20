use std::iter::{ExactSizeIterator, FusedIterator, IntoIterator, Iterator};
use std::mem;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{RootMove, MAX_MOVES};
use pleco::{BitMove, MoveList};

pub struct RootMoveList {
    len: AtomicUsize,
    moves: [RootMove; MAX_MOVES],
}

impl Clone for RootMoveList {
    fn clone(&self) -> Self {
        RootMoveList {
            len: AtomicUsize::new(self.len.load(Ordering::SeqCst)),
            moves: self.moves,
        }
    }
}

unsafe impl Send for RootMoveList {}
unsafe impl Sync for RootMoveList {}

impl RootMoveList {
    /// Creates an empty `RootMoveList`.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            RootMoveList {
                len: AtomicUsize::new(0),
                moves: [mem::MaybeUninit::uninit().assume_init(); MAX_MOVES],
            }
        }
    }

    /// Returns the length of the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    /// Replaces the current `RootMoveList` with another `RootMoveList`.
    pub fn clone_from_other(&mut self, other: &RootMoveList) {
        self.len.store(other.len(), Ordering::SeqCst);
        unsafe {
            let self_moves: *mut [RootMove; MAX_MOVES] =
                self.moves.as_mut_ptr() as *mut [RootMove; MAX_MOVES];
            let other_moves: *const [RootMove; MAX_MOVES] =
                other.moves.as_ptr() as *const [RootMove; MAX_MOVES];
            ptr::copy_nonoverlapping(other_moves, self_moves, 1);
        }
    }

    /// Replaces the current `RootMoveList` with the moves inside a `MoveList`.
    pub fn replace(&mut self, moves: &MoveList) {
        self.len.store(moves.len(), Ordering::SeqCst);
        for (i, mov) in moves.iter().enumerate() {
            self[i] = RootMove::new(*mov);
        }
    }

    /// Applies `RootMove::rollback()` to each `RootMove` inside.
    #[inline]
    pub fn rollback(&mut self) {
        self.iter_mut().for_each(|b| b.prev_score = b.score);
    }

    /// Returns the first `RootMove` in the list.
    ///
    /// # Safety
    ///
    /// May return a nonsense `RootMove` if the list hasn't been initialized since the start.
    #[inline]
    pub fn first(&mut self) -> &mut RootMove {
        unsafe { self.get_unchecked_mut(0) }
    }

    /// Converts to a `MoveList`.
    pub fn to_list(&self) -> MoveList {
        let vec = self.iter().map(|m| m.bit_move).collect::<Vec<BitMove>>();
        MoveList::from(vec)
    }

    /// Returns the previous best score.
    #[inline]
    pub fn prev_best_score(&self) -> i32 {
        unsafe { self.get_unchecked(0).prev_score }
    }

    #[inline]
    pub fn insert_score_depth(&mut self, index: usize, score: i32, depth: i16) {
        unsafe {
            let rm: &mut RootMove = self.get_unchecked_mut(index);
            rm.score = score;
            rm.depth_reached = depth;
        }
    }

    #[inline]
    pub fn insert_score(&mut self, index: usize, score: i32) {
        unsafe {
            let rm: &mut RootMove = self.get_unchecked_mut(index);
            rm.score = score;
        }
    }

    pub fn find(&mut self, mov: BitMove) -> Option<&mut RootMove> {
        self.iter_mut().find(|m| m.bit_move == mov)
    }
}

impl Deref for RootMoveList {
    type Target = [RootMove];

    #[inline]
    fn deref(&self) -> &[RootMove] {
        unsafe {
            let p = self.moves.as_ptr();
            slice::from_raw_parts(p, self.len())
        }
    }
}

impl DerefMut for RootMoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [RootMove] {
        unsafe {
            let p = self.moves.as_mut_ptr();
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
    len: usize,
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
            len: self.len(),
        }
    }
}

impl<'a> ExactSizeIterator for MoveIter<'a> {}

impl<'a> FusedIterator for MoveIter<'a> {}

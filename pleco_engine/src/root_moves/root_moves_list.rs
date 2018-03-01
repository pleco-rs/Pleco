
use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator};
use std::ptr;
use std::mem;

use std::mem::transmute;
use std::sync::atomic::{Ordering,AtomicUsize};

use rand;
use rand::Rng;

use pleco::{MoveList, Board, PieceType, BitMove};
use super::{RootMove, MAX_MOVES};


pub struct RootMoveList {
    len: AtomicUsize,
    moves: [RootMove; MAX_MOVES],
}

impl Clone for RootMoveList {
    fn clone(&self) -> Self {
        RootMoveList {
            len: AtomicUsize::new(self.len.load(Ordering::SeqCst)),
            moves: self.moves
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
                moves: [mem::uninitialized(); MAX_MOVES],
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
            let self_moves: *mut [RootMove; MAX_MOVES] = transmute::<*mut RootMove, *mut [RootMove; MAX_MOVES]>(self.moves.as_mut_ptr());
            let other_moves: *const [RootMove; MAX_MOVES] =  transmute::<*const RootMove, *const [RootMove; MAX_MOVES]>(other.moves.as_ptr());
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
        self.iter_mut()
            .for_each(|b| b.prev_score = b.score);
    }

    /// Returns the first `RootMove` in the list.
    ///
    /// # Safety
    ///
    /// May return a nonsense `RootMove` if the list hasn't been initalized since the start.
    #[inline]
    pub fn first(&mut self) -> &mut RootMove {
        unsafe {
            self.get_unchecked_mut(0)
        }
    }

    /// Applied a `Most Valuable Victim - Leave Valuable Attacker` sort to the list.
    #[inline]
    pub fn mvv_lva_sort(&mut self, board: &Board) {
        self.sort_by_key(|root_move| {
            let a = root_move.bit_move;
            let piece = board.piece_at_sq((a).get_src()).unwrap();

            if a.is_capture() {
                piece.value() - board.captured_piece(a).unwrap().value()
            } else if a.is_castle() {
                1
            } else if piece == PieceType::P {
                if a.is_double_push().0 {
                    2
                } else {
                    3
                }
            } else {
                4
            }
        });
    }

    /// Shuffles the moves arounf.
    #[inline]
    pub fn shuffle(&mut self, thread_id: usize, board: &Board) {
        if thread_id == 0 || thread_id >= 20 {
            self.mvv_lva_sort(board);
        } else {
            rand::thread_rng().shuffle(self.as_mut());
        }
    }

    /// Converts to a `MoveList`.
    pub fn to_list(&self) -> MoveList {
        let vec =  self.iter().map(|m| m.bit_move).collect::<Vec<BitMove>>();
        MoveList::from(vec)
    }

    /// Returns the previous best score.
    #[inline]
    pub fn prev_best_score(&self) -> i32 {
        unsafe {
            self.get_unchecked(0).prev_score
        }
    }

    #[inline]
    pub fn insert_score_depth(&mut self, index: usize, score: i32, depth: u16) {
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
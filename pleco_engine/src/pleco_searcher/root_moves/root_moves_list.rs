use super::{RootMove, MAX_MOVES};
use super::super::sync::GuardedBool;

use pleco::{MoveList,Board,Piece,BitMove};

use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator};
use std::ptr;

use std::mem::transmute;
use std::sync::atomic::{AtomicU16,Ordering,fence};

use rand;
use rand::Rng;

pub struct RawRootMoveList {
    len: u32, // 4 bytes
    depth_completed: AtomicU16, // 2 bytes
    pub finished: GuardedBool, // 1 byte
    moves: [RootMove; MAX_MOVES], // 4096 bytes
}

impl RawRootMoveList {
    pub fn init(&mut self) {
        self.depth_completed = AtomicU16::new(0);
        unsafe {
            let f = &mut self.finished;
            ptr::write_volatile(f, GuardedBool::new(true));
        }
    }
}

pub struct RootMoveList {
    pub moves: *mut RawRootMoveList
}

impl Clone for RootMoveList {
    fn clone(&self) -> Self {
        RootMoveList {
            moves: self.moves
        }
    }
}


unsafe impl Send for RootMoveList {}

impl RootMoveList {
    pub unsafe fn init(&mut self) {
        (*self.moves).init();
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe {(*self.moves).len as usize}
    }

    pub fn clone_from_other(&mut self, other: &RootMoveList) {
        unsafe {
            (*self.moves).len = other.len() as u32;
            let self_moves: *mut [RootMove; MAX_MOVES] = transmute::<*mut RootMove, *mut [RootMove; MAX_MOVES]>((*self.moves).moves.as_mut_ptr());
            let other_moves: *mut [RootMove; MAX_MOVES] =  transmute::<*mut RootMove, *mut [RootMove; MAX_MOVES]>((*other.moves).moves.as_mut_ptr());
            ptr::copy_nonoverlapping(other_moves, self_moves, 1);
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

    #[inline]
    pub fn rollback(&mut self) {
        self.iter_mut()
            .for_each(|b| b.prev_score = b.score);
    }

    #[inline]
    pub fn first(&mut self) -> &mut RootMove {
        unsafe {
            self.get_unchecked_mut(0)
        }
    }

    #[inline]
    pub fn depth_completed(self) -> u16 {
        unsafe {
            (*self.moves).depth_completed.load(Ordering::SeqCst)
        }
    }

    #[inline]
    pub fn set_depth_completed(&mut self, depth: u16) {
        unsafe {
            fence(Ordering::SeqCst);
            (*self.moves).depth_completed.store(depth, Ordering::SeqCst);
        }
    }

    #[inline]
    pub fn set_finished(&mut self, finished: bool) {
        unsafe {
            (*self.moves).finished.set(finished);
        }
    }

    #[inline]
    pub fn mvv_laa_sort(&mut self, board: &Board) {
        self.sort_by_key(|root_move| {
            let a = root_move.bit_move;
            let piece = board.piece_at_sq((a).get_src()).unwrap();

            if a.is_capture() {
                piece.value() - board.captured_piece(a).unwrap().value()
            } else if a.is_castle() {
                1
            } else if piece == Piece::P {
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

    #[inline]
    pub fn shuffle(&mut self, thread_id: usize, board: &Board) {
        if thread_id == 0 || thread_id >= 20 {
            self.mvv_laa_sort(board);
        } else {
            rand::thread_rng().shuffle(self.as_mut());
        }
    }

    pub fn to_list(&self) -> MoveList {
        let vec =  self.iter().map(|m| m.bit_move).collect::<Vec<BitMove>>();
        MoveList::from(vec)
    }

    pub fn prev_best_score(&self) -> i32 {
        unsafe {
            self.get_unchecked(0).prev_score
        }
    }

    pub fn insert_score_depth(&mut self, index: usize, score: i32, depth: u16) {
        unsafe {
            let rm: &mut RootMove = self.get_unchecked_mut(index);
            rm.score = score;
            rm.depth_reached = depth;

        }
    }

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
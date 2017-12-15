use super::{MAX_THREADS,MAX_MOVES,RootMove};
use super::root_moves_list::{RootMoveList,RawRootMoveList};

use std::heap::{Alloc, Layout, Heap};
use std::cell::UnsafeCell;
use std::ptr::Unique;
use std::mem;

use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator};

use pleco::{BitMove,Board};
use pleco::board::movegen::{MoveGen,Legal,PseudoLegal,Legality};
use pleco::core::mono_traits::{GenTypeTrait,AllGenType};

struct RawRmManager {
    pub rms: [RawRootMoveList; MAX_THREADS]
}

impl RawRmManager {
    pub fn new() -> Unique<RawRmManager> {
        unsafe {
            let ptr = Heap.alloc_zeroed(Layout::array::<RawRootMoveList>(MAX_THREADS).unwrap());

            let new_ptr = match ptr {
                Ok(ptr) => ptr,
                Err(err) => Heap.oom(err),
            };
            Unique::new(new_ptr as *mut RawRmManager).unwrap()
        }
    }
}


pub struct RmManager {
    threads: usize,
    moves: Unique<RawRmManager>
}

impl RmManager {
    pub fn new() -> Self {
        RmManager {
            threads: 0,
            moves: RawRmManager::new()
        }
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn add_thread(&mut self) -> Option<RootMoveList> {
        if self.threads >= MAX_THREADS {
            None
        } else {
            self.threads += 1;
            unsafe {
                Some(self.get_list_unchecked(self.threads))
            }
        }
    }

    pub fn get_list(&self, num: usize) -> Option<RootMoveList> {
        if num >= self.threads {
            None
        } else {
            unsafe {
                Some(self.get_list_unchecked(num))
            }
        }
    }

    pub unsafe fn get_list_unchecked(&self, num: usize) -> RootMoveList {
        RootMoveList {
            moves: self.as_raw_ptr().offset(num as isize)
        }
    }

    pub unsafe fn replace_moves(&mut self, board: &Board) {
        let legal_moves = MoveGen::generate::<PseudoLegal, AllGenType>(&board);
    }

    unsafe fn as_ptr(&self) -> RootMoveList {
        RootMoveList {
            moves: mem::transmute::<*mut RawRmManager, *mut RawRootMoveList>(self.moves.as_ptr())
        }
    }

    unsafe fn as_raw_ptr(&self) -> *mut RawRootMoveList {
        mem::transmute::<*mut RawRmManager, *mut RawRootMoveList>(self.moves.as_ptr())
    }
}

pub struct RootMovesIter<'a> {
    root_moves: &'a RmManager,
    idx: usize,
}

impl<'a> Iterator for RootMovesIter<'a> {
    type Item = RootMoveList;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.root_moves.threads() {
            None
        } else {
            unsafe {
                Some(self.root_moves.get_list_unchecked(self.idx))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.root_moves.threads() - self.idx, Some(self.root_moves.threads() - self.idx))
    }
}

impl<'a> IntoIterator for &'a RmManager {
    type Item = RootMoveList;
    type IntoIter = RootMovesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RootMovesIter {
            root_moves: &self,
            idx: 0,
        }
    }
}
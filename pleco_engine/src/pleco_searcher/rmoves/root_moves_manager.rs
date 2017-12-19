use super::{MAX_THREADS,MAX_MOVES,RootMove};
use super::root_moves_list::{RootMoveList,RawRootMoveList};

use std::heap::{Alloc, Layout, Heap};
use std::cell::UnsafeCell;
use std::ptr::{Unique,Shared};
use std::sync::Arc;
use std::sync::atomic::{Ordering,AtomicUsize};
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
            let raw = Unique::new(new_ptr as *mut RawRmManager).unwrap();
            for x in 0..MAX_THREADS {
                let raw_list: &mut RawRootMoveList = (*raw.as_ptr()).rms.get_unchecked_mut(x);
                raw_list.init();
            }
            raw
        }
    }
}


pub struct RmManager {
    threads: Arc<AtomicUsize>,
    moves: Unique<RawRmManager>,
    ref_count: Arc<u8>
}

impl Clone for RmManager {
    fn clone(&self) -> Self {
        RmManager {
            threads: self.threads.clone(),
            moves: self.moves.clone(),
            ref_count: self.ref_count.clone()
        }
    }
}

impl RmManager {
    pub fn new() -> Self {
        RmManager {
            threads: Arc::new(AtomicUsize::new(0)),
            moves: RawRmManager::new(),
            ref_count: Arc::new(0)
        }
    }

    pub fn threads(&self) -> usize {
        self.threads.load(Ordering::Relaxed)
    }

    pub fn add_thread(&mut self) -> Option<RootMoveList> {
        if self.threads() >= MAX_THREADS {
            None
        } else {
            let thread_idx = self.threads.fetch_add(1, Ordering::Relaxed);
            unsafe {
                Some(self.get_list_unchecked(thread_idx))
            }
        }
    }

    pub fn get_list(&self, num: usize) -> Option<RootMoveList> {
        if num >= self.threads() {
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
        let mut first = self.as_ptr();
        first.replace(&legal_moves);
        let num = self.threads();
        for i in 1..num {
            self.get_list_unchecked(i).clone_from(&first);
        }
    }

    pub unsafe fn as_ptr(&self) -> RootMoveList {
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
    threads: usize,
    idx: usize,
}

impl<'a> Iterator for RootMovesIter<'a> {
    type Item = RootMoveList;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.threads {
            None
        } else {
            unsafe {
                Some(self.root_moves.get_list_unchecked(self.idx))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.threads - self.idx, Some(self.threads - self.idx))
    }
}

impl<'a> IntoIterator for &'a RmManager {
    type Item = RootMoveList;
    type IntoIter = RootMovesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RootMovesIter {
            root_moves: &self,
            threads: self.threads(),
            idx: 0,
        }
    }
}

impl Drop for RmManager {
    fn drop(&mut self) {
        if Arc::strong_count(&self.ref_count) != 1 {
            return
        }
        unsafe {
            Heap.dealloc(self.as_raw_ptr() as *mut _,
                         Layout::array::<RawRootMoveList>(MAX_THREADS).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn rm_basic() {
        let mut rms = RmManager::new();
        assert_eq!(rms.threads(), 0);
        let moves_1 = rms.add_thread().unwrap();
        assert_eq!(rms.threads(), 1);
        assert_eq!(moves_1.len(), 0);
        let board = Board::default();
        unsafe {
            rms.replace_moves(&board);
            let moves_1_clone = rms.get_list_unchecked(0);
            assert_eq!(moves_1.len(), moves_1_clone.len());
            let moves_2 = rms.add_thread().unwrap();
            assert_eq!(rms.threads(), 2);
            rms.replace_moves(&board);
            let moves_2_clone = rms.get_list_unchecked(0);
            assert_eq!(moves_2.len(), moves_2_clone.len());
            assert_eq!(moves_1.len(), moves_2_clone.len());
        }

    }
}
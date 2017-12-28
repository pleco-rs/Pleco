use super::RootMove;
use super::super::MAX_THREADS;
use super::root_moves_list::{RootMoveList,RawRootMoveList};


use std::heap::{Alloc, Layout, Heap};
use std::ptr::Unique;
use std::sync::Arc;
use std::sync::atomic::{Ordering,AtomicUsize,fence,compiler_fence};
use std::ops::{Deref, DerefMut,Index,IndexMut};

use std::iter::{Iterator,IntoIterator};

use pleco::Board;
use pleco::board::movegen::{MoveGen,Legal};
use pleco::core::mono_traits::AllGenType;

pub struct RmManager {
    threads: Arc<AtomicUsize>,
    moves: Unique<RawRootMoveList>,
    ref_count: Arc<AtomicUsize>
}

unsafe impl Send for RmManager {}

impl Clone for RmManager {
    fn clone(&self) -> Self {
        let ref_count: Arc<AtomicUsize> = self.ref_count.clone();
        ref_count.fetch_add(1, Ordering::SeqCst);
        RmManager {
            threads: self.threads.clone(),
            moves: self.moves.clone(),
            ref_count
        }
    }
}

impl RmManager {
    pub fn new() -> Self {
        let mut rms = RmManager::init();
        rms.allocate();
        rms
    }

    fn init() -> Self {
        RmManager {
            threads: Arc::new(AtomicUsize::new(0)),
            moves: Unique::empty(),
            ref_count: Arc::new(AtomicUsize::new(1))
        }
    }

    fn allocate(&mut self) {
        unsafe {
            let layout = Layout::array::<RawRootMoveList>(MAX_THREADS).unwrap();
            let result = Heap.alloc(layout);
            let new_ptr = match result {
                Ok(ptr) => ptr,
                Err(err) => Heap.oom(err),
            };
            self.moves = Unique::new(new_ptr as *mut RawRootMoveList).unwrap();
        }
    }

    fn ptr(&self) -> *mut RawRootMoveList {
        self.moves.as_ptr()
    }

    pub fn size(&self) -> usize {
        self.threads.load(Ordering::Relaxed)
    }

    pub fn add_thread(&mut self) -> Option<RootMoveList> {
        if self.size() >= MAX_THREADS {
            None
        } else {
            let thread_idx = self.threads.fetch_add(1, Ordering::SeqCst);
            unsafe {
                let mut list = self.get_list_unchecked(thread_idx);
                list.init();
                Some(list)
            }
        }
    }

    pub fn remove_thread(&mut self) {
        if self.size() > 0 {
            self.threads.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub fn get_list(&self, num: usize) -> Option<RootMoveList> {
        if num >= self.size() {
            None
        } else {
            unsafe {
                Some(self.get_list_unchecked(num))
            }
        }
    }

    pub unsafe fn get_list_unchecked(&self, num: usize) -> RootMoveList {
        RootMoveList {
            moves: self.ptr().offset(num as isize)
        }
    }

    pub unsafe fn replace_moves(&mut self, board: &Board) {
        let legal_moves = MoveGen::generate::<Legal, AllGenType>(&board);
        let mut first = self.get_list_unchecked(0);
        first.replace(&legal_moves);
        let num = self.size();
        for i in 1..num {
            self.get_list_unchecked(i).clone_from_other(&first);
        }
    }

    pub fn wait_for_finish(&self) {
        unsafe {
            for i in 0..self.size() {
                fence(Ordering::AcqRel);
                compiler_fence(Ordering::AcqRel);
                let root_moves = self.get_list_unchecked(i).moves;
                (*root_moves).finished.await(true);
            }
        }
    }
    pub fn wait_for_start(&self) {
        unsafe {
            let num_threads = self.size();
            for i in 0..num_threads {
                fence(Ordering::AcqRel);
                compiler_fence(Ordering::AcqRel);
                let root_moves = self.get_list_unchecked(i).moves;
                (*root_moves).finished.await(false);
            }
        }
    }

    pub fn reset_depths(&self) {
        unsafe {
            for i in 0..self.size() {
                self.get_list_unchecked(i).set_depth_completed(0);
            }
        }
    }

    pub fn thread_best_move_and_depth(&self, thread_id: usize) -> (RootMove, u16) {
        unsafe {
            let mut thread = self.get_list_unchecked(thread_id);
            (thread.first().clone(), thread.depth_completed())
        }

    }


    pub fn best_rootmove(&self, use_stdout: bool) -> RootMove {
        let (mut best_root_move, mut depth_reached): (RootMove, u16) = self.thread_best_move_and_depth(0);
        if use_stdout {
            println!("id: 0, value: {}, prev_value: {}, depth: {}, depth_comp: {}, mov: {}", best_root_move.score, best_root_move.prev_score, best_root_move.depth_reached,depth_reached, best_root_move.bit_move);
        }

        for x in 1..self.size() {
            let (thread_move, thread_depth): (RootMove, u16)  = self.thread_best_move_and_depth(x);
            let depth_diff = thread_depth as i32 - depth_reached as i32;
            let value_diff = thread_move.score - best_root_move.score;
            if x != 0 {
                // If it has a bigger value and greater or equal depth
                if value_diff > 0 && depth_diff >= 0 {
                    best_root_move = thread_move;
                    depth_reached = thread_depth;
                }
            }

            if use_stdout {
                println!("id: {}, value: {}, prev_value: {}, depth: {}, depth_comp: {}, mov: {}",x, thread_move.score, thread_move.prev_score, thread_move.depth_reached,thread_depth, thread_move.bit_move);
            }
        }
        best_root_move
    }
}

impl Deref for RmManager {
    type Target = [RawRootMoveList];
    fn deref(&self) -> &[RawRootMoveList] {
        unsafe {
            ::std::slice::from_raw_parts(self.ptr(), MAX_THREADS)
        }
    }
}

impl DerefMut for RmManager {
    fn deref_mut(&mut self) -> &mut [RawRootMoveList] {
        unsafe {
            ::std::slice::from_raw_parts_mut(self.ptr(), MAX_THREADS)
        }
    }
}

impl Index<usize> for RmManager {
    type Output = RawRootMoveList;

    #[inline]
    fn index(&self, index: usize) -> &RawRootMoveList {
        &(**self)[index]
    }
}

impl IndexMut<usize> for RmManager {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut RawRootMoveList {
        &mut (**self)[index]
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
            threads: self.size(),
            idx: 0,
        }
    }
}

impl Drop for RmManager {
    fn drop(&mut self) {
        let num = self.ref_count.fetch_sub(1, Ordering::SeqCst);
        if num == 1 {
            unsafe {
                Heap.dealloc(self.ptr() as *mut _,
                             Layout::array::<RawRootMoveList>(MAX_THREADS).unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn rm_basic() {
        let mut rms = RmManager::new();
        assert_eq!(rms.size(), 0);
        let moves_1 = rms.add_thread().unwrap();
        assert_eq!(rms.size(), 1);
        assert_eq!(moves_1.len(), 0);
        let board = Board::default();
        unsafe {
            rms.replace_moves(&board);
            let moves_1_clone = rms.get_list_unchecked(0);
            assert_eq!(moves_1.len(), moves_1_clone.len());
            let moves_2 = rms.add_thread().unwrap();
            assert_eq!(rms.size(), 2);
            rms.replace_moves(&board);
            let moves_2_clone = rms.get_list_unchecked(0);
            assert_eq!(moves_2.len(), moves_2_clone.len());
            assert_eq!(moves_1.len(), moves_2_clone.len());
        }
    }

    #[test]
    fn rm_cloning() {
        let mut rms = RmManager::new();
        let rmsc = rms.clone();
        rms.add_thread();
        rms.add_thread();
        assert_eq!(rms.size(), rmsc.size());
    }
}
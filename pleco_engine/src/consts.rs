//! Constant values and static structures.
use std::heap::{Alloc, Layout, Global};

use std::ptr::{NonNull, self};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{ONCE_INIT,Once};
use std::sync::atomic::compiler_fence;

use pleco::tools::tt::TranspositionTable;
use pleco::helper::prelude;

use time::time_management::TimeManager;
use threadpool;
use search;
use tables::pawn_table;

pub const MAX_PLY: u16 = 126;
pub const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

pub const DEFAULT_TT_SIZE: usize = 256;

pub const PAWN_TABLE_SIZE: usize = 16384;
pub const MATERIAL_TABLE_SIZE: usize = 8192;

static INITALIZED: Once = ONCE_INIT;

pub static USE_STDOUT: AtomicBool = AtomicBool::new(true);
/// Global Timer
//pub static TIMER: TimeManager = TimeManager::uninitialized();

static mut TT_TABLE: NonNull<TranspositionTable> = unsafe
    {NonNull::new_unchecked(ptr::null_mut())};

static mut TIMER: NonNull<TimeManager> = unsafe
    {NonNull::new_unchecked(ptr::null_mut())};

pub fn init_globals() {
    INITALIZED.call_once(|| {
        prelude::init_statics();   // Initialize static tables
        compiler_fence(Ordering::SeqCst);
        init_tt();                 // Transposition Table
        init_timer();              // Global timer manager
        pawn_table::init();
        threadpool::init_threadpool();  // Make Threadpool
        search::init();
    });
}

// initalizes the transposition table
fn init_tt() {
    unsafe {
        let layout = Layout::new::<TranspositionTable>();
        let result = Global.alloc_zeroed(layout);
        let new_ptr: *mut TranspositionTable = match result {
            Ok(ptr) => ptr.cast().as_ptr() as *mut TranspositionTable,
            Err(_err) => Global.oom(),
        };
        ptr::write(new_ptr, TranspositionTable::new(DEFAULT_TT_SIZE));
        TT_TABLE = NonNull::new_unchecked(new_ptr);
    }
}

fn init_timer() {
    unsafe {
        let layout = Layout::new::<TimeManager>();
        let result = Global.alloc_zeroed(layout);
        let new_ptr: *mut TimeManager = match result {
            Ok(ptr) => ptr.cast().as_ptr() as *mut TimeManager,
            Err(_err) => Global.oom(),
        };
        ptr::write(new_ptr, TimeManager::uninitialized());
        TIMER = NonNull::new_unchecked(new_ptr);
    }
}

pub fn timer() -> &'static TimeManager {
    unsafe {
        &*TIMER.as_ptr()
    }
}

/// Returns access to the global transposition table
pub fn tt() -> &'static TranspositionTable {
    unsafe {
        &*TT_TABLE.as_ptr()
    }
}


pub trait PVNode {
    fn is_pv() -> bool;
}

pub struct PV {}
pub struct NonPV {}

impl PVNode for PV {
    fn is_pv() -> bool {
        true
    }
}

impl PVNode for NonPV {
    fn is_pv() -> bool {
        false
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_da() {
        threadpool::init_threadpool();

    }
}
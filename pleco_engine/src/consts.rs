//! Constant values and static structures.
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::compiler_fence;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Once;

use pleco::helper::prelude;
use pleco::tools::tt::TranspositionTable;

use search;
use tables::pawn_table;
use threadpool;
use time::time_management::TimeManager;

pub const MAX_PLY: u16 = 126;
pub const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

pub const DEFAULT_TT_SIZE: usize = 256;
pub const PAWN_TABLE_SIZE: usize = 16384;
pub const MATERIAL_TABLE_SIZE: usize = 8192;

pub static USE_STDOUT: AtomicBool = AtomicBool::new(true);

static INITIALIZED: Once = Once::new();

/// Global Transposition Table
static mut TT_TABLE: MaybeUninit<TranspositionTable> = MaybeUninit::uninit();

// Global Timer
static mut TIMER: MaybeUninit<TimeManager> = MaybeUninit::uninit();

#[cold]
pub fn init_globals() {
    INITIALIZED.call_once(|| {
        prelude::init_statics(); // Initialize static tables
        compiler_fence(Ordering::SeqCst);
        init_tt(); // Transposition Table
        init_timer(); // Global timer manager
        pawn_table::init();
        threadpool::init_threadpool(); // Make Threadpool
        search::init();
    });
}

// Initializes the transposition table
#[cold]
fn init_tt() {
    unsafe {
        ptr::write(
            TT_TABLE.as_mut_ptr(),
            TranspositionTable::new(DEFAULT_TT_SIZE),
        );
    }
}

// Initializes the global Timer
#[cold]
fn init_timer() {
    unsafe {
        ptr::write(TIMER.as_mut_ptr(), TimeManager::uninitialized());
    }
}

// Returns access to the global timer
pub fn timer() -> &'static TimeManager {
    unsafe { &*TIMER.as_ptr() }
}

/// Returns access to the global transposition table
#[inline(always)]
pub fn tt() -> &'static TranspositionTable {
    unsafe { &*TT_TABLE.as_ptr() }
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
    fn initializing_threadpool() {
        threadpool::init_threadpool();
    }
}

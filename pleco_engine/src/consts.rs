//! Constant values and static structures.
use lazy_static;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use pleco::tools::tt::TranspositionTable;
//use time::time_management::TimeManager;

pub const MAX_PLY: u16 = 126;
pub const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

pub const DEFAULT_TT_SIZE: usize = 256;

pub const PAWN_TABLE_SIZE: usize = 16384;
pub const MATERIAL_TABLE_SIZE: usize = 8192;

static INITALIZED: AtomicBool = AtomicBool::new(false);
/// Global Timer
//pub static TIMER: TimeManager = TimeManager::uninitialized();
//pub static TT_TABLE: TranspositionTable = unsafe {TranspositionTable::uninitialized()};
//pub static mut POSITION: Board = unsafe {Board::uninitialized()};

lazy_static! {
    pub static ref TT_TABLE: TranspositionTable = TranspositionTable::new(DEFAULT_TT_SIZE);
}

pub fn init_globals() {
    if !INITALIZED.swap(true, Ordering::SeqCst) {
//        unsafe {
            lazy_static::initialize(&TT_TABLE);
//            POSITION.uninitialized_init();
//        }
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

pub trait CheckState {
    fn in_check() -> bool;
}


pub struct InCheck {}
pub struct NoCheck {}

impl CheckState for InCheck {
    fn in_check() -> bool { true}
}

impl CheckState for NoCheck {
    fn in_check() -> bool { false}
}

//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn test_da() {
//        init_globals();
//
//    }
//}
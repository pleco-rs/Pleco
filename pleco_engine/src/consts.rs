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
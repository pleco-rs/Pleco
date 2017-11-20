pub mod misc;
pub mod options;
pub mod threads;
pub mod thread_search;

use engine::UCILimit;
use tools::tt::TT;
use Board;
use BitMove;
use std::thread;

use self::options::{UciOption,AllOptions};
use self::threads::ThreadPool;

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(256);
}

static mut LIMIT: UCILimit = UCILimit::Infinite;



pub struct _PlecoSearcher {
    options: AllOptions,
    thread_pool: ThreadPool
}



impl _PlecoSearcher {

    pub fn init(use_stdout: bool) -> Self {
        _PlecoSearcher {
            options: AllOptions::default(),
            thread_pool: ThreadPool::setup(8,use_stdout)
        }
    }

    pub fn search(&mut self, board: &Board, limit: &UCILimit) {
        self.thread_pool.uci_search(&board, &limit);
    }

    pub fn stop_search(&mut self) -> BitMove {
        self.thread_pool.stop_searching();
        self.thread_pool.get_move()
    }

    pub fn clear_tt(&mut self) {
        unsafe {TT_TABLE.clear() };
    }

    pub fn apply_option(&mut self, name: &str) {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {

    use super::*;

//    #[test]
    pub fn testme() {
        {
            let mut s = _PlecoSearcher::init(false);
            let limit = UCILimit::Infinite;
            let board = Board::default();
            s.search(&board, &limit);
            thread::sleep_ms(5000);
            s.stop_search();
        }
    }

}


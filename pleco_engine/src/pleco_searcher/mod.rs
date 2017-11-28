pub mod misc;
pub mod options;
pub mod threads;
pub mod thread_search;

use pleco::engine::UCILimit;
use pleco::tools::tt::TT;
use pleco::Board;
use pleco::BitMove;

use std::thread;

use self::options::{UciOption,AllOptions};
use self::threads::ThreadPool;

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(256);
}

#[derive(PartialEq)]
enum SearchType {
    None,
    Search,
    Ponder,
}

pub struct PlecoSearcher {
    options: AllOptions,
    thread_pool: ThreadPool,
    search_mode: SearchType,
}


impl PlecoSearcher {

    pub fn init(use_stdout: bool) -> Self {
        PlecoSearcher {
            options: AllOptions::default(),
            thread_pool: ThreadPool::setup(8,use_stdout),
            search_mode: SearchType::None
        }
    }

    pub fn search(&mut self, board: &Board, limit: &UCILimit) {
        TT_TABLE.new_search();
        self.search_mode = SearchType::Search;
        self.thread_pool.uci_search(&board, &limit);
    }

    pub fn stop_search(&mut self) -> BitMove {
        self.thread_pool.stop_searching();
        self.search_mode = SearchType::None;
        self.thread_pool.get_move()
    }

    pub fn is_searching(&self) -> bool {
        if self.search_mode == SearchType::None {
            return false;
        }
        true
    }

    pub fn hash_percent(&self) -> f64 {
        TT_TABLE.hash_percent()
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
            let mut s = PlecoSearcher::init(false);
            let limit = UCILimit::Infinite;
            let board = Board::default();
            s.search(&board, &limit);
            thread::sleep_ms(20000);
            s.stop_search();
            println!("TT Hash {}", 100.0 * TT_TABLE.hash_percent());
        }
    }

}


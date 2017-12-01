pub mod misc;
pub mod options;
pub mod threads;
pub mod thread_search;

use pleco::tools::UCILimit;
use pleco::tools::tt::TT;
use pleco::Board;
use pleco::BitMove;

use std::thread;
use std::io;

use self::options::{UciOption,AllOptions,UciOptionMut};
use self::threads::ThreadPool;


const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(256);
}

pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHORS: &str = "Stephen Fleischman";
pub static VERSION: &str = "0.0.2";

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

    pub fn uci(&mut self) {
        let mut full_command = String::new();
        'main: loop {
            full_command.clear();
            io::stdin().read_line(&mut full_command).ok().unwrap();
            let mut args: Vec<&str> = full_command.split_whitespace().collect();
            let command: &str = args.first().unwrap_or(&"");
            match command {
                "" => continue,
                "uci" => self.uci_startup(),
                "setoption" => self.apply_option(&full_command),
                "options" | "alloptions" => self.options.print_curr(),
                "quit" | "stop" => {
                    self.halt();
                    break;
                },
                _ => println!("Unknown Command: {}",full_command)
            }

        }
    }

    fn apply_option(&mut self, option: &str) {
        let c = self.options.apply_option(option);
        match c {
            UciOptionMut::Button(c)   => {(c)(self);},
            UciOptionMut::Check(c, v) => {(c)(self, v);},
            UciOptionMut::Spin(c, v)  => {(c)(self, v);},
            UciOptionMut::Combo(c, v) => {(c)(self, v);},
            UciOptionMut::Text(c, v)  => {(c)(self, v);},
            UciOptionMut::None => {},
        }

    }

    fn uci_startup(&self) {
        println!("id name {}",ID_NAME);
        println!("id authors {}", ID_AUTHORS);
        self.options.print_all();
        println!("uciok");
    }

    pub fn search(&mut self, board: &Board, limit: &UCILimit) {
        TT_TABLE.new_search();
        self.search_mode = SearchType::Search;
        self.thread_pool.uci_search(&board, &limit);
    }

    pub fn halt(&mut self) {
        self.thread_pool.stop_searching();
        self.search_mode = SearchType::None;
    }

    pub fn stop_search_get_move(&mut self) -> BitMove {
        if self.is_searching() {
            self.halt();
            return self.thread_pool.get_move();
        } else {
            return BitMove::null();
        }
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
            s.stop_search_get_move();
            println!("TT Hash {}", 100.0 * TT_TABLE.hash_percent());
        }
    }

}


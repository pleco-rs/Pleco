//! The main searching structure.

pub mod misc;
pub mod threads;
pub mod search;
pub mod root_moves;
pub mod sync;
pub mod parse;
pub mod uci_options;

use pleco::tools::tt::TranspositionTable;
use pleco::Board;
use pleco::BitMove;

use std::io;

use self::misc::{PreLimits};
use self::threads::ThreadPool;
use self::uci_options::OptionsMap;

use num_cpus;

// --------- STATIC VARIABLES

pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHORS: &str = "Stephen Fleischman";
pub static VERSION: &str = "0.0.3";

// -------- CONSTANTS

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;
pub const DEFAULT_TT_SIZE: usize = 256;


// MUTATABLE STATIC VARIABLES;

lazy_static! {
    pub static ref TT_TABLE: TranspositionTable = TranspositionTable::new(DEFAULT_TT_SIZE);
}


#[derive(PartialEq)]
enum SearchType {
    None,
    Search,
    Ponder,
}

pub struct PlecoSearcher {
    options: OptionsMap,
    thread_pool: ThreadPool,
    search_mode: SearchType,
    board: Option<Board>,
    limit: Option<PreLimits>,
}



impl PlecoSearcher {
    pub fn init(use_stdout: bool) -> Self {
        unsafe {
            TT_TABLE.clear();
        }
        let mut pool = ThreadPool::new();
        pool.stdout(use_stdout);
        pool.set_thread_count(num_cpus::get());
        PlecoSearcher {
            options: OptionsMap::new(),
            thread_pool: pool,
            search_mode: SearchType::None,
            board: None,
            limit: None
        }
    }

    pub fn uci(&mut self) {
        let mut full_command = String::new();
        'main: loop {
            full_command.clear();
            io::stdin().read_line(&mut full_command).ok().unwrap();
            let args: Vec<&str> = full_command.split_whitespace().collect();
            let command: &str = args.first().unwrap_or(&"");
            match command {
                "" => continue,
                "uci" => self.uci_startup(),
                "setoption" => self.apply_option(&full_command),
                "options" | "alloptions" => {},
                "ucinewgame" => self.clear_search(),
                "isready" => println!("readyok"),
                "position" => self.board = parse::parse_board(&args[1..]),
                "go" => self.uci_go(&args[1..]),
                "quit" => {
                    self.halt();
                    break;
                },
                "stop" => self.halt(),
                _ => println!("Unknown Command: {}",full_command)
            }

        }
    }

    pub fn clear_search(&mut self) {
        self.clear_tt();
        self.board = None;
    }

    fn uci_go(&mut self, args: &[&str]) {
        let limit = parse::parse_time(&args[1..]);

        let poss_board = self.board.as_ref()
            .map(|b| b.shallow_clone());
        if let Some(board) = poss_board {
            self.search(&board, &limit);
        }
    }

    fn apply_option(&mut self, full_command: &str) {
        let mut args: Vec<&str> = full_command.split_whitespace().collect();
        if args.len() < 3 || args[1] != "name" {
            println!("unknown option: {}", full_command);
        }
        args.remove(0);
        args.remove(0);

//        let name: &str = args[0];

//        let c = self.options.apply_option(option);
//        match c {
//            UciOptionMut::Button(c)   => {(c)(self);},
//            UciOptionMut::Check(c, v) => {(c)(self, v);},
//            UciOptionMut::Spin(c, v)  => {(c)(self, v);},
//            UciOptionMut::Combo(c, v) => {(c)(self, v);},
//            UciOptionMut::Text(c, v)  => {(c)(self, v);},
//            UciOptionMut::None => {},
//        }
    }

    fn uci_startup(&self) {
        println!("id name {}",ID_NAME);
        println!("id authors {}", ID_AUTHORS);
        self.options.display_all();
        println!("uciok");
    }

    pub fn search(&mut self, board: &Board, limit: &PreLimits) {
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

    pub fn await_move(&mut self) -> BitMove {
        if self.is_searching() {
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

    pub fn resize_tt(&mut self, mb: usize) {
        unsafe {TT_TABLE.resize_to_megabytes(mb)};
    }

    pub fn use_stdout(&mut self, stdout: bool) {
        self.thread_pool.stdout(stdout);
    }


}

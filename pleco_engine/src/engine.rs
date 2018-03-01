//! The main searching structure.

use std::io;
use std::sync::atomic::Ordering;

use pleco::Board;
use pleco::BitMove;

use time::uci_timer::{PreLimits};
use uci::options::{OptionsMap,OptionWork};
use uci::parse;
use TT_TABLE;
use consts::*;
use threadpool::threadpool;

use num_cpus;

// --------- STATIC VARIABLES

pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHORS: &str = "Stephen Fleischman";
pub static VERSION: &str = "0.0.8";

#[derive(PartialEq)]
enum SearchType {
    None,
    Search,
    Ponder,
}

pub struct PlecoSearcher {
    options: OptionsMap,
    search_mode: SearchType,
    board: Board
}

impl PlecoSearcher {
    pub fn init(use_stdout: bool) -> Self {
        init_globals();
        USE_STDOUT.store(use_stdout,Ordering::Relaxed);
        threadpool().set_thread_count(num_cpus::get());
        PlecoSearcher {
            options: OptionsMap::new(),
            search_mode: SearchType::None,
            board: Board::default()
        }
    }

    pub fn uci(&mut self) {
        self.uci_startup();
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
                "position" => {
                    if let Some(b) = parse::position_parse_board(&args[1..]) {
                        self.board = b;
                    } else {
                        println!("unable to parse board");
                    }
                },
                "setboard" => {
                    if let Some(b) = parse::setboard_parse_board(&args[1..]) {
                        self.board = b;
                    } else {
                        println!("unable to parse board");
                    }
                }
                "go" => self.uci_go(&args[1..]),
                "quit" => {
                    self.halt();
                    break;
                },
                "stop" => self.halt(),
                _ => print!("Unknown Command: {}",full_command)
            }
            self.apply_all_options();

        }
    }

    pub fn clear_search(&mut self) {
        self.clear_tt();
        threadpool().clear_all();
    }

    fn uci_go(&mut self, args: &[&str]) {
        let limit = parse::parse_time(&args);
        threadpool().uci_search(&self.board, &limit.create())
    }

    fn apply_option(&mut self, full_command: &str) {
        let mut args  = full_command.split_whitespace();
        args.next().unwrap();  // setoption
        if let Some(non_name) = args.next() {
            if non_name != "name" {
                println!("setoption `name`");
                return;
            }
        } else {
            println!("setoption `name`");
            return;
        }
        let mut name = String::new();
        let mut value = String::new();

        if let Some(third_arg) = args.next() { //[should be name of the option]
            name += third_arg;
        } else {
            println!("setoption needs a name!");
            return;
        }

        'nv: while let Some(ref partial_name) = args.next(){
            if *partial_name == "value" {
                value = args.map(|s| s.to_string() + " ")
                            .collect::<String>()
                            .trim()
                            .to_string();
                if &value == "" {
                    println!("forgot a value!");
                    return;
                }
                break 'nv;
            } else {
                name += " ";
                name += partial_name;
            }
        }

        println!("name :{}: value :{}:",name,value);

        if !self.options.apply_option(&name, &value) {
            println!("unable to apply option: {}",full_command);
        } else {
            self.apply_all_options();
        }
    }

    fn apply_all_options(&mut self) {
        while let Some(work) = self.options.work() {
            if self.is_searching() && !work.usable_while_searching() {
                println!("unable to apply work");
            } else {
                match work {
                    OptionWork::ClearTT => {self.clear_tt()},
                    OptionWork::ResizeTT(mb) => {self.resize_tt(mb)},
                    OptionWork::Threads(num) => {threadpool().set_thread_count(num)}
                }
            }
        }
    }

    fn uci_startup(&self) {
        println!("id name {}",ID_NAME);
        println!("id authors {}", ID_AUTHORS);
        self.options.display_all();
        println!("uciok");
    }

    pub fn search(&mut self, board: &Board, limit: &PreLimits) {
        self.search_mode = SearchType::Search;
        threadpool().uci_search(board, &(limit.clone().create()));

    }

    pub fn halt(&mut self) {
        self.search_mode = SearchType::None;
        threadpool().set_stop(true);
    }

    pub fn stop_search_get_move(&mut self) -> BitMove {
        self.search_mode = SearchType::None;
        if self.is_searching() {
            threadpool().set_stop(true);
            threadpool().wait_for_finish();
            threadpool().best_move()
        } else {
            return BitMove::null();
        }
    }

    pub fn await_move(&mut self) -> BitMove {
        if self.is_searching() {
            return {
                threadpool().wait_for_finish();
                threadpool().best_move()
            }
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
        threadpool().stdout(stdout);
    }


}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ply_3() {
        let mut limit = PreLimits::blank();
        limit.depth = Some(3);
        let board = Board::default();
        let mut s = PlecoSearcher::init(false);
        s.search(&board, &limit);
        s.await_move();
    }


}
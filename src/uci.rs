use engine::{UCILimit,UCISearcher};
use board::Board;
use bots::lazy_smp::LazySMPSearcher;
use timer::Timer;
use piece_move::BitMove;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::env;
use std::process;
use std::io::prelude::*;
use std::thread;

use std::io::{self, Read, Error};

/// commands
///
/// uci
///     -> Switches to UCI Mode
///
///
///
///


pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHORS: &str = "Stephen Fleischman";
pub static VERSION: &str = "0.1.1";


pub fn console_loop(mut args: Vec<String>) {
    print_startup();


    'main: loop {
        while args.is_empty() {
            let mut input = String::new();
            let result = io::stdin().read_line(&mut input);
            args = input.split_whitespace().map(|str| str.to_owned()).collect();
        }
        let args_clone = args.clone();
        let command: &str = args_clone.first().unwrap();

        if command == "uci" {
            uci();
            break 'main;
        } else if command == "q" || command == "quit" {
            break 'main
        } else {
            println!("command not recognized: {}",command);
        }
        args.clear();
    }
}

fn uci() {
    println!("id name {}",ID_NAME);
    println!("id author {}",ID_AUTHORS);
    println!("uciok");

    let mut is_debug: bool = false;
    let mut board = Board::default();

    let stop_searching = Arc::new(AtomicBool::new(false));

    let mut args: Vec<String> = Vec::new();
    'uci: loop {
        while args.is_empty() {
            let mut input = String::new();
            let result = io::stdin().read_line(&mut input);
            args = input.split_whitespace().map(|str| str.to_owned()).collect();
        }
        let args_clone = args.clone();
        let command: &str =   &args_clone.get(0).unwrap().clone();

        if command == "isready" {
            println!("readyok");
        } else if command == "quit" {
            break 'uci;
        } else if command == "ucinewgame" {
        } else if command == "position" {
            board = parse_board_position(args_clone);
        } else if command == "go" {
            mid_search_loop(&mut board, parse_limit(args_clone), stop_searching.clone());
        } else if command == "stop" {
            stop_searching.store(true, Ordering::Relaxed);
        } else {
            println!("command not recognized")
        }
        args.clear()
    }
}

fn mid_search_loop(board: &mut Board, limit: UCILimit, stop: Arc<AtomicBool>) {
    stop.store(false, Ordering::Relaxed);
    let mut searcher = LazySMPSearcher::setup(board.shallow_clone(), stop.clone());
    let child = thread::spawn(move || {
        searcher.uci_go(limit, true)
    });
}


fn parse_board_position(tokens: Vec<String>) -> Board {
    let mut token_stack = tokens.clone();
    token_stack.reverse();
    token_stack.pop();

    let start_str = token_stack.pop().unwrap();
    let start = &start_str;
    let mut board = if start == "startpos" {
        Board::default()
    } else if start == "fen" {
        let fen_string: &str = &token_stack.pop().unwrap();
        Board::new_from_fen(fen_string)
    } else {
        panic!()
    };

    if !token_stack.is_empty() {
        let next = &token_stack.pop().unwrap();
        if next == "moves" {
            while !token_stack.is_empty() {
                let bit_move = &token_stack.pop().unwrap();
                let mut all_moves: Vec<BitMove> = board.generate_moves();
                'check_legality: loop {
                    if all_moves.is_empty() {
                        panic!();
                    }
                    let curr_move: BitMove = all_moves.pop().unwrap();
                    if &curr_move.stringify() == bit_move {
                        board.apply_move(curr_move);
                        break 'check_legality
                    }
                }
            }
        }
    }
    board
}

fn parse_limit(tokens: Vec<String>) -> UCILimit {
    let mut token_stack = tokens.clone();
    token_stack.reverse();

    let mut white_time: i64 = i64::max_value();
    let mut black_time: i64 = i64::max_value();
    let mut white_inc: i64 = i64::max_value();
    let mut black_inc: i64 = i64::max_value();

    while !token_stack.is_empty() {
        let token = token_stack.pop().unwrap();
        if token == "inf" {
            return UCILimit::Infinite;
        } else if token == "wtime" {
            white_time = unwrap_val_or(&mut token_stack, i64::max_value());
        } else if token == "btime" {
            black_time = unwrap_val_or(&mut token_stack, i64::max_value());
        } else if token == "winc" {
            white_inc = unwrap_val_or(&mut token_stack, 0);
        } else if token == "binc" {
            black_inc = unwrap_val_or(&mut token_stack, 0);
        } else if token == "depth" {
            return UCILimit::Depth(token_stack.pop().unwrap().parse::<u16>().unwrap());
        } else if token == "mate" {
            unimplemented!()
        } else if token == "nodes" {
            unimplemented!()
        } else if token == "movestogo" {
            unimplemented!()
        } else if token == "movetime" {
            unimplemented!()
        }
    }
    UCILimit::Time(
        Timer::new(white_time, black_time, white_inc, black_inc)
    )
}

fn unwrap_val_or(tokens: &mut Vec<String>, or: i64) -> i64 {
    let val = tokens.pop();
    if val.is_some() {
        val.unwrap().parse::<i64>().unwrap_or(or)
    } else {
        or
    }

}


fn print_startup() {
    print!("{} Chess Engine -- ",ID_NAME);
    print!("version {}, ",VERSION);
    println!("by {}",ID_AUTHORS);
}

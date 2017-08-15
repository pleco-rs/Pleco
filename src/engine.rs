extern crate rand;

use piece_move::BitMove;
use timer::Timer;
use board::Board;
use templates::Player;
use std::{thread, time};
use rayon;
use std::io;
use std::error::Error;
use std::sync::{Arc, Mutex};


// Trait that defines an object that can play chess
pub trait Searcher {
    fn best_move(board: Board, timer: &Timer) -> BitMove
    where
        Self: Sized;
    fn best_move_depth(board: Board, timer: &Timer, max_depth: u16) -> BitMove
    where
        Self: Sized;
    fn name() -> &'static str
    where
        Self: Sized;
}

pub trait UCISearcher: Searcher {
    fn uci_move(board: Board, timer: &Timer, rx: Arc<Mutex<Option<GuiToEngine>>>) -> BitMove
    where
        Self: Sized;
}

//  Winner allows representation of the winner of a chess match
pub enum Winner {
    PlayerOne,
    PlayerTwo,
    Draw,
}


pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHOR: &str = "Stephen Fleischman";

pub fn compete<S: Searcher, T: Searcher>(
    player_one: &S,
    player_two: &T,
    minutes_each: i64,
    display: bool,
    randomize: bool,
    ply: u16,
) -> Winner {
    assert!(minutes_each > 0);
    let mut b: Board = Board::default();
    let mut timer = Timer::new(minutes_each);
    if display {
        println!("Match Begin  - \n");
        println!("White: {}", <S as Searcher>::name());
        println!("Black: {}", <T as Searcher>::name());
        b.pretty_print();
    }

    while !b.checkmate() {
        if randomize && b.moves_played() < 2 {
            let moves = b.generate_moves();
            b.apply_move(moves[rand::random::<usize>() % moves.len()]);
            let moves = b.generate_moves();
            b.apply_move(moves[rand::random::<usize>() % moves.len()]);
            let moves = b.generate_moves();
            b.apply_move(moves[rand::random::<usize>() % moves.len()]);
            if rand::random::<usize>() % 5 == 0 {
                let moves = b.generate_moves();
                b.apply_move(moves[rand::random::<usize>() % moves.len()]);
                let moves = b.generate_moves();
                b.apply_move(moves[rand::random::<usize>() % moves.len()]);
            }
        }
        if b.rule_50() >= 50 || b.stalemate() {
            if display {
                if b.rule_50() >= 50 {
                    println!("50 move rule");
                } else {
                    println!("Stalemate");
                }

                println!("Draw")
            }
            return Winner::Draw;
        }

        timer.start_time();
        let ret_move = match b.turn() {
            Player::White => <S as Searcher>::best_move_depth(b.shallow_clone(), &timer, ply),
            Player::Black => <T as Searcher>::best_move_depth(b.shallow_clone(), &timer, ply),
        };
        timer.stop_time();

        if timer.out_of_time() || !b.legal_move(ret_move) {
            return match b.turn() {
                Player::White => Winner::PlayerTwo,
                Player::Black => Winner::PlayerOne,
            };
        }
        timer.switch_turn();

        b.apply_move(ret_move);
        if display {
            println!("Move Chosen: {}\n", ret_move);
            b.pretty_print();
        }
    }

    if display {
        match b.turn() {
            Player::White => {
                println!("White, played by {} wins", <S as Searcher>::name());
            }
            Player::Black => {
                println!("Black, played by {} wins", <T as Searcher>::name());
            }
        };
    }

    match b.turn() {
        Player::White => Winner::PlayerTwo,
        Player::Black => Winner::PlayerOne,
    }
}

pub fn compete_multiple<S: Searcher, T: Searcher>(
    player_one: S,
    player_two: T,
    minutes_each: i64,
    times_match: u32,
    plys: u16,
    display: bool,
) -> Winner {
    let mut p_one_wins: u32 = 0;
    let mut p_two_wins: u32 = 0;
    let mut draws: u32 = 0;

    for i in 0..times_match {
        if display {
            println!{"{}... ", i + 1};
        }
        let result = if i % 2 == 0 {
            compete(&player_one, &player_two, minutes_each, false, true, plys)
        } else {
            compete(&player_two, &player_one, minutes_each, false, true, plys)
        };
        match result {
            Winner::PlayerOne => p_one_wins += 1,
            Winner::PlayerTwo => p_two_wins += 1,
            Winner::Draw => draws += 1,
        };
    }

    if display {
        println!();
        println!(
            "Player One as {} has {} wins",
            <S as Searcher>::name(),
            p_one_wins
        );
        println!(
            "Player Two as {} has {} wins",
            <T as Searcher>::name(),
            p_two_wins
        );
        println!("Draws: {}", draws);
    }

    if p_one_wins > p_two_wins {
        Winner::PlayerOne
    } else if p_two_wins > p_one_wins {
        Winner::PlayerTwo
    } else {
        Winner::Draw
    }
}

#[derive(Copy, Clone)]
pub enum GuiToEngine {
    Stop,
}

pub fn uci<S: UCISearcher>(player_one: S) {
    println!("id name {}", ID_NAME);
    println!("id author {}", ID_AUTHOR);
    let mut timer = Timer::new(3);
    let mut b = Board::default();

    let rw: Mutex<Option<GuiToEngine>> = Mutex::new(None);
    let arc = Arc::new(rw);

    loop {
        let this_arc = arc.clone();
        thread::spawn(move || { poll_stdin(this_arc.clone()); });

        let x = <S as UCISearcher>::uci_move(b.shallow_clone(), &timer, arc.clone());

    }
}


// Regularily polls stdin for any commands sent in from UCI during movement
//
fn poll_stdin(state: Arc<Mutex<Option<GuiToEngine>>>) {

    let mut stdin_input: Option<GuiToEngine> = None;

    while stdin_input.is_none() {
        thread::sleep(time::Duration::from_millis(100));

        let stdin = io::stdin();
        let mut input = &mut String::new();
        let res = stdin.read_line(input);
        return if res.is_ok() {
            stdin_input = parse_uci_interrupt(input);
        } else {
            panic!()
        };
    }

    let mut msg = state.lock().unwrap();
    *msg = stdin_input;
}


fn parse_uci_interrupt(str: &str) -> Option<GuiToEngine> {
    if str.len() <= 1 || str.eq("\n") {
        return None;
    }
    if str.eq("stop\n") {
        return Some(GuiToEngine::Stop);
    }
    None
}


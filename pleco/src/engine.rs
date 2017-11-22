//! This module contains an engine for actually playing chess.

extern crate rand;

use core::piece_move::BitMove;
use tools::timer::Timer;
use board::Board;
use core::Player;

// TODO: clean this up

/// Trait that defines an object that can play chess
pub trait Searcher {
    fn name() -> &'static str where Self: Sized;

    fn best_move(board: Board, limit: UCILimit) -> BitMove
    where
        Self: Sized;

    fn best_move_depth(board: Board, max_depth: u16) -> BitMove
    where
        Self: Sized {
        Self::best_move(board, UCILimit::Depth(max_depth))
    }
}


/// Defines a Limit for a Searcher. e.g., when a searcher should stop
/// searching.
#[derive(Clone)]
pub enum UCILimit {
    Infinite,
    Depth(u16),
    Nodes(u64),
    Time(Timer),
}

impl UCILimit {
    /// Returns if time management should be used.
    pub fn use_time(&self) -> bool {
        if let UCILimit::Time(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns if the limit is depth.
    pub fn is_depth(&self) -> bool {
        if let UCILimit::Depth(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns the depth limit if there is one, otherwise returns 10000.
    pub fn depth_limit(&self) -> u16 {
        if let UCILimit::Depth(depth) = *self {
            depth
        } else {
            10_000
        }
    }

    /// Returns the Timer for the UCILimit, if there is one to be sent.
    pub fn timer(&self) -> Option<Timer> {
        if let UCILimit::Time(timer) = *self {
            Some(timer.clone())
        } else {
            None
        }
    }
}

//  Winner allows representation of the winner of a chess match
pub enum Winner {
    PlayerOne,
    PlayerTwo,
    Draw,
}

/// Pits
pub fn compete<S: Searcher, T: Searcher>(_player_one: &S, _player_two: &T, minutes_each: i64, display: bool, randomize: bool, ply: u16, ) -> Winner {
    assert!(minutes_each > 0);
    let mut b: Board = Board::default();
    let mut timer = Timer::new_no_inc(minutes_each);
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
            Player::White => <S as Searcher>::best_move_depth(b.shallow_clone(), ply),
            Player::Black => <T as Searcher>::best_move_depth(b.shallow_clone(), ply),
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

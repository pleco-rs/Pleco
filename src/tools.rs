//! Miscellaneous tools for debugging and generating output.
use board::Board;
use engine::Searcher;
use rand;
use core::templates::*;
use std::{cmp,char};
use core::piece_move::BitMove;

use bot_prelude::{RandomBot,JamboreeSearcher,IterativeSearcher};


pub const OPENING_POS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub static STANDARD_FENS_START_POS: [&'static str; 1] = [OPENING_POS_FEN];

pub static STANDARD_FENS_MIDDLE_POS: [&'static str; 27] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 11",
    "4rrk1/pp1n3p/3q2pQ/2p1pb2/2PP4/2P3N1/P2B2PP/4RRK1 b - - 7 19",
    "r3r1k1/2p2ppp/p1p1bn2/8/1q2P3/2NPQN2/PPP3PP/R4RK1 b - - 2 15",
    "r1bbk1nr/pp3p1p/2n5/1N4p1/2Np1B2/8/PPP2PPP/2KR1B1R w kq - 0 13",
    "r1bq1rk1/ppp1nppp/4n3/3p3Q/3P4/1BP1B3/PP1N2PP/R4RK1 w - - 1 16",
    "4r1k1/r1q2ppp/ppp2n2/4P3/5Rb1/1N1BQ3/PPP3PP/R5K1 w - - 1 17",
    "2rqkb1r/ppp2p2/2npb1p1/1N1Nn2p/2P1PP2/8/PP2B1PP/R1BQK2R b KQ - 0 11",
    "r1bq1r1k/b1p1npp1/p2p3p/1p6/3PP3/1B2NN2/PP3PPP/R2Q1RK1 w - - 1 16",
    "3r1rk1/p5pp/bpp1pp2/8/q1PP1P2/b3P3/P2NQRPP/1R2B1K1 b - - 6 22",
    "r1q2rk1/2p1bppp/2Pp4/p6b/Q1PNp3/4B3/PP1R1PPP/2K4R w - - 2 18",
    "4k2r/1pb2ppp/1p2p3/1R1p4/3P4/2r1PN2/P4PPP/1R4K1 b - - 3 22",
    "3q2k1/pb3p1p/4pbp1/2r5/PpN2N2/1P2P2P/5PP1/Q2R2K1 b - - 4 26",
    "6k1/6p1/6Pp/ppp5/3pn2P/1P3K2/1PP2P2/3N4 b - - 0 1",
    "3b4/5kp1/1p1p1p1p/pP1PpP1P/P1P1P3/3KN3/8/8 w - - 0 1",
    "8/6pk/1p6/8/PP3p1p/5P2/4KP1q/3Q4 w - - 0 1",
    "7k/3p2pp/4q3/8/4Q3/5Kp1/P6b/8 w - - 0 1",
    "8/2p5/8/2kPKp1p/2p4P/2P5/3P4/8 w - - 0 1",
    "8/1p3pp1/7p/5P1P/2k3P1/8/2K2P2/8 w - - 0 1",
    "8/pp2r1k1/2p1p3/3pP2p/1P1P1P1P/P5KR/8/8 w - - 0 1",
    "8/3p4/p1bk3p/Pp6/1Kp1PpPp/2P2P1P/2P5/5B2 b - - 0 1",
    "5k2/7R/4P2p/5K2/p1r2P1p/8/8/8 b - - 0 1",
    "6k1/6p1/P6p/r1N5/5p2/7P/1b3PP1/4R1K1 w - - 0 1",
    "1r3k2/4q3/2Pp3b/3Bp3/2Q2p2/1p1P2P1/1P2KP2/3N4 w - - 0 1",
    "6k1/4pp1p/3p2p1/P1pPb3/R7/1r2P1PP/3B1P2/6K1 w - - 0 1",
    "8/3p3B/5p2/5P2/p7/PP5b/k7/6K1 w - - 0 1"
];

pub static STANDARD_FENS_5_PIECE_POS: [&'static str; 3] = [
    "8/8/8/8/5kp1/P7/8/1K1N4 w - - 0 1",     // Kc2 - mate
    "8/8/8/5N2/8/p7/8/2NK3k w - - 0 1",      // Na2 - mate
    "8/3k4/8/8/8/4B3/4KB2/2B5 w - - 0 1",    // draw
];

pub static STANDARD_FENS_6_PIECE_POS: [&'static str; 3] = [
    "8/8/1P6/5pr1/8/4R3/7k/2K5 w - - 0 1",   // Re5 - mate
    "8/2p4P/8/kr6/6R1/8/8/1K6 w - - 0 1",    // Ka2 - mate
    "8/8/3P3k/8/1p6/8/1P6/1K3n2 b - - 0 1",  // Nd2 - draw
];

pub static STANDARD_FEN_7_PIECE_POS: [&'static str; 1] = [
    "8/R7/2q5/8/6k1/8/1P5p/K6R w - - 0 124", // Draw
];


pub static STANDARD_FEN_MATE_STALEMATE: [&'static str; 4] = [
    "6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1",
    "r2r1n2/pp2bk2/2p1p2p/3q4/3PN1QP/2P3R1/P4PP1/5RK1 w - - 0 1",
    "8/8/8/8/8/6k1/6p1/6K1 w - - 0 1",
    "7k/7P/6K1/8/3B4/8/8/8 b - - 0 1",
];


lazy_static! {
    pub static ref ALL_FENS: Vec<&'static str> = {
        let mut vec = Vec::new();
        for fen in STANDARD_FENS_START_POS.iter() {vec.push(*fen); }
        for fen in STANDARD_FENS_MIDDLE_POS.iter() {vec.push(*fen); }
        for fen in STANDARD_FENS_5_PIECE_POS.iter() {vec.push(*fen); }
        for fen in STANDARD_FENS_6_PIECE_POS.iter() {vec.push(*fen); }
        for fen in STANDARD_FEN_7_PIECE_POS.iter() {vec.push(*fen); }
        for fen in STANDARD_FEN_MATE_STALEMATE.iter() {vec.push(*fen); }
        vec
    };
}

//#[test]
//fn print_fens() {
//    for fen in ALL_FENS.iter() {
//        print!("{} ",fen);
//        let board = Board::new_from_fen(fen);
//        if board.in_check() {
//            print!(" In check");
//        }
//        println!();
//    }
//}


fn gen_random_fens() {
    let mut b = Board::default();
    println!("[");
    println!("\"{}\",", b.get_fen());

    let quota = 4;

    let max = 200;
    let mut i = 0;

    let beginning_count = 0;
    let mut middle_count = 0;
    let mut end_count = 0;

    while beginning_count + middle_count + end_count <= (quota * 3) - 1 {
        if i == 0 {
            let mov = RandomBot::best_move_depth(b.shallow_clone(),  1);
            b.apply_move(mov);
            let mov = RandomBot::best_move_depth(b.shallow_clone(),  1);
            b.apply_move(mov);
        }
        if b.checkmate() || i > max {
            if beginning_count + middle_count + end_count > quota * 3 {
                i = max;
            } else {
                i = 0;
                b = Board::default();
            }
        } else {
            if i % 11 == 9 {
                let mov = RandomBot::best_move_depth(b.shallow_clone(),  1);
                b.apply_move(mov);
            } else if i % 2 == 0 {
                let mov =
                    JamboreeSearcher::best_move_depth(b.shallow_clone(),  5);
                b.apply_move(mov);
            } else {
                let mov = IterativeSearcher::best_move_depth(b.shallow_clone(),  5);
                b.apply_move(mov);
            }
            i += 1;
        }

        if b.zobrist() % 23 == 11 && b.moves_played() > 7 {
            if b.count_all_pieces() < 13 && end_count < quota {
                println!("\"{}\",", b.get_fen());
                end_count += 1;
            } else if b.count_all_pieces() < 24 && middle_count < quota {
                println!("\"{}\",", b.get_fen());
                middle_count += 1;
            } else if beginning_count < quota {
                println!("\"{}\",", b.get_fen());
                middle_count += 1;
            }
        }
    }

    println!("]");
}
// "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
// https://chess.stackexchange.com/questions/1482/how-to-know-when-a-fen-position-is-legal
// TODO: Finish
pub fn is_valid_fen(fen: &str) -> bool {
    // split the string by white space
    let det_split: Vec<&str> = fen.split_whitespace().collect();

    // must have 6 parts :
    // [ Piece Placement, Side to Move, Castling Ability, En Passant square, Half moves, full moves]
    if det_split.len() != 6 { return false; }

    // Split the first part by '/' for locations
    let b_rep: Vec<&str> = det_split[0].split('/').collect();

    // 8 ranks, so 8 parts
    if b_rep.len() != 8 { return false; }

//    let mut piece_loc: PieceLocations = PieceLocations::blank();
//    let mut piece_cnt: [[u8; PIECE_CNT]; PLAYER_CNT] = [[0; PIECE_CNT]; PLAYER_CNT];

//    // TODO: Sum of Each Rank is 8
//    for rank in b_rep {
//        let mut sum: u32 = 0;
//        for char_i in rank.chars() {
//            let dig = char_i.to_digit(10);
//            if dig.is_some() {
//                sum += dig.unwrap();
//            } else {
//                let piece = match char_i {
//                    'p' | 'P' => Piece::P,
//                    'n' | 'N' => Piece::N,
//                    'b' | 'B' => Piece::B,
//                    'r' | 'R' => Piece::R,
//                    'q' | 'Q' => Piece::Q,
//                    'k' | 'K' => Piece::K,
//                    _ => return false
//                };
//                let player: Player = if char.is_lowercase() {
//                    Player::Black
//                } else {
//                    Player::White
//                };
//                piece_loc.place(idx as u8, player, piece);
//                piece_cnt[player as usize][piece as usize] += 1;
//                sum += 1;
//            }
//        }
//        if sum != 8 {
//            return false;
//        }
//    }

    // TODO: Board In Check 0, 1, or two times
    //      In Case of two times, never pawn+(pawn, bishop, knight), bishop+bishop, knight+knight

    // TODO: No more than 8 pawns from each color;

    // TODO: No pawns in last or first ranks

    // TODO: If EP square, check for legal EP square







    true
}

/// Generates a board with a Random Position
pub fn gen_rand_legal_board() -> Board {
    gen_rand_board(RandGen::All)
}


pub fn gen_rand_in_check() -> Board {
    gen_rand_board(RandGen::InCheck)
}


pub fn gen_rand_no_check() -> Board {
    gen_rand_board(RandGen::NoCheck)
}

#[derive(Eq, PartialEq)]
enum RandGen {
    InCheck,
    NoCheck,
    All
}

fn gen_rand_board(gen: RandGen) -> Board {
    let side = rand::random::<i32>() % 2;
    loop {
        let mut board = Board::default();
        let mut i = 0;
        let mut moves = board.generate_moves();

        while i < 100 && !moves.is_empty() {
            if i > 4 {
                let mut to_ret = rand::random::<i32>() % cmp::max(17, 100 - i) == 0;
                if gen != RandGen::InCheck {
                    to_ret |= rand::random::<usize>() % 70 == 0;
                }
                if i > 19 {
                    to_ret |= rand::random::<usize>() % 79 == 0;
                    if i > 34 {
                        to_ret |= rand::random::<usize>() % 100 == 0;
                    }
                }

                if to_ret {
                    if gen == RandGen::All { return board; }
                    if gen == RandGen::InCheck && board.in_check() { return board; }
                    if gen == RandGen::NoCheck && !board.in_check() { return board; }
                }
            }
            // apply random move
            let best_move = if gen == RandGen::InCheck && side == i % 2{
                create_rand_move(&board, true)
            } else {
                create_rand_move(&board, false)
            };

            board.apply_move(best_move);

            moves = board.generate_moves();
            i += 1;
        }
    }
}

fn create_rand_move(board: &Board, favorable: bool) -> BitMove {
    let rand_num = if favorable {24} else {14};

    if rand::random::<usize>() % rand_num == 0 {
        RandomBot::best_move_depth(board.shallow_clone(), 1)
    } else if rand::random::<usize>() % 6 == 0 {
        IterativeSearcher::best_move_depth(board.shallow_clone(),3)
    } else if rand::random::<usize>() % 3 == 0 {
        JamboreeSearcher::best_move_depth(board.shallow_clone(),4)
    } else if !favorable && rand::random::<usize>() % 4 < 3 {
        JamboreeSearcher::best_move_depth(board.shallow_clone(),3)
    } else {
        IterativeSearcher::best_move_depth(board.shallow_clone(),4)
    }
}

fn apply_castling(board: &mut Board) -> bool {
    let moves = board.generate_moves();
    for mov in moves {
        if mov.is_castle() {
            board.apply_move(mov);
            return true;
        }
    }
    false
}

#[test]
fn stress_test_rand_moves() {
    let mut i = 0;
    while i < 18 {
        let mut board = gen_rand_legal_board();
        let mov = IterativeSearcher::best_move_depth(board.shallow_clone(),4);
        board.apply_move(mov);
        i += 1;
    }
}
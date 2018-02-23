//! perft, or Performance Test, Move Path Enumeration, tests the correctness of move-generation.
//!
//! Use these functions on a `Board` to test that the correct amount of leaf nodes are created.

use super::{Board,MoveList};

/// Holds all information about the number of nodes counted.
pub struct PerftNodes {
    pub nodes: u64,
    pub captures: u64,
    pub en_passant: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub checkmates: u64,
}

impl Default for PerftNodes {
    fn default() -> Self {
        PerftNodes {
            nodes: 0,
            captures: 0,
            en_passant: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        }
    }
}

/// Returns the number of leaf nodes from generating moves to a certain depth.
pub fn perft(board: &Board, depth: u16) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut pos = board.shallow_clone();
        inner_perft(&mut pos, depth)
    }
}

/// Returns the count of all move types for the leaf nodes up to a certain depth.
pub fn perft_all(board: &Board, depth: u16) -> PerftNodes {
    let mut b = board.shallow_clone();
    let mut perft = PerftNodes::default();
    inner_perft_all(&mut b,depth,&mut perft);
    perft
}


fn inner_perft(board: &mut Board, depth: u16) -> u64 {
    let moves: MoveList = board.generate_moves();

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut count: u64 = 0;

    for mov in moves {
        board.apply_move(mov);
        count += inner_perft(board, depth - 1);
        board.undo_move();
    }

    count
}

fn inner_perft_all(board: &mut Board, depth: u16, perft: &mut PerftNodes) {
    let moves: MoveList = board.generate_moves();

    if depth == 0 {
        perft.nodes += 1;
        if board.in_check() {
            perft.checks += 1;
            if moves.is_empty() {
                perft.checkmates += 1;
            }
        }
    } else {
        for mov in moves {
            if depth == 1 {
                if mov.is_capture() { perft.captures += 1 }
                if mov.is_en_passant() { perft.en_passant += 1 }
                if mov.is_castle() { perft.castles += 1 }
                if mov.is_promo() { perft.promotions += 1 }
            }
            board.apply_move(mov);
            inner_perft_all(board, depth - 1, perft);
            board.undo_move();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_pos_perft() {
        let b: Board = Board::default();
        assert_eq!(1, perft(&b,0));
        assert_eq!(20, perft(&b,1));
        assert_eq!(400, perft(&b,2));
        assert_eq!(8902, perft(&b,3));
        assert_eq!(197_281, perft(&b,4));
        assert_eq!(4_865_609, perft(&b,5));
    }

    #[test]
    fn start_pos_perft_all() {
        let b: Board = Board::default();
        check_perft(perft_all(&b,3),
                    8902, 34, 0, 0, 12, 0);
        check_perft(perft_all(&b,4),
                    197_281, 1576, 0, 0, 469, 8);
        check_perft(perft_all(&b,5),
                    4_865_609, 82_719, 258, 0, 27351, 347);
    }

    fn check_perft(perft: PerftNodes,
                   nodes: u64,      captures: u64, en_passant: u64,
                   promotions: u64, checks: u64,   checkmates: u64) {

        assert_eq!(perft.nodes, nodes);
        assert_eq!(perft.captures, captures);
        assert_eq!(perft.en_passant, en_passant);
        assert_eq!(perft.promotions, promotions);
        assert_eq!(perft.checks, checks);
        assert_eq!(perft.checkmates, checkmates);
    }
}

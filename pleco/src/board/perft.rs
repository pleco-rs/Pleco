//! perft, or Performance Test, Move Path Enumeration, tests the correctness of move-generation.
//!
//! Use these functions on a [`Board`] to test that the correct amount of leaf nodes are created.
//!
//! [`Board`]: ../struct.Board.html

use super::{Board, MoveList};

/// Holds all information about the number of nodes counted.
pub struct PerftNodes {
    /// Total number of nodes counted.
    pub nodes: u64,
    /// Number of capturing moves, including en-passant moves.
    pub captures: u64,
    /// Number of En-Passant moves.
    pub en_passant: u64,
    /// Number of Castles.
    pub castles: u64,
    /// The number of promotions
    pub promotions: u64,
    /// The number of checking moves.
    pub checks: u64,
    /// The number of moves resulting in a checkmate.
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

impl PerftNodes {
    /// Checks for the correct number of nodes in each category. If the results don't
    /// match, panics with an error-message containing the failed checks.
    pub fn check(
        &self,
        nodes: u64,
        captures: u64,
        en_passant: u64,
        castles: u64,
        promotions: u64,
        checks: u64,
        checkmates: u64,
    ) {
        if self.captures != captures
            || self.en_passant != en_passant
            || self.promotions != promotions
            || self.checks != checks
            || self.checkmates != checkmates
            || self.castles != castles
            || self.nodes != nodes
        {
            panic!(
                "\n Perft did not return the correct results!\
            \n total nodes {}, expected: {}, difference: {}\
            \n captures {}, expected: {}, difference: {}\
            \n en_passant {}, expected: {}, difference: {}\
            \n promotions {}, expected: {}, difference: {}\
            \n checks {}, expected: {}, difference: {}\
            \n checkmates {}, expected: {}, difference: {}\
            \n castles {}, expected: {}, difference: {}\n",
                self.nodes,
                nodes,
                nodes - self.nodes,
                self.captures,
                captures,
                captures as i64 - self.captures as i64,
                self.en_passant,
                en_passant,
                en_passant as i64 - self.en_passant as i64,
                self.promotions,
                promotions,
                promotions as i64 - self.promotions as i64,
                self.checks,
                checks,
                checks as i64 - self.checks as i64,
                self.checkmates,
                checkmates as i64,
                checkmates as i64 - self.checkmates as i64,
                self.castles,
                castles,
                castles as i64 - self.castles as i64
            );
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
    inner_perft_all(&mut b, depth, &mut perft);
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
                if mov.is_capture() {
                    perft.captures += 1
                }
                if mov.is_en_passant() {
                    perft.en_passant += 1
                }
                if mov.is_castle() {
                    perft.castles += 1
                }
                if mov.is_promo() {
                    perft.promotions += 1
                }
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
        let b: Board = Board::start_pos();
        assert_eq!(1, perft(&b, 0));
        assert_eq!(20, perft(&b, 1));
        assert_eq!(400, perft(&b, 2));
        assert_eq!(8902, perft(&b, 3));
        assert_eq!(197_281, perft(&b, 4));
        assert_eq!(4_865_609, perft(&b, 5));
    }

    #[test]
    fn start_pos_perft_all() {
        let b: Board = Board::start_pos();
        perft_all(&b, 3).check(8902, 34, 0, 0, 0, 12, 0);
        perft_all(&b, 4).check(197_281, 1576, 0, 0, 0, 469, 8);
        perft_all(&b, 5).check(4_865_609, 82_719, 258, 0, 0, 27351, 347);
    }

    #[test]
    fn perft_kiwipete() {
        let b: Board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap();
        assert_eq!(48, perft(&b, 1));
        assert_eq!(2039, perft(&b, 2));
        assert_eq!(97862, perft(&b, 3));
        assert_eq!(4085603, perft(&b, 4));
        assert_eq!(193690690, perft(&b, 5));
    }

    // This passes, but we're gonna ignore it as it takes a long time to use.
    #[ignore]
    #[test]
    fn perft_kiwipete_all() {
        let b: Board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap();
        perft_all(&b, 3).check(97862, 17102, 45, 3162, 0, 993, 1);
        perft_all(&b, 4).check(4085603, 757163, 1929, 128013, 15172, 25523, 43);
        perft_all(&b, 5).check(193690690, 35043416, 73365, 4993637, 8392, 3309887, 30171);
    }

    #[test]
    fn perft_board_3() {
        let b: Board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();
        assert_eq!(14, perft(&b, 1));
        assert_eq!(191, perft(&b, 2));
        assert_eq!(2812, perft(&b, 3));
        assert_eq!(43238, perft(&b, 4));
        assert_eq!(674624, perft(&b, 5));
    }

    #[test]
    fn perft_board_5() {
        let b: Board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8 ").unwrap();
        assert_eq!(44, perft(&b, 1));
        assert_eq!(1_486, perft(&b, 2));
        assert_eq!(62_379, perft(&b, 3));
        assert_eq!(2_103_487, perft(&b, 4));
        assert_eq!(89_941_194, perft(&b, 5));
    }

    #[test]
    fn perft_board_6() {
        let b: Board = Board::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        assert_eq!(46, perft(&b, 1));
        assert_eq!(2_079, perft(&b, 2));
        assert_eq!(89_890, perft(&b, 3));
        assert_eq!(3_894_594, perft(&b, 4));
    }
}

//! Contains various FEN (Forsyth–Edwards Notation) functions and constants.
//!
//! A FEN string is a way of describing the particular state of a chess game.
//!
//! For example, the start position fen is
//! `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`.
//!
//! See [this Wikipedia article](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation)
//! for more information.

use super::super::core::sq::NO_SQ;
use super::{Board, FenBuildError};
use {BitBoard, PieceType, Player, Rank, SQ};

/// The fen string for the start position.
pub const OPENING_POS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[doc(hidden)]
pub static STANDARD_FENS_START_POS: [&str; 1] = [OPENING_POS_FEN];

#[doc(hidden)]
pub static STANDARD_FENS_MIDDLE_POS: [&str; 27] = [
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
    "8/3p3B/5p2/5P2/p7/PP5b/k7/6K1 w - - 0 1",
];

#[doc(hidden)]
pub static STANDARD_FENS_5_PIECE_POS: [&str; 3] = [
    "8/8/8/8/5kp1/P7/8/1K1N4 w - - 0 1",  // Kc2 - mate
    "8/8/8/5N2/8/p7/8/2NK3k w - - 0 1",   // Na2 - mate
    "8/3k4/8/8/8/4B3/4KB2/2B5 w - - 0 1", // draw
];

#[doc(hidden)]
pub static STANDARD_FENS_6_PIECE_POS: [&str; 3] = [
    "8/8/1P6/5pr1/8/4R3/7k/2K5 w - - 0 1",  // Re5 - mate
    "8/2p4P/8/kr6/6R1/8/8/1K6 w - - 0 1",   // Ka2 - mate
    "8/8/3P3k/8/1p6/8/1P6/1K3n2 b - - 0 1", // Nd2 - draw
];

#[doc(hidden)]
pub static STANDARD_FEN_7_PIECE_POS: [&str; 1] = [
    "8/R7/2q5/8/6k1/8/1P5p/K6R w - - 0 124", // Draw
];

#[doc(hidden)]
pub static STANDARD_FEN_MATE_STALEMATE: [&str; 4] = [
    "6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1",
    "r2r1n2/pp2bk2/2p1p2p/3q4/3PN1QP/2P3R1/P4PP1/5RK1 w - - 0 1",
    "8/8/8/8/8/6k1/6p1/6K1 w - - 0 1",
    "7k/7P/6K1/8/3B4/8/8/8 b - - 0 1",
];

lazy_static! {
    #[doc(hidden)]
    pub static ref ALL_FENS: Vec<&'static str> = {
        let mut vec = Vec::new();
        for fen in &STANDARD_FENS_START_POS {vec.push(*fen); }
        for fen in &STANDARD_FENS_MIDDLE_POS {vec.push(*fen); }
        for fen in &STANDARD_FENS_5_PIECE_POS {vec.push(*fen); }
        for fen in &STANDARD_FENS_6_PIECE_POS {vec.push(*fen); }
        for fen in &STANDARD_FEN_7_PIECE_POS {vec.push(*fen); }
        for fen in &STANDARD_FEN_MATE_STALEMATE {vec.push(*fen); }
        vec
    };
}

// "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
// https://chess.stackexchange.com/questions/1482/how-to-know-when-a-fen-position-is-legal

/// Returns if a [`Board`] generated from a fen string is a legal position.
///
/// This is called automatically by [`Board::new_from_fen`] after a [`Board`] is created, so
/// this method doesn't have great individual use.
///
/// [`Board`]: ../struct.Board.html
/// [`Board::new_from_fen`]: ../struct.Board.html#method.new_from_fen
pub fn is_valid_fen(board: Board) -> Result<Board, FenBuildError> {
    let checks = board.checkers();
    let num_checks = checks.count_bits();
    // Can't be more than 2 checking pieces at a time
    if num_checks > 2 {
        return Err(FenBuildError::IllegalNumCheckingPieces { num: num_checks });
    }
    if num_checks == 2 {
        let sq_1bb = checks.lsb();
        let sq_2 = (checks & !sq_1bb).to_sq();
        let sq_1 = sq_1bb.to_sq();
        let piece_1 = board.piece_at_sq(sq_1).type_of();
        let piece_2 = board.piece_at_sq(sq_2).type_of();

        // Some combinations of pieces can never check the king at the same time.
        if piece_1 == PieceType::P {
            if piece_2 == PieceType::B || piece_2 == PieceType::N || piece_2 == PieceType::P {
                return Err(FenBuildError::IllegalCheckState { piece_1, piece_2 });
            }
        } else if piece_1 == PieceType::B && (piece_2 == PieceType::P || piece_2 == PieceType::B)
            || piece_1 == PieceType::N && (piece_2 == PieceType::P || piece_2 == PieceType::N)
        {
            return Err(FenBuildError::IllegalCheckState { piece_1, piece_2 });
        }
    }

    let all_pawns: BitBoard =
        board.piece_bb_both_players(PieceType::P) & (BitBoard::RANK_1 | BitBoard::RANK_8);

    // No pawns on Rank 1 or 8
    if all_pawns.is_not_empty() {
        return Err(FenBuildError::PawnOnLastRow);
    }

    // Check for more pawns than possible
    let white_pawns = board.count_piece(Player::White, PieceType::P);
    let black_pawns = board.count_piece(Player::Black, PieceType::P);
    if white_pawns > 8 {
        return Err(FenBuildError::TooManyPawns {
            player: Player::White,
            num: white_pawns,
        });
    }

    if black_pawns > 8 {
        return Err(FenBuildError::TooManyPawns {
            player: Player::Black,
            num: black_pawns,
        });
    }

    // check for correct en-passant square rank
    let ep_sq = board.ep_square();
    if ep_sq != NO_SQ {
        match board.turn() {
            Player::White => {
                if ep_sq.rank() != Rank::R6 {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }

                let ep_p_sq = ep_sq - SQ(8);

                let (ep_player, ep_piece) = board.piece_at_sq(ep_p_sq).player_piece_lossy();

                if ep_piece == PieceType::None {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }

                if ep_player != Player::Black || ep_piece != PieceType::P {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }
            }
            Player::Black => {
                if ep_sq.rank() != Rank::R3 {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }

                let ep_p_sq = ep_sq + SQ(8);

                let (ep_player, ep_piece) = board.piece_at_sq(ep_p_sq).player_piece_lossy();

                if ep_piece == PieceType::None {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }

                if ep_player != Player::White || ep_piece != PieceType::P {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: ep_sq.to_string(),
                    });
                }
            }
        }
    }

    Ok(board)
}

#[cfg(test)]
mod tests {
    use Board;

    const EXTRA_PAWNS: &str = "rnbqkbnr/pppppppp/8/8/8/7P/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn fen_extra_pawns() {
        assert!(Board::from_fen(EXTRA_PAWNS).is_err());
    }
}

use super::{Board,FenBuildError};
use {BitBoard,Piece,Player};

// "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
// https://chess.stackexchange.com/questions/1482/how-to-know-when-a-fen-position-is-legal
pub fn is_valid_fen(board: Board) -> Result<Board,FenBuildError> {
    let checks = board.checkers();
    let num_checks = checks.count_bits();
    if num_checks > 2 { return Err(FenBuildError::IllegalNumCheckingPieces {num: num_checks}) }
    if num_checks == 2 {
        let sq_1bb = checks.lsb();
        let sq_2 = (checks & !sq_1bb).to_sq();
        let sq_1 = sq_1bb.to_sq();
        let piece_1 = board.piece_at_sq(sq_1).unwrap();
        let piece_2 = board.piece_at_sq(sq_2).unwrap();
        if piece_1 == Piece::P {
            if piece_2 == Piece::B || piece_2 == Piece::N || piece_2 == Piece::P {
                return Err(FenBuildError::IllegalCheckState {piece_1, piece_2});
            }
        } else if piece_1 == Piece::B {
            if piece_2 == Piece::P || piece_2 == Piece::B {
                return Err(FenBuildError::IllegalCheckState {piece_1, piece_2});
            }
        } else if piece_1 == Piece::N {
            if piece_2 == Piece::P || piece_2 == Piece::N {
                return Err(FenBuildError::IllegalCheckState { piece_1, piece_2 });
            }
        }
    }

    let all_pawns: BitBoard = board.piece_bb_both_players(Piece::P) & (BitBoard::RANK_1 | BitBoard::RANK_8 );
    if all_pawns.is_not_empty() {
        return Err(FenBuildError::PawnOnLastRow);
    }

    let white_pawns = board.count_piece(Player::White,Piece::P);
    let black_pawns = board.count_piece(Player::Black,Piece::P);
    if white_pawns > 8 {
        return Err(FenBuildError::TooManyPawns { player: Player::White, num: white_pawns });
    }

    if black_pawns > 8 {
        return Err(FenBuildError::TooManyPawns { player: Player::Black, num: black_pawns });
    }
    // TODO: If EP square, check for legal EP square

    Ok(board)
}
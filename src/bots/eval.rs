use board::Board;
use std::i16;
use templates::*;
use bit_twiddles::*;

pub struct Eval<'a> {
    board: &'a Board,
    us: Player,
    them: Player
}

pub const MIN: i16 = 0b1000000000000000;
pub const MAX: i16 = 0b0111111111111111;

pub const INFINITY: i16 = 30000;
pub const NEG_INFINITY: i16 = -30000;
pub const STALEMATE: i16 = 0;

pub const PAWN_VALUE: i16 = 100;
pub const KNIGHT_VALUE: i16 = 300;
pub const BISHOP_VALUE: i16 = 300;
pub const ROOK_VALUE: i16 = 500;
pub const QUEEN_VALUE: i16 = 800;
pub const KING_VALUE: i16 = 350;

pub const CASTLE_ABILITY: i16 = 10;

pub const KING_BOTTOM: i16 = 25;

// Pawn, Knight, Bishop, Rook, Queen, King
pub const PIECE_VALS: [i16; PIECE_CNT] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, KING_VALUE];


impl <'a> Eval<'a> {
    pub fn eval(board: &Board) -> i16 {
        let eval = Eval{
            board: &board,
            us: board.turn(),
            them: other_player(board.turn())
        };
        eval.eval_all()
    }
}


impl <'a> Eval<'a> {
    fn eval_all(&self) -> i16 {
        let mut score: i16 = 0;
        for &piece in ALL_PIECES.iter() {
            score += (self.board.count_piece(self.us,piece) as i16).wrapping_sub(self.board.count_piece(self.them,piece) as i16) * PIECE_VALS[piece as usize];
        }
        score + self.eval_castling() + self.eval_king_pos()
    }

    fn eval_castling(&self) -> i16 {
        let mut score: i16 = 0;
//        if self.board.can_castle(self.us, CastleType::KingSide) {score += CASTLE_ABILITY}
//        if self.board.can_castle(self.us, CastleType::QueenSide) {score += CASTLE_ABILITY}
//        if self.board.can_castle(self.them, CastleType::KingSide) {score -= CASTLE_ABILITY}
//        if self.board.can_castle(self.them, CastleType::QueenSide) {score -= CASTLE_ABILITY}
        score
    }

    fn eval_king_pos(&self) -> i16 {
        let mut score: i16 = 0;
//        if rank_of_sq(self.board.king_sq(self.us)) == Rank::R1 || rank_of_sq(self.board.king_sq(self.us)) == Rank::R8 {score += KING_BOTTOM}
//        if rank_of_sq(self.board.king_sq(self.them)) == Rank::R1 || rank_of_sq(self.board.king_sq(self.them)) == Rank::R8 {score -= KING_BOTTOM}

        score
    }

    fn eval_pawns(&self) -> i16 {
        let mut score: i16 = 0;
        let us_p = self.board.piece_bb(self.us, Piece::P);
        let them_p = self.board.piece_bb(self.them, Piece::P);

        let files_us: [u8; FILE_CNT] = [popcount64(FILE_A & us_p),
                                        popcount64(FILE_B & us_p),
                                        popcount64(FILE_C & us_p),
                                        popcount64(FILE_D & us_p),
                                        popcount64(FILE_E & us_p),
                                        popcount64(FILE_F & us_p),
                                        popcount64(FILE_G & us_p),
                                        popcount64(FILE_H & us_p)];

        let files_them: [u8; FILE_CNT] = [popcount64(FILE_A & them_p),
            popcount64(FILE_B & them_p),
            popcount64(FILE_C & them_p),
            popcount64(FILE_D & them_p),
            popcount64(FILE_E & them_p),
            popcount64(FILE_F & them_p),
            popcount64(FILE_G & them_p),
            popcount64(FILE_H & them_p)];

        for i in 0..FILE_CNT {
            if files_us[i] > 1 {
                score -= (files_us[i] * 5) as i16;
            }
            if files_them[i] > 1 {
                score += (files_them[i] * 5) as i16;
            }

            if i > 0 && i < 7 {
                if files_us[i] != 0 {
                    if files_us[i - 1] != 0 {
                        if files_us[i + 1] != 0 {
                            score += 25;
                        } else {
                            score += 11;
                        }
                    } else if files_us[i + 1] != 0 {
                        score += 11;
                    } else {
                        score -= 19;
                    }
                }

                if files_them[i] != 0 {
                    if files_them[i - 1] != 0 {
                        if files_them[i + 1] != 0 {
                            score -= 25;
                        } else {
                            score -= 11;
                        }
                    } else if files_them[i + 1] != 0 {
                        score -= 11;
                    } else {
                        score += 19;
                    }
                }
            }
        }


        score
    }


}


// TODO Mobility Bonus
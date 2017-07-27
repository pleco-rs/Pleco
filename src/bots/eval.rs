use board::Board;
use std::i16;
use templates::*;
use bit_twiddles::*;
use lazy_static;


lazy_static! {
    pub static ref BISHOP_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(BISHOP_POS_ARRAY)), flatten(BISHOP_POS_ARRAY) ];
    pub static ref KNIGHT_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(KNIGHT_POS_ARRAY)), flatten(KNIGHT_POS_ARRAY) ];
    pub static ref PAWN_POS:   [[i16; SQ_CNT]; PLAYER_CNT] = [   flatten(flip(PAWN_POS_ARRAY)), flatten(PAWN_POS_ARRAY)   ];
}

pub const BISHOP_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_8
    [-5, 10,  5,  8,  8,  5, 10, -5],
    [-5,  5,  3,  8,  8,  3,  5, -5],
    [-5,  3, 10,  3,  3, 10,  3, -5],
    [-5,  3, 10,  3,  3, 10,  3, -5],
    [-5,  5,  3,  8,  8,  3,  5, -5],
    [-5, 10,  5,  8,  8,  5, 10, -5],
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_1
];

pub const KNIGHT_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_8
    [ -8,  0,  0,  3,  3,  0,  0,  -8],
    [ -8,  0, 10,  8,  8, 10,  0,  -8],
    [ -8,  0,  8, 10, 10,  8,  0,  -8],
    [ -8,  0,  8, 10, 10,  8,  0,  -8],
    [ -8,  0, 10,  8,  8, 10,  0,  -8],
    [ -8,  0,  0,  3,  3,  0,  0,  -8],
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_1
];

pub const PAWN_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [0,  0,  0,  0,  0,  0,  0, 0], // RANK_8
    [5, 10, 15, 20, 20, 15, 10, 5],
    [4,  8, 12, 16, 16, 12,  8, 4],
    [0,  6,  9, 10, 10,  9,  6, 0],
    [0,  4,  6, 10, 10,  6,  4, 0],
    [0,  2,  3,  4,  4,  3,  2, 0],
    [0,  0,  0, -5, -5,  0,  0, 0],
    [0,  0,  0,  0,  0,  0,  0, 0], // RANK_1
];

//  Flips the board, so rank_1 becomes rank_8, rank_8 becomes rank_1, rank_2 becomes rank_7, etc
fn flip(arr: [[i16; FILE_CNT]; RANK_CNT]) -> [[i16; FILE_CNT]; RANK_CNT] {
    let mut new_arr: [[i16; FILE_CNT]; RANK_CNT] = [[0; FILE_CNT]; RANK_CNT];
    for i in 0..RANK_CNT {
        new_arr[i] = arr[7 - i];
    }
    new_arr
}

// Flattens 2D array to a singular 1D array
fn flatten(arr: [[i16; FILE_CNT]; RANK_CNT] ) -> [i16; SQ_CNT] {
    let mut new_arr: [i16; SQ_CNT] = [0; SQ_CNT];
    for i in 0..SQ_CNT {
        new_arr[i] = arr[i / 8][i % 8];
    }
    new_arr
}

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
pub const CASTLE_BONUS: i16 = 32;

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
        self.eval_piece_counts()
            + self.eval_castling()
            + self.eval_king_pos()
            + self.eval_bishop_pos()
            + self.eval_pawns()
            + self.eval_knight_pos()
    }

    fn eval_piece_counts(&self) -> i16 {
        ((self.board.count_piece(self.us,Piece::P) as i16 - self.board.count_piece(self.them,Piece::P) as i16) * PAWN_VALUE)
            + ((self.board.count_piece(self.us,Piece::N) as i16 - self.board.count_piece(self.them,Piece::N) as i16) * KNIGHT_VALUE)
            + ((self.board.count_piece(self.us,Piece::B) as i16 - self.board.count_piece(self.them,Piece::B) as i16) * BISHOP_VALUE)
            + ((self.board.count_piece(self.us,Piece::R) as i16 - self.board.count_piece(self.them,Piece::R) as i16) * ROOK_VALUE)
            + ((self.board.count_piece(self.us,Piece::Q) as i16 - self.board.count_piece(self.them,Piece::Q) as i16) * QUEEN_VALUE)
    }

    fn eval_castling(&self) -> i16 {
        let mut score: i16 = 0;
        if self.board.has_castled(self.us) {
            score += CASTLE_BONUS
        } else {
            if self.board.can_castle(self.us, CastleType::KingSide) {score += CASTLE_ABILITY}
            if self.board.can_castle(self.us, CastleType::QueenSide) {score += CASTLE_ABILITY}
        }
        if self.board.has_castled(self.them) {
            score -= CASTLE_BONUS
        } else {
            if self.board.can_castle(self.them, CastleType::KingSide) {score -= CASTLE_ABILITY}
            if self.board.can_castle(self.them, CastleType::QueenSide) {score -= CASTLE_ABILITY}
        }
        score
    }

    fn eval_king_pos(&self) -> i16 {
        let mut score: i16 = 0;
//        if rank_of_sq(self.board.king_sq(self.us)) == Rank::R1 || rank_of_sq(self.board.king_sq(self.us)) == Rank::R8 {score += KING_BOTTOM}
//        if rank_of_sq(self.board.king_sq(self.them)) == Rank::R1 || rank_of_sq(self.board.king_sq(self.them)) == Rank::R8 {score -= KING_BOTTOM}

        score
    }

    fn eval_bishop_pos(&self) -> i16 {
        let mut score: i16 = 0;
        let mut us_b = self.board.piece_bb(self.us, Piece::B);
        let mut them_pb = self.board.piece_bb(self.them, Piece::B);
        while us_b != 0 {
            let lsb = lsb(us_b);
            score +=  BISHOP_POS[self.us as usize][bb_to_sq(lsb) as usize];
            us_b &= !lsb;
        }

        while them_pb != 0 {
            let lsb = lsb(them_pb);
            score -=  BISHOP_POS[self.them as usize][bb_to_sq(lsb) as usize];
            them_pb &= !lsb;
        }
        score
    }

    fn eval_knight_pos(&self) -> i16 {
        let mut score: i16 = 0;
        let mut us_b = self.board.piece_bb(self.us, Piece::N);
        let mut them_pb = self.board.piece_bb(self.them, Piece::N);
        while us_b != 0 {
            let lsb = lsb(us_b);
            score +=  BISHOP_POS[self.us as usize][bb_to_sq(lsb) as usize];
            us_b &= !lsb;
        }

        while them_pb != 0 {
            let lsb = lsb(them_pb);
            score -=  KNIGHT_POS[self.them as usize][bb_to_sq(lsb) as usize];
            them_pb &= !lsb;
        }
        score
    }

    fn eval_pawns(&self) -> i16 {
        let mut score: i16 = 0;
        let us_p = self.board.piece_bb(self.us, Piece::P);
        let them_p = self.board.piece_bb(self.them, Piece::P);

        let mut us_pos_m = us_p;
        while us_pos_m != 0 {
            let lsb = lsb(us_pos_m);
            score +=  PAWN_POS[self.us as usize][bb_to_sq(lsb) as usize];
            us_pos_m &= !lsb;
        }

        let mut them_pos_m = them_p;
        while them_pos_m != 0 {
            let lsb = lsb(them_pos_m);
            score -=  PAWN_POS[self.them as usize][bb_to_sq(lsb) as usize];
            them_pos_m &= !lsb;
        }


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
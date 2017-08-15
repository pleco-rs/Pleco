use board::Board;
use std::i16;
use templates::*;
use bit_twiddles::*;

lazy_static! {
    pub static ref BISHOP_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(BISHOP_POS_ARRAY)), flatten(BISHOP_POS_ARRAY) ];
    pub static ref KNIGHT_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(KNIGHT_POS_ARRAY)), flatten(KNIGHT_POS_ARRAY) ];
    pub static ref PAWN_POS:   [[i16; SQ_CNT]; PLAYER_CNT] = [   flatten(flip(PAWN_POS_ARRAY)), flatten(PAWN_POS_ARRAY)   ];
}

pub const BISHOP_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_8
    [-5, 10, 5, 8, 8, 5, 10, -5],
    [-5, 5, 3, 8, 8, 3, 5, -5],
    [-5, 3, 10, 3, 3, 10, 3, -5],
    [-5, 3, 10, 3, 3, 10, 3, -5],
    [-5, 5, 3, 8, 8, 3, 5, -5],
    [-5, 10, 5, 8, 8, 5, 10, -5],
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_1
];

pub const KNIGHT_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_8
    [-8, 0, 0, 3, 3, 0, 0, -8],
    [-8, 0, 10, 8, 8, 10, 0, -8],
    [-8, 0, 8, 10, 10, 8, 0, -8],
    [-8, 0, 8, 10, 10, 8, 0, -8],
    [-8, 0, 10, 8, 8, 10, 0, -8],
    [-8, 0, 0, 3, 3, 0, 0, -8],
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_1
];

pub const PAWN_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [0, 0, 0, 0, 0, 0, 0, 0], // RANK_8
    [5, 10, 15, 20, 20, 15, 10, 5],
    [4, 8, 12, 16, 16, 12, 8, 4],
    [0, 6, 9, 10, 10, 9, 6, 0],
    [0, 4, 6, 10, 10, 6, 4, 0],
    [0, 2, 3, 4, 4, 3, 2, 0],
    [0, 0, 0, -5, -5, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0], // RANK_1
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
fn flatten(arr: [[i16; FILE_CNT]; RANK_CNT]) -> [i16; SQ_CNT] {
    let mut new_arr: [i16; SQ_CNT] = [0; SQ_CNT];
    for i in 0..SQ_CNT {
        new_arr[i] = arr[i / 8][i % 8];
    }
    new_arr
}

pub struct Eval<'a> {
    board: &'a Board,
    us: Player,
    them: Player,
}



pub const INFINITY: i16 = 30_002;
pub const NEG_INFINITY: i16 = -30_001;
pub const STALEMATE: i16 = 0;

pub const PAWN_VALUE: i16 = 100;
pub const KNIGHT_VALUE: i16 = 300;
pub const BISHOP_VALUE: i16 = 300;
pub const ROOK_VALUE: i16 = 500;
pub const QUEEN_VALUE: i16 = 800;
pub const KING_VALUE: i16 = 350;

pub const CASTLE_ABILITY: i16 = 7;
pub const CASTLE_BONUS: i16 = 20;

pub const KING_BOTTOM: i16 = 11;

pub const MATE: i16 = -25000;
pub const CHECK: i16 = 20;

// Pawn, Knight, Bishop, Rook, Queen, King
pub const PIECE_VALS: [i16; PIECE_CNT] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];


impl<'a> Eval<'a> {
    pub fn eval_low(board: &Board) -> i16 {
        let eval = Eval {
            board: board,
            us: board.turn(),
            them: other_player(board.turn()),
        };
        eval.eval_simple()
    }
}


impl<'a> Eval<'a> {
    fn eval_simple(&self) -> i16 {
        self.eval_piece_counts() + self.eval_castling() + self.eval_king_pos() +
            self.eval_bishop_pos() + self.eval_knight_pos() +
            eval_pawns(self.board, self.us) - eval_pawns(self.board, self.them) +
            eval_king_blockers_pinners(self.board, self.us) -
            eval_king_blockers_pinners(self.board, self.them) +
            if self.board.rule_50() > 40 { -33 } else { 0 }
    }

    fn eval_piece_counts(&self) -> i16 {
        ((self.board.count_piece(self.us, Piece::P) as i16 -
              self.board.count_piece(self.them, Piece::P) as i16) * PAWN_VALUE) +
            ((self.board.count_piece(self.us, Piece::N) as i16 -
                  self.board.count_piece(self.them, Piece::N) as i16) * KNIGHT_VALUE) +
            ((self.board.count_piece(self.us, Piece::B) as i16 -
                  self.board.count_piece(self.them, Piece::B) as i16) * BISHOP_VALUE) +
            ((self.board.count_piece(self.us, Piece::R) as i16 -
                  self.board.count_piece(self.them, Piece::R) as i16) * ROOK_VALUE) +
            ((self.board.count_piece(self.us, Piece::Q) as i16 -
                  self.board.count_piece(self.them, Piece::Q) as i16) * QUEEN_VALUE)
    }

    fn eval_castling(&self) -> i16 {
        let mut score: i16 = 0;
        if self.board.has_castled(self.us) {
            score += CASTLE_BONUS
        } else {
            if self.board.can_castle(self.us, CastleType::KingSide) {
                score += CASTLE_ABILITY
            }
            if self.board.can_castle(self.us, CastleType::QueenSide) {
                score += CASTLE_ABILITY
            }
        }
        if self.board.has_castled(self.them) {
            score -= CASTLE_BONUS
        } else {
            if self.board.can_castle(self.them, CastleType::KingSide) {
                score -= CASTLE_ABILITY
            }
            if self.board.can_castle(self.them, CastleType::QueenSide) {
                score -= CASTLE_ABILITY
            }
        }
        score
    }



    fn eval_king_pos(&self) -> i16 {
        let mut score: i16 = 0;
        let us_ksq = self.board.king_sq(self.us);
        let them_ksq = self.board.king_sq(self.them);
        if rank_of_sq(us_ksq) == Rank::R1 || rank_of_sq(us_ksq) == Rank::R8 {
            score += KING_BOTTOM
        }
        if rank_of_sq(them_ksq) == Rank::R1 || rank_of_sq(them_ksq) == Rank::R8 {
            score -= KING_BOTTOM
        }

        if self.board.in_check() {
            score -= CHECK
        }

        let bb_around_us = self.board.magic_helper.king_moves(us_ksq) &
            self.board.get_occupied_player(self.us);
        let bb_around_them = self.board.magic_helper.king_moves(them_ksq) &
            self.board.get_occupied_player(self.them);

        score += popcount64(bb_around_us) as i16 * 9;
        score -= popcount64(bb_around_them) as i16 * 9;

        score
    }

    fn eval_bishop_pos(&self) -> i16 {
        let mut score: i16 = 0;
        let mut us_b = self.board.piece_bb(self.us, Piece::B);
        let mut them_pb = self.board.piece_bb(self.them, Piece::B);
        while us_b != 0 {
            let lsb = lsb(us_b);
            score += BISHOP_POS[self.us as usize][bb_to_sq(lsb) as usize];
            us_b &= !lsb;
        }

        while them_pb != 0 {
            let lsb = lsb(them_pb);
            score -= BISHOP_POS[self.them as usize][bb_to_sq(lsb) as usize];
            them_pb &= !lsb;
        }

        if self.board.count_piece(self.us, Piece::B) > 1 {
            score += 19
        }
        if self.board.count_piece(self.them, Piece::B) > 1 {
            score -= 19
        }

        score
    }

    fn eval_knight_pos(&self) -> i16 {
        let mut score: i16 = 0;
        let mut us_b = self.board.piece_bb(self.us, Piece::N);
        let mut them_pb = self.board.piece_bb(self.them, Piece::N);
        while us_b != 0 {
            let lsb = lsb(us_b);
            score += KNIGHT_POS[self.us as usize][bb_to_sq(lsb) as usize];
            us_b &= !lsb;
        }

        while them_pb != 0 {
            let lsb = lsb(them_pb);
            score -= KNIGHT_POS[self.them as usize][bb_to_sq(lsb) as usize];
            them_pb &= !lsb;
        }
        score
    }
}

fn eval_king_blockers_pinners(board: &Board, turn: Player) -> i16 {
    let mut score: i16 = 0;

    let blockers: BitBoard = board.all_pinned_pieces(turn);

    let them_blockers = blockers & board.get_occupied_player(other_player(turn));

    // Our pieces blocking a check on their king
    let us_blockers = blockers & board.get_occupied_player(turn);

    score += 25 * popcount64(us_blockers) as i16;

    score += 6 * popcount64(them_blockers) as i16;

    score
}

fn eval_pawns(board: &Board, turn: Player) -> i16 {
    let mut score: i16 = 0;

    let pawns_bb = board.piece_bb(turn, Piece::P);
    let mut bb = pawns_bb;
    let mut file_counts: [u8; FILE_CNT] = [0; FILE_CNT];

    let mut sqs_defended: BitBoard = 0;

    while bb != 0 {
        let lsb = lsb(bb);
        let sq = bb_to_sq(lsb);
        sqs_defended |= board.magic_helper.pawn_attacks_from(sq, turn);
        file_counts[(sq % 8) as usize] += 1;
        score += PAWN_POS[turn as usize][sq as usize];
        bb &= !lsb;
    }

    // Add score for squares attacked by pawns
    score += popcount64(sqs_defended) as i16;

    // Add score for pawns defending other pawns
    sqs_defended &= pawns_bb;
    score += 3 * popcount64(sqs_defended) as i16;

    for i in 0..FILE_CNT {
        if file_counts[i] > 1 {
            score -= (file_counts[i] * 3) as i16;
        }
        if i > 0 && i < 7 && file_counts[i] > 0 {
            if file_counts[i - 1] != 0 {
                if file_counts[i + 1] != 0 {
                    score += 7;
                } else {
                    score += 3;
                }
            } else if file_counts[i + 1] != 0 {
                score += 3;
            } else {
                score -= 4;
            }
        }
    }

    score


}


// TODO Mobility Bonus

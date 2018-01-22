use super::super::core::{Player};
use super::super::core::bitboard::BitBoard;
use super::super::core::sq::SQ;
use super::super::core::masks::PLAYER_CNT;
use super::super::board::castle_rights::Castling;

pub struct PawnTable {

}

pub struct Entry {
    key: u64,
    score: i16,
    passed_pawns: [BitBoard; PLAYER_CNT],
    pawn_attacks: [BitBoard; PLAYER_CNT],
    pawn_attacks_span: [BitBoard; PLAYER_CNT],
    king_squares: [SQ; PLAYER_CNT],
    king_safety_score: [i16; PLAYER_CNT],
    weak_unopposed: [i16; PLAYER_CNT],
    castling_rights: [Castling; PLAYER_CNT],
    semiopen_files: [u8; PLAYER_CNT],
    pawns_on_squares: [[u8; PLAYER_CNT]; PLAYER_CNT], // [color][light/dark squares]
    asymmetry: i16,
    open_files: u8
}

impl Entry {
    pub fn pawns_score(&self) -> i16 {
        self.score
    }

    pub fn pawn_attacks(&self, player: Player) -> BitBoard {
        self.pawn_attacks[player as usize]
    }

    pub fn passed_pawns(&self, player: Player) -> BitBoard {
        self.passed_pawns[player as usize]
    }

    pub fn pawn_attacks_span(&self, player: Player) -> BitBoard {
        self.pawn_attacks_span[player as usize]
    }

    pub fn weak_unopposed(&self, player: Player) -> i16 {
        self.weak_unopposed[player as usize]
    }

    pub fn asymmetry(&self) -> i16 {
        self.asymmetry
    }

    pub fn open_files(&self) -> u8 {
        self.open_files
    }

}
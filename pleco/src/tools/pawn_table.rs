use {Player,File,SQ,BitBoard};
use super::super::core::masks::PLAYER_CNT;
use super::super::board::castle_rights::Castling;

use super::TableBase;

pub struct PawnTable {
    table: TableBase<Entry>
}

impl PawnTable {
    pub fn new(size: usize) -> Self {
        PawnTable {
            table: TableBase::new(size).unwrap()
        }
    }

    pub fn get(&self, key: u64) -> &mut Entry {
        self.table.get_mut(key)
    }
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
    // per
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

    pub fn semiopen_file(&self, player: Player, file: File) -> bool {
        self.semiopen_files[player as usize] & (1 << file as u8) != 0
    }

    pub fn semiopen_side(&self, player: Player, file: File, left_side: bool) -> bool {
        let side_mask: u8 = if left_side {
            file.left_side_mask()
        } else {
            file.right_side_mask()
        };
        self.semiopen_files[player as usize] & side_mask != 0
    }

    // returns count
    pub fn pawns_on_same_color_squares(&self, player: Player, sq: SQ) -> u8 {
        self.pawns_on_squares[player as usize][sq.square_color_index()]
    }

    

}
use {Player,File,SQ,BitBoard};
use super::super::core::masks::PLAYER_CNT;
use super::super::core::score::*;
use super::super::board::castle_rights::Castling;

use super::TableBase;

const ISOLATED: Score = Score(13, 18);

const BACKWARDS: Score = Score(24, 12);

lazy_static!{
    static ref CONNECTED: [[[[Score; 2]; 2] ;3]; 8] = init_connected();
}

#[inline(always)]
fn init_connected() -> [[[[Score; 2]; 2] ;3]; 8] {
    let seed: [i32; 8] = [0, 13, 24, 18, 76, 100, 175, 330];
    let mut a: [[[[Score; 2]; 2] ;3]; 8] = [[[[Score(0,0); 2]; 2] ;3]; 8];
    for opposed in 0..2 {
        for phalanx in 0..2 {
            for support in 0..3 {
                for r in 1..8 {
                    let mut v: i32 = 17 * support;
                    v += (seed[r] + (phalanx * ((seed[r as usize +1] - seed[r as usize]) / 2))) >> opposed;
                    let eg: i16 = (v * (r as i32 - 2) / 4) as i16;
                    a[opposed as usize][phalanx as usize][support as usize][r as usize] = Score(v as i16, eg);
                }
            }
        }
    }
    a
}

pub struct PawnTable {
    table: TableBase<Entry>,
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
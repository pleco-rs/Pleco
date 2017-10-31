
use super::bitboard::BitBoard;
use super::masks::*;
use super::templates::*;


use std::fmt;
use std::ops::*;



#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
pub struct SQ(pub u8);

impl_bit_ops!(SQ, u8);

pub const NO_SQ: SQ = SQ(64);


impl SQ {
    #[inline]
    pub fn to_string(self) -> String {
        assert!(self.is_okay());
        let mut str = String::default();
        str.push(FILE_DISPLAYS[self.file_of_sq() as usize]);
        str.push(RANK_DISPLAYS[self.rank_of_sq() as usize]);
        str
    }

    #[inline]
    pub fn is_okay(self) -> bool {
        self.0 < 64
    }

    #[inline]
    pub fn sq_to_bb(self) -> BitBoard {
        assert!(self.is_okay());
        BitBoard((1 as u64).wrapping_shl(self.0 as u32))
    }



    #[inline]
    pub fn rank_bb(self) -> BitBoard {
        BitBoard(RANK_BB[self.rank_of_sq() as usize])
    }

    #[inline]
    pub fn rank_of_sq(self) -> Rank {
        ALL_RANKS[(self.0 >> 3) as usize]
    }

    #[inline]
    pub fn rank_idx_of_sq(self) -> u8 {
        (self.0 >> 3) as u8
    }

    #[inline]
    pub fn file_bb(self) -> BitBoard {
        BitBoard(FILE_BB[self.file_of_sq() as usize])
    }

    #[inline]
    pub fn file_of_sq(self) -> File {
        ALL_FILES[(self.0 & 0b0000_0111) as usize]
    }

    #[inline]
    pub fn file_idx_of_sq(self) -> u8 {
        (self.0 & 0b0000_0111) as u8
    }

    pub fn castle_rights_mask(self) -> u8 {
        match self.0 {
            ROOK_WHITE_KSIDE_START => C_WHITE_K_MASK,
            ROOK_WHITE_QSIDE_START => C_WHITE_Q_MASK,
            ROOK_BLACK_KSIDE_START => C_BLACK_K_MASK,
            ROOK_BLACK_QSIDE_START => C_BLACK_Q_MASK,
            WHITE_KING_START => C_WHITE_K_MASK | C_WHITE_Q_MASK,
            BLACK_KING_START => C_BLACK_K_MASK | C_BLACK_Q_MASK,
            _ => 0
        }
    }
}

impl fmt::Display for SQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&self.to_string())
    }
}

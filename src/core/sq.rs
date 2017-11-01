
use super::bitboard::BitBoard;
use super::masks::*;
use super::*;


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

    #[inline(always)]
    pub fn is_okay(self) -> bool {
        self.0 < 64
    }

    #[inline(always)]
    pub fn to_bb(self) -> BitBoard {
        assert!(self.is_okay());
        BitBoard((1 as u64).wrapping_shl(self.0 as u32))
    }

    #[inline(always)]
    pub fn rank_bb(self) -> BitBoard {
        BitBoard(RANK_BB[self.rank_of_sq() as usize])
    }

    #[inline(always)]
    pub fn rank_of_sq(self) -> Rank {
        ALL_RANKS[(self.0 >> 3) as usize]
    }

    #[inline(always)]
    pub fn rank_idx_of_sq(self) -> u8 {
        (self.0 >> 3) as u8
    }

    #[inline(always)]
    pub fn file_bb(self) -> BitBoard {
        BitBoard(FILE_BB[self.file_of_sq() as usize])
    }

    #[inline(always)]
    pub fn file_of_sq(self) -> File {
        ALL_FILES[(self.0 & 0b0000_0111) as usize]
    }

    #[inline(always)]
    pub fn file_idx_of_sq(self) -> u8 {
        (self.0 & 0b0000_0111) as u8
    }

    #[inline]
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

    #[inline]
    pub fn make(file: File, rank: Rank) -> SQ {
        SQ(((rank as u8).wrapping_shl(3) + (file as u8)) as u8)
    }
}

impl SQ {

    pub const NO_SQ: SQ = NO_SQ;

    pub const A1: SQ = SQ(0b000000);
    pub const B1: SQ = SQ(0b000001);
    pub const C1: SQ = SQ(0b000010);
    pub const D1: SQ = SQ(0b000011);
    pub const E1: SQ = SQ(0b000100);
    pub const F1: SQ = SQ(0b000101);
    pub const G1: SQ = SQ(0b000110);
    pub const H1: SQ = SQ(0b000111);

    pub const A2: SQ = SQ(0b001000);
    pub const B2: SQ = SQ(0b001001);
    pub const C2: SQ = SQ(0b001010);
    pub const D2: SQ = SQ(0b001011);
    pub const E2: SQ = SQ(0b001100);
    pub const F2: SQ = SQ(0b001101);
    pub const G2: SQ = SQ(0b001110);
    pub const H2: SQ = SQ(0b001111);

    pub const A3: SQ = SQ(0b010000);
    pub const B3: SQ = SQ(0b010001);
    pub const C3: SQ = SQ(0b010010);
    pub const D3: SQ = SQ(0b010011);
    pub const E3: SQ = SQ(0b010100);
    pub const F3: SQ = SQ(0b010101);
    pub const G3: SQ = SQ(0b010110);
    pub const H3: SQ = SQ(0b010111);

    pub const A4: SQ = SQ(0b011000);
    pub const B4: SQ = SQ(0b011001);
    pub const C4: SQ = SQ(0b011010);
    pub const D4: SQ = SQ(0b011011);
    pub const E4: SQ = SQ(0b011100);
    pub const F4: SQ = SQ(0b011101);
    pub const G4: SQ = SQ(0b011110);
    pub const H4: SQ = SQ(0b011111);

    pub const A5: SQ = SQ(0b100000);
    pub const B5: SQ = SQ(0b100001);
    pub const C5: SQ = SQ(0b100010);
    pub const D5: SQ = SQ(0b100011);
    pub const E5: SQ = SQ(0b100100);
    pub const F5: SQ = SQ(0b100101);
    pub const G5: SQ = SQ(0b100110);
    pub const H5: SQ = SQ(0b100111);

    pub const A6: SQ = SQ(0b101000);
    pub const B6: SQ = SQ(0b101001);
    pub const C6: SQ = SQ(0b101010);
    pub const D6: SQ = SQ(0b101011);
    pub const E6: SQ = SQ(0b101100);
    pub const F6: SQ = SQ(0b101101);
    pub const G6: SQ = SQ(0b101110);
    pub const H6: SQ = SQ(0b101111);

    pub const A7: SQ = SQ(0b110000);
    pub const B7: SQ = SQ(0b110001);
    pub const C7: SQ = SQ(0b110010);
    pub const D7: SQ = SQ(0b110011);
    pub const E7: SQ = SQ(0b110100);
    pub const F7: SQ = SQ(0b110101);
    pub const G7: SQ = SQ(0b110110);
    pub const H7: SQ = SQ(0b110111);

    pub const A8: SQ = SQ(0b111000);
    pub const B8: SQ = SQ(0b111001);
    pub const C8: SQ = SQ(0b111010);
    pub const D8: SQ = SQ(0b111011);
    pub const E8: SQ = SQ(0b111100);
    pub const F8: SQ = SQ(0b111101);
    pub const G8: SQ = SQ(0b111110);
    pub const H8: SQ = SQ(0b111111);
}

impl fmt::Display for SQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&self.to_string())
    }
}

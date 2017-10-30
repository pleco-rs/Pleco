pub mod bit_twiddles;
pub mod piece_move;
pub mod templates;
pub mod magic_helper;
pub mod masks;
pub mod bit_macros;

use self::masks::*;
use self::templates::*;

use std::ops::*;
//
//#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
//pub struct Bitboard(pub u64);
//
//impl Bitboard {
//    #[inline]
//    pub fn bb_to_sq(self) -> Sq {
//        debug_assert_eq!(popcount64(b), 1);
//        bit_scan_forward(self) as Sq
//    }
//}
//
//#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
//pub struct Sq(pub u8);
//
//impl From<Bitboard> for u64 {
//    #[inline(always)]
//    fn from(bb: Bitboard) -> Self { bb.0 }
//}
//
//impl From<u64> for Bitboard {
//    #[inline(always)]
//    fn from(bits: u64) -> Self { Bitboard(bits) }
//}
//impl Sq {
//    #[inline]
//    pub fn parse_sq(self) -> String {
//        assert!(self.is_okay());
//        let mut str = String::default();
//        str.push(FILE_DISPLAYS[file_of_sq(self) as usize]);
//        str.push(RANK_DISPLAYS[rank_of_sq(self) as usize]);
//        str
//    }
//
//    #[inline]
//    pub fn is_okay(self) -> bool {
//        self < 64
//    }
//
//    #[inline]
//    pub fn sq_to_bb(self) -> Bitboard {
//        assert!(self.is_okay());
//        (1 as u64).wrapping_shl(s as u32)
//    }
//
//    #[inline]
//    pub fn rank_bb(self) -> BitBoard {
//        RANK_BB[self.rank_of_sq() as usize]
//    }
//
//    #[inline]
//    pub fn rank_of_sq(self) -> Rank {
//        ALL_RANKS[(self >> 3) as usize]
//    }
//
//    #[inline]
//    pub fn rank_idx_of_sq(self) -> u8 {
//        (self >> 3) as u8
//    }
//
//    #[inline]
//    pub fn file_bb(self) -> u64 {
//        FILE_BB[self.file_of_sq() as usize]
//    }
//
//    #[inline]
//    pub fn file_of_sq(self) -> File {
//        ALL_FILES[(self & 0b0000_0111) as usize]
//    }
//
//    #[inline]
//    pub fn file_idx_of_sq(self) -> u8 {
//        (self & 0b0000_0111) as u8
//    }
//
//    pub fn castle_rights_mask(self) -> u8 {
//        match self.0 {
//            ROOK_WHITE_KSIDE_START => C_WHITE_K_MASK,
//            ROOK_WHITE_QSIDE_START => C_WHITE_Q_MASK,
//            ROOK_BLACK_KSIDE_START => C_BLACK_K_MASK,
//            ROOK_BLACK_QSIDE_START => C_BLACK_Q_MASK,
//            WHITE_KING_START => C_WHITE_K_MASK | C_WHITE_Q_MASK,
//            BLACK_KING_START => C_BLACK_K_MASK | C_BLACK_Q_MASK,
//            _ => 0
//        }
//    }
//}

//impl From<Sq> for u8 {
//    #[inline(always)]
//    fn from(sq: Sq) -> Self { bb.0 }
//}
//
//
//impl From<u8> for Sq {
//    #[inline(always)]
//    fn from(bits: u8) -> Self { Sq(bits) }
//}

//impl BitAnd for Sq {
//    type Output = Self;
//
//    // rhs is the "right-hand side" of the expression `a & b`
//    fn bitand(self, rhs: Self) -> Self {
//        Sq(self.fro & rhs as u8)
//    }
//}
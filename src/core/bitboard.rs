
use super::sq::SQ;
use super::bit_twiddles::*;
use super::masks::*;

use std::mem;
use std::ops::*;
use std::fmt;

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
pub struct BitBoard(pub u64);

impl_bit_ops!(BitBoard, u64);

impl BitBoard {

    pub const FILE_A: BitBoard = BitBoard(0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001);
    pub const FILE_B: BitBoard = BitBoard(0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010);
    pub const FILE_C: BitBoard = BitBoard(0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100);
    pub const FILE_D: BitBoard = BitBoard(0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000);
    pub const FILE_E: BitBoard = BitBoard(0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000);
    pub const FILE_F: BitBoard = BitBoard(0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000);
    pub const FILE_G: BitBoard = BitBoard(0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000);
    pub const FILE_H: BitBoard = BitBoard(0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000);
    pub const RANK_1: BitBoard = BitBoard(0x0000_0000_0000_00FF);
    pub const RANK_2: BitBoard = BitBoard(0x0000_0000_0000_FF00);
    pub const RANK_3: BitBoard = BitBoard(0x0000_0000_00FF_0000);
    pub const RANK_4: BitBoard = BitBoard(0x0000_0000_FF00_0000);
    pub const RANK_5: BitBoard = BitBoard(0x0000_00FF_0000_0000);
    pub const RANK_6: BitBoard = BitBoard(0x0000_FF00_0000_0000);
    pub const RANK_7: BitBoard = BitBoard(0x00FF_0000_0000_0000);
    pub const RANK_8: BitBoard = BitBoard(0xFF00_0000_0000_0000);

    #[inline(always)]
    pub fn to_sq(self) -> SQ {
        debug_assert_eq!(self.count_bits(), 1);
        SQ(bit_scan_forward(self.0))
    }

    #[inline(always)]
    pub fn count_bits(self) -> u8 {
        popcount64(self.0)
    }

    #[inline(always)]
    pub fn bit_scan_forward(self) -> SQ {
        SQ(self.bit_scan_forward_u8())
    }

    #[inline(always)]
    pub fn bit_scan_forward_u8(self) -> u8 {
        assert!(self.is_not_empty());
        bit_scan_forward(self.0)
    }

    #[inline(always)]
    pub fn more_than_one(self) -> bool {
        more_than_one(self.0)
    }



    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn is_not_empty(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn lsb(self) -> BitBoard {
        BitBoard(self.lsb_u64())
    }

    #[inline(always)]
    pub fn lsb_u64(self) -> u64 {
        lsb(self.0)
    }

    pub fn start_bbs() -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
        [[
            BitBoard(START_W_PAWN),
            BitBoard(START_W_KNIGHT),
            BitBoard(START_W_BISHOP),
            BitBoard(START_W_ROOK),
            BitBoard(START_W_QUEEN),
            BitBoard(START_W_KING),
        ], [
            BitBoard(START_B_PAWN),
            BitBoard(START_B_KNIGHT),
            BitBoard(START_B_BISHOP),
            BitBoard(START_B_ROOK),
            BitBoard(START_B_QUEEN),
            BitBoard(START_B_KING),
        ], ]
    }

    #[inline(always)]
    pub fn clone_all_occ(bbs: &[[BitBoard; PIECE_CNT]; PLAYER_CNT], ) -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
        let new_bbs: [[BitBoard; PIECE_CNT]; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }

    #[inline(always)]
    pub fn clone_occ_bbs(bbs: &[BitBoard; PLAYER_CNT]) -> [BitBoard; PLAYER_CNT] {
        let new_bbs: [BitBoard; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }
}



impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = &string_u64(reverse_bytes(self.0));
        f.pad(s)
    }
}

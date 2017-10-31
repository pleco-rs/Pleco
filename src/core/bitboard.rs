
use super::sq::*;
use super::bit_twiddles::*;
use super::masks::*;
use super::templates::*;

use std::ops::*;

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl_bit_ops!(Bitboard, u64);

impl Bitboard {
    #[inline]
    pub fn bb_to_sq(self) -> Sq {
        debug_assert_eq!(popcount64(self.0), 1);
        Sq::from(bit_scan_forward(self.0))
    }
}

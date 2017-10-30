//use super::{Sq,Bitboard};
use std::ops::*;

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct Bitboard(pub u64);

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct Sq(pub u8);

macro_rules! impl_indv_shift_ops {
    ($t:ty, $tname:ident, $fname:ident, $ta_name:ident, $fa_name:ident) => (

        impl $tname<usize> for $t {
            type Output = Self;

            #[inline]
            fn $fname(self, rhs: usize) -> Self {
                Self::from((self.0).$fname(rhs))
            }
        }

        impl $ta_name<usize> for $t {

            #[inline]
            fn $fa_name(&mut self, rhs: usize) {
                *self = Self::from((self.0).$fname(rhs));
            }
        }
    )
}


macro_rules! impl_indv_bit_ops {
    ($t:ty, $tname:ident, $fname:ident, $ta_name:ident, $fa_name:ident) => (

        impl $tname for $t {
            type Output = Self;

            #[inline]
            fn $fname(self, rhs: $t) -> Self {
                Self::from((self.0).$fname(rhs.0))
            }
        }

        impl $ta_name for $t {

            #[inline]
            fn $fa_name(&mut self, rhs: $t) {
                *self = Self::from((self.0).$fname(rhs.0));
            }
        }
    )
}

//Todo: bits! macro


macro_rules! impl_bit_ops {
    ($t:tt, $b:tt) => (
        impl From<$b> for $t {
            fn from(bit_type: $b) -> Self {
                $t(bit_type)
            }
        }

        impl From<$t> for $b {
            fn from(it:$t) -> Self {
                it.0
            }
        }
        impl_indv_bit_ops!($t, Add, add, AddAssign, add_assign);
        impl_indv_bit_ops!($t, BitOr, bitor, BitOrAssign, bitor_assign);
        impl_indv_bit_ops!($t, BitAnd, bitand, BitAndAssign, bitand_assign);
        impl_indv_bit_ops!($t, BitXor, bitxor, BitXorAssign, bitxor_assign);
        impl_indv_bit_ops!($t, Div, div, DivAssign, div_assign);
        impl_indv_bit_ops!($t, Mul, mul, MulAssign, mul_assign);
        impl_indv_bit_ops!($t, Rem, rem, RemAssign, rem_assign);
        impl_indv_bit_ops!($t, Sub, sub, SubAssign, sub_assign);
        impl_indv_shift_ops!($t, Shl, shl, ShlAssign, shl_assign);
        impl_indv_shift_ops!($t, Shr, shr, ShrAssign, shr_assign);

        impl Not for $t {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                Self::from((self.0).not())
            }
        }
    )
}

impl_bit_ops!(Sq, u8);
impl_bit_ops!(Bitboard, u64);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn testme() {
        let x = Sq::from(0b1110);
        let y = Sq::from(0b0011);
        let mut z = x & y;
        z &= y;

        let mut c = Sq::from(0b1010) | x;
        c |= y;

    }

}
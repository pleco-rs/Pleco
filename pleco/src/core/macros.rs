//! Macros for easily implementing bit operations, shifting operations, math operations,
//! and the `From` trait to a struct.

/// Allows for shifting operations to be applied to a struct consisting of a singular tuple
/// containing a type that implements that bit operation.
macro_rules! impl_indv_shift_ops {
    ($t:ty, $tname:ident, $fname:ident, $w:ident, $ta_name:ident, $fa_name:ident) => {
        impl $tname<usize> for $t {
            type Output = $t;

            #[inline]
            fn $fname(self, rhs: usize) -> $t {
                Self::from((self.0).$w(rhs as u32))
            }
        }

        impl $ta_name<usize> for $t {
            #[inline]
            fn $fa_name(&mut self, rhs: usize) {
                *self = Self::from((self.0).$w(rhs as u32));
            }
        }
    };
}

/// Allows for bit operations to be applied to a struct consisting of a singular tuple
/// containing a type that implements that bit operation.
macro_rules! impl_indv_bit_ops {
    ($t:ty, $b:ty, $tname:ident, $fname:ident, $w:ident, $ta_name:ident, $fa_name:ident) => {
        impl $tname for $t {
            type Output = $t;

            #[inline]
            fn $fname(self, rhs: $t) -> $t {
                Self::from((self.0).$w(rhs.0))
            }
        }

        impl $ta_name for $t {
            #[inline]
            fn $fa_name(&mut self, rhs: $t) {
                *self = Self::from((self.0).$w(rhs.0));
            }
        }

        impl $tname<$b> for $t {
            type Output = $t;

            #[inline]
            fn $fname(self, rhs: $b) -> $t {
                Self::from((self.0).$w(rhs))
            }
        }

        impl $ta_name<$b> for $t {
            #[inline]
            fn $fa_name(&mut self, rhs: $b) {
                *self = Self::from((self.0).$w(rhs));
            }
        }
    };
}

/// Implies bit operations `&, |, ^, !`, shifting operations `<< >>`,
/// math operations `+, -, *, /, %` and `From` trait to a struct consisting of a
/// singular tuple. This tuple must contain a type that implements these bit operations.
macro_rules! impl_bit_ops {
    ($t:tt, $b:tt) => {
        impl From<$b> for $t {
            fn from(bit_type: $b) -> Self {
                $t(bit_type)
            }
        }

        impl From<$t> for $b {
            fn from(it: $t) -> Self {
                it.0
            }
        }

        impl_indv_bit_ops!($t, $b, Rem, rem, rem, RemAssign, rem_assign);
        impl_indv_bit_ops!($t, $b, BitOr, bitor, bitor, BitOrAssign, bitor_assign);
        impl_indv_bit_ops!($t, $b, BitAnd, bitand, bitand, BitAndAssign, bitand_assign);
        impl_indv_bit_ops!($t, $b, BitXor, bitxor, bitxor, BitXorAssign, bitxor_assign);

        impl_indv_bit_ops!($t, $b, Add, add, wrapping_add, AddAssign, add_assign);
        impl_indv_bit_ops!($t, $b, Div, div, wrapping_div, DivAssign, div_assign);
        impl_indv_bit_ops!($t, $b, Mul, mul, wrapping_mul, MulAssign, mul_assign);
        impl_indv_bit_ops!($t, $b, Sub, sub, wrapping_sub, SubAssign, sub_assign);

        impl_indv_shift_ops!($t, Shl, shl, wrapping_shl, ShlAssign, shl_assign);
        impl_indv_shift_ops!($t, Shr, shr, wrapping_shr, ShrAssign, shr_assign);

        impl Not for $t {
            type Output = $t;

            #[inline]
            fn not(self) -> $t {
                $t(!self.0)
            }
        }
    };
}

#[cfg(test)]
mod tests {

    use std::ops::*;

    macro_rules! test_bit_ops_impls {
        ($t:tt, $int_t:ty, $fi:expr, $si:expr, $opp:tt) => ({
            let c_a = $fi $opp $si;
            let i_fo = $t::from($fi);
            let i_so = $t::from($si);
            let c = i_fo $opp i_so;
            assert_eq!(c.0, c_a);
        });
    }

    macro_rules! test_math_impls {
        ($t:tt, $int_t:ty, $fi:expr, $si:expr, $opp:tt, $w_opp:tt) => ({
            let c_a = $fi.$w_opp($si);
            let i_fo = $t::from($fi);
            let i_so = $t::from($si);
            let c = i_fo $opp i_so;
            assert_eq!(c.0, c_a);
        });
    }

    macro_rules! test_bit_shift_impls {
        ($t:tt, $int_t:ty, $fi:expr, $si:expr, $opp:tt, $w_opp:tt) => ({
            let c_a = $fi.$w_opp($si as u32);
            let i_fo = $t::from($fi);
            let c = i_fo $opp $si as usize;
            assert_eq!(c.0, c_a);
        });
    }

    #[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
    struct DummyBB(pub u64);

    #[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
    struct DummySQ(pub u8);

    impl_bit_ops!(DummySQ, u8);
    impl_bit_ops!(DummyBB, u64);

    const SQ_CONSTS: [u8; 18] = [
        0xFE, 0xC1, 0x21, 0x9F, 0x44, 0xA0, 0xF7, 0xFF, 0x11, 0x7A, 0x01, 0x02, 0x03, 0x04, 0x05,
        0x06, 0x07, 0x08,
    ];

    const BIT_CONSTS: [u64; 18] = [
        0xFE00C4D0,
        0x12F450012,
        0xFFFFFFFF,
        0x00000001,
        0xA0E34001,
        0x9ABBC0AA,
        0x412CBFFF,
        0x90000C10,
        0xC200C4D0,
        0xFE00C4D0,
        0xFE00C4D0,
        0x44FF2221,
        0x772C0F64,
        0x09F3C833,
        0x04444A09,
        0x3333FFEE,
        0x670FA111,
        0x7BBBB005,
    ];

    #[test]
    pub fn macro_imlps_sq() {
        for bits in SQ_CONSTS.iter() {
            assert_eq!((!DummySQ::from(*bits)).0, !(*bits));
            for bits_2 in SQ_CONSTS.iter() {
                test_bit_ops_impls!(DummySQ, u8, *bits, *bits_2, % );
                test_bit_ops_impls!(DummySQ, u8, *bits, *bits_2, ^ );
                test_bit_ops_impls!(DummySQ, u8, *bits, *bits_2, | );
                test_bit_ops_impls!(DummySQ, u8, *bits, *bits_2, & );
                test_math_impls!(DummySQ, u8, *bits, *bits_2, + , wrapping_add);
                test_math_impls!(DummySQ, u8, *bits, *bits_2, * , wrapping_mul);
                test_math_impls!(DummySQ, u8, *bits, *bits_2, - , wrapping_sub);
                test_math_impls!(DummySQ, u8, *bits, *bits_2, / , wrapping_div);
                test_bit_shift_impls!(DummySQ, u8, *bits, *bits_2, << , wrapping_shl);
                test_bit_shift_impls!(DummySQ, u8, *bits, *bits_2, >> , wrapping_shr);
            }
        }
    }

    #[test]
    pub fn macro_imlps_bb() {
        for bits in BIT_CONSTS.iter() {
            assert_eq!((!DummyBB::from(*bits)).0, !(*bits));
            for bits_2 in BIT_CONSTS.iter() {
                test_bit_ops_impls!(DummyBB, u8, *bits, *bits_2, % );
                test_bit_ops_impls!(DummyBB, u8, *bits, *bits_2, ^ );
                test_bit_ops_impls!(DummyBB, u8, *bits, *bits_2, | );
                test_bit_ops_impls!(DummyBB, u8, *bits, *bits_2, & );
                test_math_impls!(DummyBB, u8, *bits, *bits_2, + , wrapping_add);
                test_math_impls!(DummyBB, u8, *bits, *bits_2, * , wrapping_mul);
                test_math_impls!(DummyBB, u8, *bits, *bits_2, - , wrapping_sub);
                test_math_impls!(DummyBB, u8, *bits, *bits_2, / , wrapping_div);
            }

            for x in 0..67usize {
                test_bit_shift_impls!(DummyBB, u8, *bits, x, << , wrapping_shl);
                test_bit_shift_impls!(DummyBB, u8, *bits, x, >> , wrapping_shr);
            }
        }
    }
}

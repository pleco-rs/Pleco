//! [bit_twiddles] is the a collection of useful functions oreinted around modifying
//! singular bits of integer types.

static POPCNT8: &'static [u8] = &[
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8
];

static DEBRUIJ_T: &'static [u8] = &[
    0, 47,  1, 56, 48, 27,  2, 60,
    57, 49, 41, 37, 28, 16,  3, 61,
    54, 58, 35, 52, 50, 42, 21, 44,
    38, 32, 29, 23, 17, 11,  4, 62,
    46, 55, 26, 59, 40, 36, 15, 53,
    34, 51, 20, 43, 31, 22, 10, 45,
    25, 39, 14, 33, 19, 30,  9, 24,
    13, 18,  8, 12,  7,  6,  5, 63
];

const DEBRUIJ_M: u64 = 0x03f7_9d71_b4cb_0a89;

// BitScanForward: Djuin:            9 s
// BitScanForward: PopCount - Old : 18 s
// BitScanForward: PopCount - Rust: 26 s

// PopCount: Rust:  22 s
// PopCount: Old :  37 s


/// Counts the number of bits
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert_eq!(popcount64(0b1001), 2);
/// ```
#[inline(always)]
pub fn popcount64(x: u64) -> u8 {
    x.count_ones() as u8
}

/// Returns index of the Least Significant Bit
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert_eq!(bit_scan_forward(0b10100),2)
/// ```
///
#[inline(always)]
pub fn bit_scan_forward(bits: u64) -> u8 {
    assert_ne!(bits, 0);
    DEBRUIJ_T[(((bits ^ bits.wrapping_sub(1)).wrapping_mul(DEBRUIJ_M)).wrapping_shr(58)) as usize]
}

/// Returns index of the Least Significant Bit
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert_eq!(bit_scan_forward(0b100),2);
/// ```
///
#[inline(always)]
pub fn bit_scan_forward_rust_trailing(bits: u64) -> u8 {
    assert_ne!(bits, 0);
    bits.trailing_zeros() as u8
}

/// Returns index of the Most Significant Bit
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert_eq!(bit_scan_reverse(0b101),2);
/// ```
///
#[inline(always)]
pub fn bit_scan_reverse(mut bb: u64) -> u8 {
    assert_ne!(bb, 0);
    bb |= bb >> 1;
    bb |= bb >> 2;
    bb |= bb >> 4;
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    DEBRUIJ_T[(bb.wrapping_mul(DEBRUIJ_M)).wrapping_shr(58) as usize]
}

/// Returns if there are more than one bits in a u64
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert!(more_than_one(0b1111));
///
/// assert!(!more_than_one(0b0001))
///
/// ```
#[inline(always)]
pub fn more_than_one(x: u64) -> bool {
    (x & (x.wrapping_sub(1))) != 0
}


/// Returns the least significant bit
///
/// # Examples
///
/// ```
/// use pleco::bit_twiddles::*;
///
/// assert_eq!(lsb(0b1001), 0b0001);
/// ```
#[inline(always)]
pub fn lsb(bits: u64) -> u64 {
    (1 as u64).wrapping_shl(bits.trailing_zeros())
}

/// Counts the number of bits
#[inline(always)]
fn popcount_old(x: u64) -> u8 {
    let x = x as usize;
    if x == 0 {
        return 0;
    }
    if x & (x.wrapping_sub(1)) == 0 {
        return 1;
    }
    POPCNT8[x >> 56] + POPCNT8[(x >> 48) & 0xFF] + POPCNT8[(x >> 40) & 0xFF] +
        POPCNT8[(x >> 32) & 0xFF] + POPCNT8[(x >> 24) & 0xFF] + POPCNT8[(x >> 16) & 0xFF] +
        POPCNT8[(x >> 8) & 0xFF] + POPCNT8[x & 0xFF]
}

#[cfg(test)]
mod tests {

    use bit_twiddles;

    #[test]
    fn test_bit_scan() {
        assert_eq!(bit_twiddles::bit_scan_forward(2), 1);
        assert_eq!(bit_twiddles::bit_scan_forward(4), 2);
        assert_eq!(bit_twiddles::bit_scan_forward(8), 3);
        assert_eq!(bit_twiddles::bit_scan_forward(16), 4);
        assert_eq!(bit_twiddles::bit_scan_forward(32), 5);
        assert_eq!(bit_twiddles::bit_scan_forward(31), 0);
        assert_eq!(bit_twiddles::bit_scan_forward(0b000000000000001), 0);
        assert_eq!(bit_twiddles::bit_scan_forward(0b000000000000010), 1);
        assert_eq!(bit_twiddles::bit_scan_forward(0b110011100000010), 1);
        assert_eq!(bit_twiddles::bit_scan_forward(0b110011100000010), 1);
    }

    #[test]
    fn popcount() {
        assert_eq!(bit_twiddles::popcount64(0b000000000000000), 0);
        assert_eq!(bit_twiddles::popcount64(0b11111100000001), 7);
        assert_eq!(bit_twiddles::popcount64(0b1000010000), 2);
        assert_eq!(bit_twiddles::popcount64(0xFFFFFFFF), 32);
        assert_eq!(bit_twiddles::popcount64(0x55555555), 16);
    }

    #[test]
    fn lsb() {
        assert_eq!(bit_twiddles::lsb(0b110011100000010), 0b10);
        assert_eq!(bit_twiddles::lsb(0b1010000000000000), 0b10000000000000);
        assert_eq!(bit_twiddles::lsb(0b11001110000), 0b10000);
        assert_eq!(bit_twiddles::lsb(0b100001000000), 0b1000000);
        assert_eq!(bit_twiddles::lsb(0b1), 0b1);
    }

}
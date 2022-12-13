//! Collection of useful functions oriented around modifying singular bits of integer types.
//! You will rarely need to interact with this module directly unless you need functions
//! involving the manipulation of bits.

static POPCNT8: &[u8] = &[
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
];

static DEBRUIJ_T: &[u8] = &[
    0, 47, 1, 56, 48, 27, 2, 60, 57, 49, 41, 37, 28, 16, 3, 61, 54, 58, 35, 52, 50, 42, 21, 44, 38,
    32, 29, 23, 17, 11, 4, 62, 46, 55, 26, 59, 40, 36, 15, 53, 34, 51, 20, 43, 31, 22, 10, 45, 25,
    39, 14, 33, 19, 30, 9, 24, 13, 18, 8, 12, 7, 6, 5, 63,
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
/// use pleco::core::bit_twiddles::*;
///
/// assert_eq!(popcount64(0b1001), 2);
/// ```
#[inline(always)]
pub fn popcount64(x: u64) -> u8 {
    popcount_rust(x)
}

/// Returns index of the Least Significant Bit
///
/// # Examples
///
/// ```
/// use pleco::core::bit_twiddles::*;
///
/// assert_eq!(bit_scan_forward(0b10100),2);
/// ```
///
#[inline(always)]
pub fn bit_scan_forward(bits: u64) -> u8 {
    assert_ne!(bits, 0);
    unsafe {
        *DEBRUIJ_T.get_unchecked(
            (((bits ^ bits.wrapping_sub(1)).wrapping_mul(DEBRUIJ_M)).wrapping_shr(58)) as usize,
        )
    }
}

/// Returns index of the Least Significant Bit
///
/// # Examples
///
/// ```
/// use pleco::core::bit_twiddles::*;
///
/// assert_eq!(bit_scan_forward_rust_trailing(0b100),2);
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
/// use pleco::core::bit_twiddles::*;
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
    unsafe { *DEBRUIJ_T.get_unchecked((bb.wrapping_mul(DEBRUIJ_M)).wrapping_shr(58) as usize) }
}

/// Returns if there are more than one bits in a u64.
///
/// # Examples
///
/// ```
/// use pleco::core::bit_twiddles::*;
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
/// use pleco::core::bit_twiddles::*;
///
/// assert_eq!(lsb(0b1001), 0b0001);
/// ```
#[inline(always)]
pub fn lsb(bits: u64) -> u64 {
    1_u64.wrapping_shl(bits.trailing_zeros())
}

/// Counts the number of bits in a u64.
#[inline]
pub fn popcount_table(x: u64) -> u8 {
    let x = x as usize;
    if x == 0 {
        return 0;
    }
    if x & (x.wrapping_sub(1)) == 0 {
        return 1;
    }

    POPCNT8[x >> 56]
        + POPCNT8[(x >> 48) & 0xFF]
        + POPCNT8[(x >> 40) & 0xFF]
        + POPCNT8[(x >> 32) & 0xFF]
        + POPCNT8[(x >> 24) & 0xFF]
        + POPCNT8[(x >> 16) & 0xFF]
        + POPCNT8[(x >> 8) & 0xFF]
        + POPCNT8[x & 0xFF]
}

/// Counts the number of bits in a u64.
#[inline(always)]
pub fn popcount_rust(x: u64) -> u8 {
    x.count_ones() as u8
}

/// Returns the positive difference between two unsigned u8s.
#[inline(always)]
pub fn diff(x: u8, y: u8) -> u8 {
    if x < y {
        y - x
    } else {
        x - y
    }
}

/// Gives the most significant bit of a `u64`.
pub fn msb(x: u64) -> u64 {
    1_u64.wrapping_shl(63 - x.leading_zeros())
}

/// Reverses all the bytes in a u64.
///
/// # Examples
///
/// ```
/// use pleco::core::bit_twiddles::*;
///
/// let x: u64 =  0b11001100_00000001;
/// let reverse = 0b00110011_10000000;
/// assert_eq!(reverse, reverse_bytes(x));
/// ```
pub fn reverse_bytes(b: u64) -> u64 {
    let mut m: u64 = 0;
    m |= (reverse_byte(((b >> 56) & 0xFF) as u8) as u64) << 56;
    m |= (reverse_byte(((b >> 48) & 0xFF) as u8) as u64) << 48;
    m |= (reverse_byte(((b >> 40) & 0xFF) as u8) as u64) << 40;
    m |= (reverse_byte(((b >> 32) & 0xFF) as u8) as u64) << 32;
    m |= (reverse_byte(((b >> 24) & 0xFF) as u8) as u64) << 24;
    m |= (reverse_byte(((b >> 16) & 0xFF) as u8) as u64) << 16;
    m |= (reverse_byte(((b >> 8) & 0xFF) as u8) as u64) << 8;
    m |= reverse_byte((b & 0xFF) as u8) as u64;
    m
}

/// Reverses all a byte.
///
/// # Examples
///
/// ```
/// use pleco::core::bit_twiddles::*;
///
/// let x: u8 =  0b00000001;
/// let reverse = 0b10000000;
/// assert_eq!(reverse, reverse_byte(x));
/// ```
#[inline]
pub fn reverse_byte(b: u8) -> u8 {
    let m: u8 = ((0b0000_0001 & b) << 7)
        | ((0b0000_0010 & b) << 5)
        | ((0b0000_0100 & b) << 3)
        | ((0b0000_1000 & b) << 1)
        | ((0b0001_0000 & b) >> 1)
        | ((0b0010_0000 & b) >> 3)
        | ((0b0100_0000 & b) >> 5)
        | ((0b1000_0000 & b) >> 7);
    m
}

/// Returns a String of the given u64, formatted in the order of where each bit maps to
/// a specific square.
pub fn string_u64(input: u64) -> String {
    let mut s = String::new();
    let format_in = format_u64(input);
    for x in 0..8 {
        let slice = &format_in[x * 8..((x * 8) + 8)];
        s += slice;
        s += "\n";
    }
    s
}

/// Returns a stringified u64 with all 64 bits being represented.
fn format_u64(input: u64) -> String {
    format!("{:064b}", input)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bit_scan() {
        assert_eq!(bit_scan_forward(2), 1);
        assert_eq!(bit_scan_forward(4), 2);
        assert_eq!(bit_scan_forward(8), 3);
        assert_eq!(bit_scan_forward(16), 4);
        assert_eq!(bit_scan_forward(32), 5);
        assert_eq!(bit_scan_forward(31), 0);
        assert_eq!(bit_scan_forward(0b000000000000001), 0);
        assert_eq!(bit_scan_forward(0b000000000000010), 1);
        assert_eq!(bit_scan_forward(0b110011100000010), 1);
        assert_eq!(bit_scan_forward(0b110011100000010), 1);
    }

    #[test]
    fn msb_t() {
        assert_eq!(msb(0b0011), 0b0010);
    }

    #[test]
    fn popcount_t() {
        assert_eq!(popcount64(0b000000000000000), 0);
        assert_eq!(popcount64(0b11111100000001), 7);
        assert_eq!(popcount64(0b1000010000), 2);
        assert_eq!(popcount64(0xFFFFFFFF), 32);
        assert_eq!(popcount64(0x55555555), 16);
    }

    #[test]
    fn lsb_t() {
        assert_eq!(lsb(0b110011100000010), 0b10);
        assert_eq!(lsb(0b1010000000000000), 0b10000000000000);
        assert_eq!(lsb(0b11001110000), 0b10000);
        assert_eq!(lsb(0b100001000000), 0b1000000);
        assert_eq!(lsb(0b1), 0b1);
    }
}

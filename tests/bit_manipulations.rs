extern crate rusty_chess;

use rusty_chess::bit_twiddles as bit_twiddles;


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
    assert_eq!(bit_twiddles::pop_count(0b000000000000000), 0);
    assert_eq!(bit_twiddles::pop_count(0b11111100000001), 7);
    assert_eq!(bit_twiddles::pop_count(0b1000010000), 2);
    assert_eq!(bit_twiddles::pop_count(0xFFFFFFFF), 32);
    assert_eq!(bit_twiddles::pop_count(0x55555555), 16);
}

fn lsb() {
    assert_eq!(bit_twiddles::lsb(0b110011100000010), 0b10);
    assert_eq!(bit_twiddles::lsb(0b1010000000000000), 0b10000000000000);
    assert_eq!(bit_twiddles::lsb(0b11001110000), 0b10000);
    assert_eq!(bit_twiddles::lsb(0b100001000000), 0b1000000);
    assert_eq!(bit_twiddles::lsb(0b1), 0b1);
}
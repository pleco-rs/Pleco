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
}


#[derive(Copy, Clone)]
pub enum Player {
    White = 0,
    Black = 1,
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum SQ {
    A1 = 0, B1 = 1, C1 = 2, D1 = 3, E1 = 4, F1 = 5, G1 = 6, H1 = 7,
    A2 = 8, B2 = 9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
}

pub fn to_SQ(num: u8) -> SQ {
    match num {
        0  => SQ::A1,
        1  => SQ::A2,
        2  => SQ::A3,
        3  => SQ::A4,
        4  => SQ::A5,
        5  => SQ::A6,
        6  => SQ::A7,
        7  => SQ::A8,
        8  => SQ::B1,
        9  => SQ::B2,
        10 => SQ::B3,
        11 => SQ::B4,
        12 => SQ::B5,
        13 => SQ::B6,
        14 => SQ::B7,
        15 => SQ::B8,
        16 => SQ::C1,
        17 => SQ::C2,
        18 => SQ::C3,
        19 => SQ::C4,
        20 => SQ::C5,
        21 => SQ::C6,
        22 => SQ::C7,
        23 => SQ::C8,
        24 => SQ::D1,
        25 => SQ::D2,
        26 => SQ::D3,
        27 => SQ::D4,
        28 => SQ::D5,
        29 => SQ::D6,
        30 => SQ::D7,
        31 => SQ::D8,
        32 => SQ::E1,
        33 => SQ::E2,
        34 => SQ::E3,
        35 => SQ::E4,
        36 => SQ::E5,
        37 => SQ::E6,
        38 => SQ::E7,
        39 => SQ::E8,
        40 => SQ::F1,
        41 => SQ::F2,
        42 => SQ::F3,
        43 => SQ::F4,
        44 => SQ::F5,
        45 => SQ::F6,
        46 => SQ::F7,
        47 => SQ::F8,
        48 => SQ::G1,
        49 => SQ::G2,
        50 => SQ::G3,
        51 => SQ::G4,
        52 => SQ::G5,
        53 => SQ::G6,
        54 => SQ::G7,
        55 => SQ::G8,
        56 => SQ::H1,
        57 => SQ::H2,
        58 => SQ::H3,
        59 => SQ::H4,
        60 => SQ::H5,
        61 => SQ::H6,
        62 => SQ::H7,
        63 => SQ::H8,
        _ => SQ::A1
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Piece {
    K = 6,
    Q = 5,
    R = 4,
    B = 3,
    N = 2,
    P = 1,
}

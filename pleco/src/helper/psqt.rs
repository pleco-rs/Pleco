use core::masks::*;
use core::score::*;
use {File, Piece, Player, SQ};

const BONUS: [[[Score; (FILE_CNT / 2)]; RANK_CNT]; PIECE_TYPE_CNT] = [
    [
        // NO PIECE
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
    ],
    [
        // Pawn
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(-11, 7), Score(6, -4), Score(7, 8), Score(3, -2)],
        [Score(-18, -4), Score(-2, -5), Score(19, 5), Score(24, 4)],
        [Score(-17, 3), Score(-9, 3), Score(20, -8), Score(35, -3)],
        [Score(-6, 8), Score(5, 9), Score(3, 7), Score(21, -6)],
        [Score(-6, 8), Score(-8, -5), Score(-6, 2), Score(-2, 4)],
        [Score(-4, 3), Score(20, -9), Score(-8, 1), Score(-4, 18)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
    ],
    [
        // Knight
        [
            Score(-161, -105),
            Score(-96, -82),
            Score(-80, -46),
            Score(-73, -14),
        ],
        [
            Score(-83, -69),
            Score(-43, -54),
            Score(-21, -17),
            Score(-10, 9),
        ],
        [Score(-71, -50), Score(-22, -39), Score(0, -7), Score(9, 28)],
        [Score(-25, -41), Score(18, -25), Score(43, 6), Score(47, 38)],
        [Score(-26, -46), Score(16, -25), Score(38, 3), Score(50, 40)],
        [
            Score(-11, -54),
            Score(37, -38),
            Score(56, -7),
            Score(65, 27),
        ],
        [
            Score(-63, -65),
            Score(-19, -50),
            Score(5, -24),
            Score(14, 13),
        ],
        [
            Score(-195, -109),
            Score(-67, -89),
            Score(-42, -50),
            Score(-29, -13),
        ],
    ],
    [
        // Bishop
        [
            Score(-44, -58),
            Score(-13, -31),
            Score(-25, -37),
            Score(-34, -19),
        ],
        [Score(-20, -34), Score(20, -9), Score(12, -14), Score(1, 4)],
        [Score(-9, -23), Score(27, 0), Score(21, -3), Score(11, 16)],
        [Score(-11, -26), Score(28, -3), Score(21, -5), Score(10, 16)],
        [Score(-11, -26), Score(27, -4), Score(16, -7), Score(9, 14)],
        [Score(-17, -24), Score(16, -2), Score(12, 0), Score(2, 13)],
        [Score(-23, -34), Score(17, -10), Score(6, -12), Score(-2, 6)],
        [
            Score(-35, -55),
            Score(-11, -32),
            Score(-19, -36),
            Score(-29, -17),
        ],
    ],
    [
        // Rook
        [Score(-25, 0), Score(-16, 0), Score(-16, 0), Score(-9, 0)],
        [Score(-21, 0), Score(-8, 0), Score(-3, 0), Score(0, 0)],
        [Score(-21, 0), Score(-9, 0), Score(-4, 0), Score(2, 0)],
        [Score(-22, 0), Score(-6, 0), Score(-1, 0), Score(2, 0)],
        [Score(-22, 0), Score(-7, 0), Score(0, 0), Score(1, 0)],
        [Score(-21, 0), Score(-7, 0), Score(0, 0), Score(2, 0)],
        [Score(-12, 0), Score(4, 0), Score(8, 0), Score(12, 0)],
        [Score(-23, 0), Score(-15, 0), Score(-11, 0), Score(-5, 0)],
    ],
    [
        // Queen
        [
            Score(0, -71),
            Score(-4, -56),
            Score(-3, -42),
            Score(-1, -29),
        ],
        [Score(-4, -56), Score(6, -30), Score(9, -21), Score(8, -5)],
        [Score(-2, -39), Score(6, -17), Score(9, -8), Score(9, 5)],
        [Score(-1, -29), Score(8, -5), Score(10, 9), Score(7, 19)],
        [Score(-3, -27), Score(9, -5), Score(8, 10), Score(7, 21)],
        [Score(-2, -40), Score(6, -16), Score(8, -10), Score(10, 3)],
        [Score(-2, -55), Score(7, -30), Score(7, -21), Score(6, -6)],
        [
            Score(-1, -74),
            Score(-4, -55),
            Score(-1, -43),
            Score(0, -30),
        ],
    ],
    [
        // King
        [
            Score(267, 0),
            Score(320, 48),
            Score(270, 75),
            Score(195, 84),
        ],
        [
            Score(264, 43),
            Score(304, 92),
            Score(238, 143),
            Score(180, 132),
        ],
        [
            Score(200, 83),
            Score(245, 138),
            Score(176, 167),
            Score(110, 165),
        ],
        [
            Score(177, 106),
            Score(185, 169),
            Score(148, 169),
            Score(110, 179),
        ],
        [
            Score(149, 108),
            Score(177, 163),
            Score(115, 200),
            Score(66, 203),
        ],
        [
            Score(118, 95),
            Score(159, 155),
            Score(84, 176),
            Score(41, 174),
        ],
        [
            Score(87, 50),
            Score(128, 99),
            Score(63, 122),
            Score(20, 139),
        ],
        [Score(63, 9), Score(88, 55), Score(47, 80), Score(0, 90)],
    ],
    [
        // ALL PIECE
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
        [Score(0, 0), Score(0, 0), Score(0, 0), Score(0, 0)],
    ],
];

static mut PSQ: [[Score; SQ_CNT]; PIECE_CNT] = [[Score(0, 0); SQ_CNT]; PIECE_CNT];

static PIECE_VALUE: [[Value; PHASE_CNT]; PIECE_CNT] = [
    [0, 0],                 // Empty
    [PAWN_MG, PAWN_EG],     // White Pawn
    [KNIGHT_MG, KNIGHT_EG], // White Knight
    [BISHOP_MG, BISHOP_EG], // White Bishop
    [ROOK_MG, ROOK_EG],     // White Rook
    [QUEEN_MG, QUEEN_MG],   // White Queen
    [ZERO, ZERO],           // White King
    [0, 0],
    [0, 0],                 // Empty
    [PAWN_MG, PAWN_EG],     // Black Pawn
    [KNIGHT_MG, KNIGHT_EG], // Black Knight
    [BISHOP_MG, BISHOP_EG], // Black Bishop
    [ROOK_MG, ROOK_EG],     // Black Rook
    [QUEEN_MG, QUEEN_MG],   // Black Queen
    [ZERO, ZERO],           // Black King
    [0, 0],
];

#[cold]
pub fn init_psqt() {
    for piece in 0..PIECE_TYPE_CNT {
        let v: Score = Score(PIECE_VALUE[piece][0], PIECE_VALUE[piece][1]);
        for s in 0..SQ_CNT {
            let sq: SQ = SQ(s as u8);
            let f: File = sq.file().min(!sq.file());
            let score = v + BONUS[piece][sq.rank() as usize][f as usize];
            unsafe {
                PSQ[(Player::White as usize) << 3 | piece][s] = score;
                PSQ[(Player::Black as usize) << 3 | piece][sq.flip().0 as usize] = -score;
            }
        }
    }
}

/// Returns the score for a player's piece being at a particular square.
#[inline(always)]
pub fn psq(piece: Piece, sq: SQ) -> Score {
    debug_assert!(sq.is_okay());
    unsafe { *(PSQ.get_unchecked(piece as usize)).get_unchecked(sq.0 as usize) }
}

/// Returns the value of a piece for a player. If `eg` is true, it returns the end game value. Otherwise,
/// it'll return the midgame value.
#[inline(always)]
pub fn piece_value(piece: Piece, eg: bool) -> Value {
    unsafe { *(PIECE_VALUE.get_unchecked(piece as usize)).get_unchecked(eg as usize) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psq_tes() {
        init_psqt();
        assert_eq!(
            psq(Piece::WhiteQueen, SQ::A1),
            -psq(Piece::BlackQueen, SQ::A8)
        );
        assert_eq!(
            psq(Piece::WhiteRook, SQ::A1),
            -psq(Piece::BlackRook, SQ::A8)
        );
        assert_eq!(
            psq(Piece::WhitePawn, SQ::B1),
            -psq(Piece::BlackPawn, SQ::B8)
        );
        assert_eq!(
            psq(Piece::BlackKnight, SQ::B4),
            -psq(Piece::WhiteKnight, SQ::B5)
        );
    }
}

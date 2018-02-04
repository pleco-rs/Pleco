//! This module contains useful pre-computed lookup tables involving `BitBoard`s.
//!
//! A `MagicHelper` is computed at the beginning of execution using `lazy_static` inside
//! `board/mod.rs`.


#![cfg_attr(feature="clippy", allow(invalid_ref))]
#![cfg_attr(feature="clippy", allow(needless_range_loop))]

use super::bit_twiddles::*;
use super::masks::*;
use super::sq::SQ;
use super::bitboard::BitBoard;
use tools::prng::PRNG;
use std::{mem, slice};
use super::*;
use super::score::*;

/// Size of the magic rook table.
const ROOK_M_SIZE: usize = 102_400;
/// Size of the magic bishop table.
const BISHOP_M_SIZE: usize = 5248;

const B_DELTAS: [i8; 4] = [7, 9, -9, -7];
const R_DELTAS: [i8; 4] = [8, 1, -8, -1];
const DELTAS: [[i8; 4]; 2] = [B_DELTAS, R_DELTAS];

/// Seeds for the `MagicHelper's` pseudo-random number generator.
const SEEDS: [[u64; 8]; 2] = [
    [8977, 44_560, 54_343, 38_998, 5731, 95_205, 104_912, 17_020],
    [728, 10_316, 55_013, 32_803, 12_281, 15_100, 16_645, 255],
];

/// Seed for the Zobrist's pseudo-random number generator.
const ZOBRIST_SEED: u64 = 23_081;



const BONUS: [[[Score; (FILE_CNT / 2)]; RANK_CNT]; PIECE_TYPE_CNT] = [
    [ // Pawn
        [ Score(  0, 0), Score(  0, 0), Score(  0, 0), Score( 0, 0) ],
        [ Score(-11, 7), Score(  6,-4), Score(  7, 8), Score( 3,-2) ],
        [ Score(-18,-4), Score( -2,-5), Score( 19, 5), Score(24, 4) ],
        [ Score(-17, 3), Score( -9, 3), Score( 20,-8), Score(35,-3) ],
        [ Score( -6, 8), Score(  5, 9), Score(  3, 7), Score(21,-6) ],
        [ Score( -6, 8), Score( -8,-5), Score( -6, 2), Score(-2, 4) ],
        [ Score( -4, 3), Score( 20,-9), Score( -8, 1), Score(-4,18) ],
        [ Score(  0, 0), Score(  0, 0), Score(  0, 0), Score( 0, 0) ]
    ],
    [ // Knight
        [ Score(-161,-105), Score(-96,-82), Score(-80,-46), Score(-73,-14) ],
        [ Score( -83, -69), Score(-43,-54), Score(-21,-17), Score(-10,  9) ],
        [ Score( -71, -50), Score(-22,-39), Score(  0, -7), Score(  9, 28) ],
        [ Score( -25, -41), Score( 18,-25), Score( 43,  6), Score( 47, 38) ],
        [ Score( -26, -46), Score( 16,-25), Score( 38,  3), Score( 50, 40) ],
        [ Score( -11, -54), Score( 37,-38), Score( 56, -7), Score( 65, 27) ],
        [ Score( -63, -65), Score(-19,-50), Score(  5,-24), Score( 14, 13) ],
        [ Score(-195,-109), Score(-67,-89), Score(-42,-50), Score(-29,-13) ]
    ],
    [ // Bishop
        [ Score(-44,-58), Score(-13,-31), Score(-25,-37), Score(-34,-19) ],
        [ Score(-20,-34), Score( 20, -9), Score( 12,-14), Score(  1,  4) ],
        [ Score( -9,-23), Score( 27,  0), Score( 21, -3), Score( 11, 16) ],
        [ Score(-11,-26), Score( 28, -3), Score( 21, -5), Score( 10, 16) ],
        [ Score(-11,-26), Score( 27, -4), Score( 16, -7), Score(  9, 14) ],
        [ Score(-17,-24), Score( 16, -2), Score( 12,  0), Score(  2, 13) ],
        [ Score(-23,-34), Score( 17,-10), Score(  6,-12), Score( -2,  6) ],
        [ Score(-35,-55), Score(-11,-32), Score(-19,-36), Score(-29,-17) ]
    ],
        [ // Rook
        [ Score(-25, 0), Score(-16, 0), Score(-16, 0), Score(-9, 0) ],
        [ Score(-21, 0), Score( -8, 0), Score( -3, 0), Score( 0, 0) ],
        [ Score(-21, 0), Score( -9, 0), Score( -4, 0), Score( 2, 0) ],
        [ Score(-22, 0), Score( -6, 0), Score( -1, 0), Score( 2, 0) ],
        [ Score(-22, 0), Score( -7, 0), Score(  0, 0), Score( 1, 0) ],
        [ Score(-21, 0), Score( -7, 0), Score(  0, 0), Score( 2, 0) ],
        [ Score(-12, 0), Score(  4, 0), Score(  8, 0), Score(12, 0) ],
        [ Score(-23, 0), Score(-15, 0), Score(-11, 0), Score(-5, 0) ]
    ],
    [ // Queen
        [ Score( 0,-71), Score(-4,-56), Score(-3,-42), Score(-1,-29) ],
        [ Score(-4,-56), Score( 6,-30), Score( 9,-21), Score( 8, -5) ],
        [ Score(-2,-39), Score( 6,-17), Score( 9, -8), Score( 9,  5) ],
        [ Score(-1,-29), Score( 8, -5), Score(10,  9), Score( 7, 19) ],
        [ Score(-3,-27), Score( 9, -5), Score( 8, 10), Score( 7, 21) ],
        [ Score(-2,-40), Score( 6,-16), Score( 8,-10), Score(10,  3) ],
        [ Score(-2,-55), Score( 7,-30), Score( 7,-21), Score( 6, -6) ],
        [ Score(-1,-74), Score(-4,-55), Score(-1,-43), Score( 0,-30) ]
    ],
    [ // King
        [ Score(267,  0), Score(320, 48), Score(270, 75), Score(195, 84) ],
        [ Score(264, 43), Score(304, 92), Score(238,143), Score(180,132) ],
        [ Score(200, 83), Score(245,138), Score(176,167), Score(110,165) ],
        [ Score(177,106), Score(185,169), Score(148,169), Score(110,179) ],
        [ Score(149,108), Score(177,163), Score(115,200), Score( 66,203) ],
        [ Score(118, 95), Score(159,155), Score( 84,176), Score( 41,174) ],
        [ Score( 87, 50), Score(128, 99), Score( 63,122), Score( 20,139) ],
        [ Score( 63,  9), Score( 88, 55), Score( 47, 80), Score(  0, 90) ]
    ]
];



pub struct PSQT {
    psq: [[[Score; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT],  // [player][piece][sq]
    piece_val: [[[Value; PHASE_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT], // [player][piece][eg value?]
}

impl PSQT {
    pub fn init() -> Self {
        let mut psq: [[[Score; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
            = [[[Score::new(0,0); SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT];

        let piece_val: [[[Value; PHASE_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
            = [[[ PAWN_MG,    PAWN_EG],  // White Pawn
                [ KNIGHT_MG,  KNIGHT_EG],// White Knight
                [ BISHOP_MG,  BISHOP_EG],// White Bishop
                [ ROOK_MG,    ROOK_EG],  // White Rook
                [ QUEEN_MG,   QUEEN_MG], // White Queen
                [ NONE,       NONE]],    // White King
               [[-PAWN_MG,   -PAWN_EG],  // Black Pawn
                [-KNIGHT_MG, -KNIGHT_EG],// Black Knight
                [-BISHOP_MG, -BISHOP_EG],// Black Bishop
                [-ROOK_MG,   -ROOK_EG],  // Black Rook
                [-QUEEN_MG,  -QUEEN_MG], // Black Queen
                [-NONE,      -NONE]]     // Black King
        ];

        for piece in 0..PIECE_TYPE_CNT {
            let v: Score = Score(piece_val[0][piece][0], piece_val[0][piece][1]);
            for s in 0..SQ_CNT {
                let sq: SQ = SQ(s as u8);
                let f: File = sq.file().min(!sq.file());
                let score = v + BONUS[piece][sq.rank() as usize][f as usize];
                psq[Player::White as usize][piece][s] = score;
                psq[Player::Black as usize][piece][sq.flip().0 as usize] = -score;
            }
        }

        PSQT {
            psq,
            piece_val
        }
    }
}

/// Structure for helping determine Zobrist hashes for a given position.
pub struct Zobrist {
    /// Zobrist key for each piece on each square.
    pub sq_piece: [[[u64; PIECE_TYPE_CNT]; PLAYER_CNT]; SQ_CNT], // 8 * 6 * 8
    /// Zobrist key for each possible en-passant capturable file.
    pub en_p: [u64; FILE_CNT], // 8 * 8
    /// Zobrist key for each possible castling rights.
    pub castle: [u64; ALL_CASTLING_RIGHTS], // 8 * 4
    /// Zobrist key for the side to move.
    pub side: u64, // 8
}

// Creates zobrist hashes based on a Pseudo Random Number generator.
impl Zobrist {

    /// Returns a Zobrist object.
    fn default() -> Zobrist {
        let mut zob = Zobrist {
            sq_piece: [[[0; PIECE_TYPE_CNT]; PLAYER_CNT]; SQ_CNT],
            en_p: [0; FILE_CNT],
            castle: [0; ALL_CASTLING_RIGHTS],
            side: 0,
        };

        let mut rng = PRNG::init(ZOBRIST_SEED);

        for i in 0..SQ_CNT {
            for j in 0..PIECE_TYPE_CNT {
                zob.sq_piece[i][0][j] = rng.rand();
                zob.sq_piece[i][1][j] = rng.rand();
            }
        }

        for i in 0..FILE_CNT {
            zob.en_p[i] = rng.rand()
        }

        zob.castle[0] = 0;

        for i in 1..ALL_CASTLING_RIGHTS {
            zob.castle[i] = rng.rand()
        }

        zob.side = rng.rand();
        zob
    }
}


// Size (Bytes) of each field in the Stack / Heap (Dispite this being statically allocated)
//              |  STACK  |  HEAP   |  TOTAL  | KiloBytes   |
// magic_rook   |   2563  |  819200 |  821763 | ~819.2 KB   |
// magic_bishop |   2563  |   41984 |   44547 |  ~44.5 KB   |
// knight_table |    512  |       0 |     512 |   ~0.5 KB   |
// king_table   |    512  |       0 |     512 |   ~0.5 KB   |
// dist_table   |   4096  |       0 |     512 |   ~4.0 KB   |
// line_bb      |  32768  |       0 |   32768 |   ~0.5 KB   |
// btw_sq_bb    |  32768  |       0 |   32768 |   ~0.5 KB   |
// adj_files_bb |     64  |       0 |      64 |   ~0.5 KB   |
// pawn_atks_f  |   1024  |       0 |    1024 |   ~0.5 KB   |
// Zobrist      |    600  |       0 |     600 |   ~ ???     |

/// Struct which provides various pre-computed lookup tables.
///
///
/// Thread safe. Once initializes, IT SHOULD NOT BE MODIFIED. It is intended as a globally
/// static struct created by the `Board`.
///
/// Currently does the following:
///      - Generates King and Rook Move `BitBoards`
///      - Generates Rook, Bishop, Queen Magic `BitBoard`s for Move generation
///      - Generates distance table for quick lookup of distance
///      - Line `BitBoard` and Between `BitBoard` given two squares
///      - Adjacent Files `BitBoard`.
///      - Pawn Attacks from a certain square
///      - Zobrist Structure for Zobrist Hashing
pub struct MagicHelper<'a, 'b> {
    /// Magic Bitboard structure for the rook.
    magic_rook: MagicTable<'a>,
    /// Magic Bitboard structure for the bishop.
    magic_bishop: MagicTable<'b>,
    /// Fast lookup Knight moves for each square.
    knight_table: [u64; 64],
    /// Fast lookup King moves for each square.
    king_table: [u64; 64],
    /// Fast lookup distance between each square.
    dist_table: [[u8; 64]; 64],
    /// Ring around a certain square
    dist_ring_table: [[u64; 64]; 8],
    /// Fast lookup line bitboards for any two squares.
    line_bitboard: [[u64; 64]; 64],
    /// Fast lookup bitboards for the squares between any two squares.
    between_sqs_bb: [[u64; 64]; 64],
    adjacent_files_bb: [u64; 8],
    pawn_attacks_from: [[u64; 64]; 2],

    pawn_attacks_span: [[u64; 64]; 2],
    forward_file_bb: [[u64; 64]; 2],
    passed_pawn_mask: [[u64; 64]; 2],

    forward_ranks_bb: [[u64; PLAYER_CNT]; RANK_CNT],
    /// Zobrist hasher.
    pub zobrist: Zobrist,
    pub psqt: PSQT,
}

unsafe impl<'a, 'b> Send for MagicHelper<'a, 'b> {}

unsafe impl<'a, 'b> Sync for MagicHelper<'a, 'b> {}

impl<'a, 'b> Default for MagicHelper<'a, 'b> {
    fn default() -> MagicHelper<'a, 'b> {
        MagicHelper::new()
    }
}

impl<'a, 'b> MagicHelper<'a, 'b> {
    /// Create a new Magic Helper.
    pub fn new() -> MagicHelper<'a, 'b> {
        MagicHelper::init()
            .gen_between_and_line_bbs()
            .gen_adjacent_file_bbs()
            .gen_pawn_attacks()
            .gen_ring_distance_bb()
            .gen_forward_ranks_bb()
            .gen_pawn_attacks_span()
    }

    fn init() -> MagicHelper<'a, 'b> {
        MagicHelper {
            magic_rook: MagicTable::init(ROOK_M_SIZE,&R_DELTAS),
            magic_bishop: MagicTable::init(BISHOP_M_SIZE,&B_DELTAS),
            knight_table: gen_knight_moves(),
            king_table: gen_king_moves(),
            dist_table: init_distance_table(),
            dist_ring_table: [[0; 64]; 8],
            line_bitboard: [[0; 64]; 64],
            between_sqs_bb: [[0; 64]; 64],
            adjacent_files_bb: [0; 8],
            pawn_attacks_from: [[0; 64]; 2],
            pawn_attacks_span: [[0; 64]; 2],
            forward_file_bb: [[0; 64]; 2],
            passed_pawn_mask: [[0; 64]; 2],
            forward_ranks_bb: [[0; PLAYER_CNT]; RANK_CNT],
            zobrist: Zobrist::default(),
            psqt: PSQT::init(),
        }
    }

    /// Returns the value of a piece for a player. If `eg` is true, it returns the end game value. Otherwise,
    /// it'll return the midgame value.
    pub fn piece_value(&self, piece: PieceType, player: Player, eg: bool) -> Value {
        unsafe {
            (*(*(self.psqt.piece_val.get_unchecked(player as usize)).get_unchecked(piece as usize)).get_unchecked(eg as usize))
        }
    }

    /// Returns the score for a player's piece being at a particular square.
    pub fn psq(&self, piece: PieceType, player: Player, sq: SQ) -> Score {
        unsafe {
            (*(*(self.psqt.psq.get_unchecked(player as usize)).get_unchecked(piece as usize)).get_unchecked(sq.0 as usize))
        }
    }

    /// Generate Knight Moves `BitBoard` from a source square.
    #[inline(always)]
    pub fn knight_moves(&self, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        BitBoard(
            unsafe { *self.knight_table.get_unchecked(sq.0 as usize)}
        )
    }

    /// Generate King moves `BitBoard` from a source square.
    #[inline(always)]
    pub fn king_moves(&self, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        BitBoard(
            unsafe { *self.king_table.get_unchecked(sq.0 as usize)}
        )
    }

    /// Generate Bishop Moves `BitBoard` from a bishop square and all occupied squares on the board.
    #[inline(always)]
    pub fn bishop_moves(&self, occupied: BitBoard, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        BitBoard(self.magic_bishop.attacks(occupied.0, sq.0))
    }

    /// Generate Rook Moves `BitBoard` from a bishop square and all occupied squares on the board.
    #[inline(always)]
    pub fn rook_moves(&self, occupied: BitBoard, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        BitBoard(self.magic_rook.attacks(occupied.0, sq.0))
    }

    /// Generate Queen Moves `BitBoard` from a bishop square and all occupied squares on the board.
    #[inline(always)]
    pub fn queen_moves(&self, occupied: BitBoard, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        BitBoard(self.magic_rook.attacks(occupied.0, sq.0) |
            self.magic_bishop.attacks(occupied.0, sq.0))
    }

    /// Get the distance of two squares.
    #[inline(always)]
    pub fn distance_of_sqs(&self, sq_one: SQ, sq_two: SQ) -> u8 {
        debug_assert!(sq_one.is_okay());
        debug_assert!(sq_two.is_okay());
        self.dist_table[sq_one.0 as usize][sq_two.0 as usize]
    }

    /// Get the line (diagonal / file / rank) `BitBoard` that two squares both exist on, if it exists.
    #[inline(always)]
    pub fn line_bb(&self, sq_one: SQ, sq_two: SQ) -> BitBoard {
        debug_assert!(sq_one.is_okay());
        debug_assert!(sq_two.is_okay());
        unsafe {
            BitBoard(*(self.line_bitboard.get_unchecked(sq_one.0 as usize)).get_unchecked(sq_two.0 as usize))
        }
    }

    /// Get the line (diagonal / file / rank) `BitBoard` between two squares, not including the squares, if it exists.
    #[inline(always)]
    pub fn between_bb(&self, sq_one: SQ, sq_two: SQ) -> BitBoard {
        debug_assert!(sq_one.is_okay());
        debug_assert!(sq_two.is_okay());
        unsafe {
            BitBoard(*(self.between_sqs_bb.get_unchecked(sq_one.0 as usize)).get_unchecked(sq_two.0 as usize))
        }
    }

    /// Gets the adjacent files `BitBoard` of the square
    #[inline(always)]
    pub fn adjacent_sq_file(&self, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        unsafe {
            BitBoard(*self.adjacent_files_bb.get_unchecked(sq.file() as usize))
        }
    }

    /// Gets the adjacent files `BitBoard` of the file
    #[inline(always)]
    pub fn adjacent_file(&self, f: File) -> BitBoard {
        unsafe {
            BitBoard(*self.adjacent_files_bb.get_unchecked(f as usize))
        }
    }

    /// Pawn attacks `BitBoard` from a given square, per player.
    /// Basically, given square x, returns the BitBoard of squares a pawn on x attacks.
    #[inline(always)]
    pub fn pawn_attacks_from(&self, sq: SQ, player: Player) -> BitBoard {
        debug_assert!(sq.is_okay());
        unsafe {
            BitBoard (
                *self.pawn_attacks_from.get_unchecked(player as usize).get_unchecked(sq.0 as usize)
//                match player {
//                    Player::White => self.pawn_attacks_from.get_unchecked(0).get_unchecked(sq.0 as usize),
//                    Player::Black => self.pawn_attacks_from[1][sq.0 as usize],
//                }
            )
        }
    }


    /// Returns if three Squares are in the same diagonal, file, or rank.
    #[inline(always)]
    pub fn aligned(&self, s1: SQ, s2: SQ, s3: SQ) -> bool {
        (self.line_bb(s1, s2) & s3.to_bb()).is_not_empty()
    }

    /// Returns the Zobrist Hash for a given piece as a given Square
    #[inline(always)]
    pub fn z_piece_at_sq(&self, piece: PieceType, square: SQ, player: Player) -> u64 {
        debug_assert!(square.is_okay());
        unsafe {
            *(*(self.zobrist.sq_piece.get_unchecked(square.0 as usize)).get_unchecked(player as usize)).get_unchecked(piece as usize)
        }
//        self.zobrist.sq_piece[square.0 as usize][piece as usize]
    }

    /// Returns the zobrist hash for the given Square of Enpassant.
    /// Doesnt assume the EP square is a valid square. It will take the file of the square regardless.
    #[inline(always)]
    pub fn z_ep_file(&self, square: SQ) -> u64 {
        debug_assert!(square.is_okay());
        unsafe {
            *self.zobrist.en_p.get_unchecked(file_of_sq(square.0) as usize)
        }
    }

    /// Returns a zobrast hash of the castling rights, as defined by the Board.
    #[inline(always)]
    pub fn z_castle_rights(&self, castle: u8) -> u64 {
        debug_assert!((castle as usize) < ALL_CASTLING_RIGHTS);
        unsafe {
            *self.zobrist.en_p.get_unchecked(castle as usize)
        }
    }

    /// Returns Zobrist Hash of flipping sides.
    #[inline(always)]
    pub fn z_side(&self) -> u64 {
        self.zobrist.side
    }

    /// Returns the ring of bits surrounding the square sq at a specified distance.
    ///
    /// # Safety
    ///
    /// distance must be less than 8, or else a panic will occur.
    #[inline(always)]
    pub fn ring_distance(&self, sq: SQ, distance: u8) -> BitBoard {
        debug_assert!(distance <= 7);
        unsafe {
            BitBoard(*self.dist_ring_table.get_unchecked(distance as usize).get_unchecked(sq.0 as usize))
        }
    }


    /// Returns the BitBoard of all squares in the rank in front of the given one.
    pub fn forward_rank_bb(&self, player: Player, rank: Rank) -> BitBoard {
        unsafe {
            BitBoard(*self.forward_ranks_bb.get_unchecked(rank as usize).get_unchecked(player as usize))
        }
//        BitBoard(self.forward_ranks_bb[rank as usize][player as usize])
    }

    /// Returns the `BitBoard` of all squares that can be attacked by a pawn
    /// of the same color when it moves along its file, starting from the
    /// given square. Basically, if the pawn progresses along the same file
    /// for the entire game, this bitboard would contain all possible forward squares
    /// it could attack
    ///
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
    #[inline(always)]
    pub fn pawn_attacks_span(&self, player: Player, sq: SQ) -> BitBoard {
        debug_assert!(sq.is_okay());
        unsafe {
            BitBoard(*self.pawn_attacks_span.get_unchecked(player as usize).get_unchecked(sq.0 as usize))
        }
    }

    /// Returns the BitBoard of all squares in the file in front of the given one.
    ///
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
    #[inline(always)]
    pub fn forward_file_bb(&self, player: Player, sq: SQ) -> BitBoard {
        unsafe {
            BitBoard(*self.forward_file_bb.get_unchecked(player as usize).get_unchecked(sq.0 as usize))
        }
    }

    /// Returns a `BitBoard` allowing for testing of the a pawn being a
    /// "passed pawn".
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
    #[inline(always)]
    pub fn passed_pawn_mask(&self, player: Player, sq: SQ) -> BitBoard {
        unsafe {
            BitBoard(*self.passed_pawn_mask.get_unchecked(player as usize).get_unchecked(sq.0 as usize))
        }
    }

    #[inline(always)]
    fn bishop_moves_bb(&self, occupied: u64, square: u8) -> u64 {
        debug_assert!(square < 64);
        self.magic_bishop.attacks(occupied, square)
    }

    #[inline(always)]
    fn rook_moves_bb(&self, occupied: u64, square: u8) -> u64 {
        debug_assert!(square < 64);
        self.magic_rook.attacks(occupied, square)
    }


    fn gen_between_and_line_bbs(mut self) -> Self {
        for i in 0..64 as u8 {
            for j in 0..64 as u8 {
                let i_bb: u64 = (1 as u64) << i;
                let j_bb: u64 = (1 as u64) << j;
                if self.rook_moves_bb(0, i) & j_bb != 0 {
                    self.line_bitboard[i as usize][j as usize] |=
                        (self.rook_moves_bb(0, j) & self.rook_moves_bb(0, i)) | i_bb | j_bb;
                    self.between_sqs_bb[i as usize][j as usize] = self.rook_moves_bb(i_bb, j) &
                        self.rook_moves_bb(j_bb, i);
                } else if self.bishop_moves_bb(0, i) & j_bb != 0 {
                    self.line_bitboard[i as usize][j as usize] |=
                        (self.bishop_moves_bb(0, j) & self.bishop_moves_bb(0, i)) | i_bb | j_bb;
                    self.between_sqs_bb[i as usize][j as usize] = self.bishop_moves_bb(i_bb, j) &
                        self.bishop_moves_bb(j_bb, i);
                } else {
                    self.line_bitboard[i as usize][j as usize] = 0;
                    self.between_sqs_bb[i as usize][j as usize] = 0;
                }
            }
        }
        self
    }

    // Generates adjacent files of a given file
    // Files go from 0..7, representing files 1..8
    fn gen_adjacent_file_bbs(mut self) -> Self {
        for file in 0..8 as u8 {
            if file != 0 {
                self.adjacent_files_bb[file as usize] |= file_bb(file - 1);
            }
            if file != 7 {
                self.adjacent_files_bb[file as usize] |= file_bb(file + 1);
            }
        }
        self
    }

    /// Generate the pawn attacks table. Maps from a square to the bitboard of
    /// possible attacks from that square. This is done per player.
    fn gen_pawn_attacks(mut self) -> Self {
        // gen white pawn attacks
        for i in 0..56 as u8 {
            let mut bb: u64 = 0;
            if file_of_sq(i) != File::A {
                bb |= u8_to_u64(i + 7)
            }
            if file_of_sq(i) != File::H {
                bb |= u8_to_u64(i + 9)
            }
            self.pawn_attacks_from[0][i as usize] = bb;
        }

        // Black pawn attacks
        for i in 8..64 as u8 {
            let mut bb: u64 = 0;
            if file_of_sq(i) != File::A {
                bb |= u8_to_u64(i - 9)
            }
            if file_of_sq(i) != File::H {
                bb |= u8_to_u64(i - 7)
            }
            self.pawn_attacks_from[1][i as usize] = bb;
        }
        self
    }

    fn gen_ring_distance_bb(mut self) -> Self {
        for i in 0..64 {
            for j in 0..64 {
                if i != j {
                    let dist = self.dist_table[i][j] as usize;
                    self.dist_ring_table[dist - 1][i] |= (1 as u64) << (j as usize);
                }
            }
        }
        self
    }

    fn gen_forward_ranks_bb(mut self) -> Self {
        for i in 0..7 {
            self.forward_ranks_bb[i + 1][Player::Black as usize] = self.forward_ranks_bb[i][Player::Black as usize] | rank_bb(i as u8);
            self.forward_ranks_bb[i][Player::White as usize] = !self.forward_ranks_bb[i + 1][Player::Black as usize];
        }
        self
    }

    fn gen_pawn_attacks_span(mut self) -> Self {
        for p in 0..2 {
            for s in 0..64 {
                self.forward_file_bb[p][s] = self.forward_ranks_bb[rank_idx_of_sq(s as u8) as usize][p] & FILE_BB[file_idx_of_sq(s as u8) as usize];
                self.pawn_attacks_span[p][s] = self.forward_ranks_bb[rank_idx_of_sq(s as u8) as usize][p] & self.adjacent_files_bb[file_idx_of_sq(s as u8) as usize];
                self.passed_pawn_mask[p][s] = self.forward_file_bb[p][s] | self.pawn_attacks_span[p][s];
            }
        }

        self
    }
}


/// Structure inside a `MagicTable` for a specific hash. For a certain square,
/// contains a mask,  magic number, number to shift by, and a pointer into the array slice
/// where the position is held.
#[warn(dead_code)]
struct SMagic<'a> {
    ptr: &'a [u64],
    mask: u64,
    magic: u64,
    shift: u32,
}

/// Temporary struct used to create an actual `SMagic` Object.
#[warn(dead_code)]
struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl PreSMagic {
    pub fn init() -> PreSMagic {
        PreSMagic {
            start: 0,
            len: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }

    // creates an array of PreSMagic
    pub unsafe fn init64() -> [PreSMagic; 64] {
        let arr: [PreSMagic; 64] = mem::uninitialized();
        arr
    }

    // Helper method to compute the next index
    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

/// Contains the actual data of pre-computed tables for a given
/// piece (either rook or bishop).
struct MagicTable<'a> {
    sq_magics: [SMagic<'a>; 64],
    attacks: Vec<u64>
}

impl<'a> MagicTable<'a> {
    /// Simple version that creates the table with an empty array.
    /// used for testing purposes where MagicStruct is not needed.
    pub fn simple() -> MagicTable<'a> {
        let sq_table: [SMagic<'a>; 64] = unsafe { mem::uninitialized() };
        MagicTable {
            sq_magics: sq_table,
            attacks: Vec::new(),
        }
    }

    /// Creates the `MagicTable` struct. The table size is relative to the piece for computation,
    /// and the deltas are the directions on the board the piece can go.
    pub fn init(table_size: usize, deltas: &[i8; 4]) -> MagicTable<'a> {
        // Creates PreSMagic to hold raw numbers. Technically jsut adds room to stack
        let mut pre_sq_table: [PreSMagic; 64] = unsafe { PreSMagic::init64() };

        // Initializes each PreSMagic
        for table in pre_sq_table.iter_mut() {
            *table = PreSMagic::init();
        }

        // Creates Vector to hold attacks. Has capacity as we know the exact size of this.
        let mut attacks: Vec<u64> = vec![0u64; table_size];

        // Occupancy tracks occupancy permutations. MAX permutations = subset of 12 bits = 2^12
        // Reference is similar, tracks the sliding moves from a given occupancy
        // Age tracks the best index for a current permutation
        let mut occupancy: [u64; 4096] = [0; 4096];
        let mut reference: [u64; 4096] = [0; 4096];
        let mut age: [i32; 4096] = [0; 4096];

        // Size tracks the size of permutations of the current block
        let mut size: usize;

        // b is used for generating the permutations through ripple - carry
        let mut b: u64;

        // current and i is a placeholder for actually generating correct magic numbers
        let mut current: i32 = 0;
        let mut i: usize;

        // set the first PreSMagic start = 0. Just in case.
        pre_sq_table[0].start = 0;

        // Loop through each square! s is a SQ
        for s in 0..64 as u8 {
            // Magic number for later
            let mut magic: u64;

            // edges is the bitboard represenation of the edges s is not on.
            // e.g. sq A1 is on FileA and Rank1, so edges = bitboard of FileH and Rank8
            // mask = occupancy mask of square s
            let edges: u64 = ((RANK_1 | RANK_8) & !rank_bb(s)) |
                ((FILE_A | FILE_H) & !file_bb(s));
            let mask: u64 = sliding_attack(deltas, s, 0) & !edges;

            // Shift = number of bits in 64 - bits in mask = log2(size)
            let shift: u32 = (64 - popcount64(mask)) as u32;
            b = 0;
            size = 0;

            // Ripple carry to determine occupancy, reference, and size
            'bit: loop {
                occupancy[size] = b;
                reference[size] = sliding_attack(deltas, s, b);
                size += 1;
                b = ((b).wrapping_sub(mask)) as u64 & mask;
                if b == 0 {
                    break 'bit;
                }
            }

            // Set current PreSMagic length to be of size
            pre_sq_table[s as usize].len = size;

            // If there is a next square, set the start of it.
            if s < 63 {
                pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();

            }
            // Create our Random Number Generator with a seed
            let mut rng = PRNG::init(SEEDS[1][SQ(s).rank() as usize]);

            // Loop until we have found our magics!
            'outer: loop {
                // Create a magic with our desired number of bits in the first 8 places
                'first_in: loop {
                    magic = rng.sparse_rand();
                    if popcount64((magic.wrapping_mul(mask)).wrapping_shr(56)) >= 6 {
                        break 'first_in;
                    }
                }
                current += 1;
                i = 0;

                // Filling the attacks Vector up to size digits
                while i < size {
                    // Magic part! The index is = ((occupancy[s] & mask) * magic >> shift)
                    let index: usize = ((occupancy[i as usize] & mask).wrapping_mul(magic) as
                        u64)
                        .wrapping_shr(shift) as usize;

                    // Checking to see if we have visited this index already with a lower current number
                    if age[index] < current {

                        // If we have visited with lower current, we replace it with this current number,
                        // as this current is higher and has gone through more passes
                        age[index] = current;
                        attacks[pre_sq_table[s as usize].start + index] = reference[i];

                    } else if attacks[pre_sq_table[s as usize].start + index] != reference[i] {
                        // If a magic maps to the same index but different result, either magic is bad or we are done
                        break;
                    }
                    i += 1;
                }
                // If we have filled it up to size or greater, we are done
                if i >= size {
                    break 'outer;
                }
            }
            // Set the remaining variables for the PreSMagic Struct
            pre_sq_table[s as usize].magic = magic;
            pre_sq_table[s as usize].mask = mask;
            pre_sq_table[s as usize].shift = shift;
        }

        // Now the fun part. We got to convert all the PreMagicStructs to MStructs
        // UNSAFE as we are initializing raw memory, AND creating a Slice of our array from raw pointers. scary!
        unsafe {
            // Make Memory for our SMagics!
            let mut sq_table: [SMagic<'a>; 64] = mem::uninitialized();

            // size = running total of total size
            let mut size = 0;
            for i in 0..64 {
                // begin ptr points to the beginning of the current slice in the vector
                let beginptr = attacks.as_ptr().offset(size as isize);
                let mut table_i: SMagic = SMagic {
                    ptr: mem::uninitialized(),
                    mask: pre_sq_table[i].mask,
                    magic: pre_sq_table[i].magic,
                    shift: pre_sq_table[i].shift,
                };
                // Create the pointer to the slice with begin_ptr / length
                table_i.ptr = slice::from_raw_parts(beginptr, pre_sq_table[i].len);
                size += pre_sq_table[i].len;
                sq_table[i] = table_i;
            }
            // Sanity check
            assert_eq!(size, table_size);
            MagicTable {
                sq_magics: sq_table,
                attacks: attacks,
            }
        }
    }

    /// Returns an attack bitboard (as a u64) for a given square and occupancy board.
    /// The result must be AND'd with the negation of the player's occupied bitboard, as
    /// the returned u64 also contains bits where the piece can capture it's own pieces.
    #[inline(always)]
    pub fn attacks(&self, mut occupied: u64, square: u8) -> u64 {
        let magic_entry = unsafe { self.sq_magics.get_unchecked(square as usize)};
        occupied &= magic_entry.mask;
        occupied = occupied.wrapping_mul(magic_entry.magic);
        occupied = occupied.wrapping_shr(magic_entry.shift);
        unsafe { *magic_entry.ptr.get_unchecked(occupied as usize) }
    }
}


/// Returns an array of king moves (bitboards) for each square.
fn gen_king_moves() -> [u64; 64] {
    let mut moves: [u64; 64] = [0; 64];

    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;
        // LEFT
        if file != 0 {
            mask |= 1 << (index - 1);
        }
        // RIGHT
        if file != 7 {
            mask |= 1 << (index + 1);
        }
        // UP
        if index < 56 {
            mask |= 1 << (index + 8);
        }
        // DOWN
        if index > 7 {
            mask |= 1 << (index - 8);
        }
        // LEFT UP
        if file != 0 && index < 56 {
            mask |= 1 << (index + 7);
        }
        // LEFT DOWN
        if file != 0 && index > 7 {
            mask |= 1 << (index - 9);
        }
        // RIGHT DOWN
        if file != 7 && index > 7 {
            mask |= 1 << (index - 7);
        }
        // RIGHT UP
        if file != 7 && index < 56 {
            mask |= 1 << (index + 9);
        }
        moves[index] = mask;
    }
    moves
}

// Returns an array of knight moves, seeing as kings can only move up to
// 8 static places no matter the square
fn gen_knight_moves() -> [u64; 64] {
    let mut moves: [u64; 64] = [0; 64];
    for (index, spot) in moves.iter_mut().enumerate() {
        let mut mask: u64 = 0;
        let file = index % 8;

        // 1 UP   + 2 LEFT
        if file > 1 && index < 56 {
            mask |= 1 << (index + 6);
        }
        // 2 UP   + 1 LEFT
        if file != 0 && index < 48 {
            mask |= 1 << (index + 15);
        }
        // 2 UP   + 1 RIGHT
        if file != 7 && index < 48 {
            mask |= 1 << (index + 17);
        }
        // 1 UP   + 2 RIGHT
        if file < 6 && index < 56 {
            mask |= 1 << (index + 10);
        }
        // 1 DOWN   + 2 RIGHT
        if file < 6 && index > 7 {
            mask |= 1 << (index - 6);
        }
        // 2 DOWN   + 1 RIGHT
        if file != 7 && index > 15 {
            mask |= 1 << (index - 15);
        }
        // 2 DOWN   + 1 LEFT
        if file != 0 && index > 15 {
            mask |= 1 << (index - 17);
        }
        // 1 DOWN   + 2 LEFT
        if file > 1 && index > 7 {
            mask |= 1 << (index - 10);
        }
        *spot = mask;
    }
    moves
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: &[i8; 4], sq: u8, occupied: u64) -> u64 {
    assert!(sq < 64);
    let mut attack: u64 = 0;
    let square: i16 = sq as i16;
    for delta in deltas.iter().take(4 as usize) {
        let mut s: u8 = ((square as i16) + (*delta as i16)) as u8;
        'inner: while s < 64 &&
            SQ(s as u8).distance(SQ(((s as i16) - (*delta as i16)) as u8)) == 1
        {
            attack |= (1 as u64).wrapping_shl(s as u32);
            if occupied & (1 as u64).wrapping_shl(s as u32) != 0 {
                break 'inner;
            }
            s = ((s as i16) + (*delta as i16)) as u8;
        }
    }
    attack
}

/// Return an array mapping from two squares to the distance between them.
/// distance is in terms of squares away, not algebraic distance.
fn init_distance_table() -> [[u8; 64]; 64] {
    let mut arr: [[u8; 64]; 64] = [[0; 64]; 64];
    for i in 0..64 as u8 {
        for j in 0..64 as u8 {
            arr[i as usize][j as usize] = (SQ(i)).distance(SQ(j));
        }
    }
    arr
}

#[cfg(test)]
mod tests {

    use super::*;

//    #[allow(unused_imports)]
//    use test;

    #[test]
    fn test_king_mask_gen() {
        let arr = gen_king_moves().to_vec();
        let sum = arr.iter()
            .fold(0 as u64, |a, &b| a + (popcount64(b) as u64));
        assert_eq!(sum, (3 * 4) + (5 * 6 * 4) + (8 * 6 * 6));
    }

    #[test]
    fn test_knight_mask_gen() {
        let arr = gen_knight_moves().to_vec();
        let sum = arr.iter()
            .fold(0 as u64, |a, &b| a + (popcount64(b) as u64));
        assert_eq!(
            sum,
            (2 * 4) + (4 * 4) + (3 * 2 * 4) + (4 * 4 * 4) + (6 * 4 * 4) + (8 * 4 * 4)
        );
    }

    #[test]
    fn occupancy_and_sliding() {
        let rook_deltas: [i8; 4] = [8, 1, -8, -1];
        assert_eq!(popcount64(sliding_attack(&rook_deltas, 0, 0)), 14);
        assert_eq!(popcount64(sliding_attack(&rook_deltas, 0, 0xFF00)), 8);
        assert_eq!(popcount64(sliding_attack(&rook_deltas, 19, 0)), 14);
    }

    #[test]
    fn rmagics() {
        let mstruct = MagicTable::init(ROOK_M_SIZE, &R_DELTAS);
        assert_eq!(mem::size_of_val(&mstruct), 2584);
        let bstruct = MagicTable::init(BISHOP_M_SIZE, &B_DELTAS);
        assert_eq!(mem::size_of_val(&bstruct), 2584);
    }


}

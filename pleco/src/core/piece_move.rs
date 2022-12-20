//! Module for the implementation and definition of a move to be played.
//!
//! We define a move as the struct `BitMove`. A move needs 16 bits to be stored, and
//! they are used as such:
//!
//! ```md,ignore
//! bits  0 - 5:  destination square (from 0 to 63)
//! bits  6 - 11: origin square (from 0 to 63)
//! bits 12 - 13: promotion piece type - 2 (from KNIGHT-0 to QUEEN-4)
//! bits 14 - 15: special move flag: promotion (1), en passant (2), castling (3)
//! ```
//!
//! # Special cases
//!
//! Special cases are MOVE_NONE and MOVE_NULL. We can sneak these in because in
//! any normal move destination square is always different from origin square
//! while MOVE_NONE and MOVE_NULL have the same origin and destination square.
//!
//! Another special case is where the move is a castling move. If the move is a
//! castle, then the corresponding flags will be set and the origin square will be the
//! square of the king, while the destination square will be the square of the rook to
//! castle with.
//!
//! Lastly, the En-passant flag is only set if the move is a pawn double-push.
//!
//! # Bit Flags for a `BitMove`
//!
//! The flags for a move are set as such:
//!
//! ```md,ignore
//! x??? --> Promotion bit
//! ?x?? --> Capture bit
//! ??xx --> flag Bit
//! ```
//!
//! More specifically, the flags correspond to the following bit patterns:
//!
//! ```md,ignore
//! 0000  ===> Quiet move
//! 0001  ===> Double Pawn Push
//! 0010  ===> King Castle
//! 0011  ===> Queen Castle
//! 0100  ===> Capture
//! 0101  ===> EP Capture
//! 0110  ===>
//! 0111  ===>
//! 1000  ===> Knight Promotion
//! 1001  ===> Bishop Promo
//! 1010  ===> Rook   Promo
//! 1011  ===> Queen  Capture  Promo
//! 1100  ===> Knight Capture  Promotion
//! 1101  ===> Bishop Capture  Promo
//! 1110  ===> Rook   Capture  Promo
//! 1111  ===> Queen  Capture  Promo
//! ```
//!
//! # Safety
//!
//! A `BitMove` is only guaranteed to be legal for a specific position. If a Board generates a
//! list of moves, then only those moves are correct. It is not recommended to use `BitMove`s
//! on a `Board` that didn't directly create them, unless it is otherwise known that move
//! correlates to that specific board position.

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::fmt;

use super::sq::SQ;
use super::*;

// Castles have the src as the king bit and the dst as the rook
const SRC_MASK: u16 = 0b0000_000000_111111;
const DST_MASK: u16 = 0b0000_111111_000000;
const FROM_TO_MASK: u16 = 0b0000_111111_111111;
const PR_MASK: u16 = 0b1000_000000_000000;
const CP_MASK: u16 = 0b0100_000000_000000;
const FLAG_MASK: u16 = 0b1111_000000_000000;
const SP_MASK: u16 = 0b0011_000000_000000;

/// Represents a singular move.
///
/// A `BitMove` consists of 16 bits, all of which to include a source square, destination square,
/// and special move-flags to differentiate types of moves.
///
/// A `BitMove` should never be created directly, but rather instigated with a `PreMoveInfo`. This is because
/// the bits are in a special order, and manually creating moves risks creating an invalid move.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct BitMove {
    data: u16,
}

/// Selected Meta-Data to accompany each move.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    /// The move is a promotion.
    Promotion {
        /// Marks the move as a capturing promotion.
        capture: bool,
        /// The piece that the move promotes to.
        prom: PieceType,
    },
    /// The move is a castle.
    Castle {
        /// Determines if the castle is a castle on the king side.
        king_side: bool,
    },
    /// The move is a double pawn push.
    DoublePawnPush,
    /// The move is a capturing move.
    Capture {
        /// Marks this move as an en-passant capture.
        ep_capture: bool,
    },
    /// The move is a quiet move. This means its not a capture, promotion, castle, or double-pawn push.
    QuietMove,
}

/// A Subset of `MoveFlag`, used to determine the overall classification of a move.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MoveType {
    /// The move is "Normal", So its not a castle, promotion, or en-passant.
    Normal = 0, //0b000x
    /// The move is castling move.
    Castle = 1, //0b001x
    /// The move is an en-passant capture.
    EnPassant = 5, // 0b0101
    /// The move is a promotion.
    Promotion = 8, //0b1xxx
}

/// Useful pre-encoding of a move's information before it is compressed into a `BitMove` struct.
#[derive(Copy, Clone, PartialEq)]
pub struct PreMoveInfo {
    /// The square the moving piece originates from.
    pub src: SQ,
    /// The square the piece is moving to.
    pub dst: SQ,
    /// Marks the type of move. E.g, Promotion, Castle, Capture.
    pub flags: MoveFlag,
}

impl fmt::Display for BitMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}

// https://chessprogramming.wikispaces.com/Encoding+Moves
impl BitMove {
    pub const FLAG_QUIET: u16 = 0b0000;
    pub const FLAG_DOUBLE_PAWN: u16 = 0b0001;
    pub const FLAG_KING_CASTLE: u16 = 0b0010;
    pub const FLAG_QUEEN_CASTLE: u16 = 0b0011;
    pub const FLAG_CAPTURE: u16 = 0b0100;
    pub const FLAG_EP: u16 = 0b0101;
    pub const ILLEGAL_FLAG_1: u16 = 0b0110;
    pub const ILLEGAL_FLAG_2: u16 = 0b0111;
    pub const FLAG_PROMO_N: u16 = 0b1000;
    pub const FLAG_PROMO_B: u16 = 0b1001;
    pub const FLAG_PROMO_R: u16 = 0b1010;
    pub const FLAG_PROMO_Q: u16 = 0b1011;
    pub const FLAG_PROMO_CAP_N: u16 = 0b1100;
    pub const FLAG_PROMO_CAP_B: u16 = 0b1101;
    pub const FLAG_PROMO_CAP_R: u16 = 0b1110;
    pub const FLAG_PROMO_CAP_Q: u16 = 0b1111;

    /// Creates a new BitMove from raw bits.
    ///
    /// # Safety
    ///
    /// Using this method cannot guarantee that the move is legal. The input bits must be encoding a legal
    /// move, or else there is Undefined Behavior if the resulting BitMove is used.
    #[inline]
    pub const fn new(input: u16) -> BitMove {
        BitMove { data: input }
    }

    /// Makes a quiet `BitMove` from a source and destination square.
    #[inline(always)]
    pub const fn make_quiet(src: SQ, dst: SQ) -> BitMove {
        BitMove::make(BitMove::FLAG_QUIET, src, dst)
    }

    /// Makes a pawn-push `BitMove` from a source and destination square.
    #[inline(always)]
    pub const fn make_pawn_push(src: SQ, dst: SQ) -> BitMove {
        BitMove::make(BitMove::FLAG_DOUBLE_PAWN, src, dst)
    }

    /// Makes a non-enpassant capturing `BitMove` from a source and destination square.
    #[inline(always)]
    pub const fn make_capture(src: SQ, dst: SQ) -> BitMove {
        BitMove::make(BitMove::FLAG_CAPTURE, src, dst)
    }

    /// Makes an enpassant `BitMove` from a source and destination square.
    #[inline(always)]
    pub const fn make_ep_capture(src: SQ, dst: SQ) -> BitMove {
        BitMove::make(BitMove::FLAG_EP, src, dst)
    }

    /// Creates a `BitMove` from a source and destination square, as well as the current
    /// flag.
    #[inline(always)]
    pub const fn make(flag_bits: u16, src: SQ, dst: SQ) -> BitMove {
        BitMove {
            data: (flag_bits << 12) | src.0 as u16 | ((dst.0 as u16) << 6),
        }
    }

    /// Returns the promotion flag bits of a `PieceType`.
    #[inline(always)]
    fn promotion_piece_flag(piece: PieceType) -> u16 {
        match piece {
            PieceType::R => 2,
            PieceType::B => 1,
            PieceType::N => 0,
            PieceType::Q | _ => 3,
        }
    }

    /// Creates a BitMove from a `PreMoveInfo`.
    #[inline]
    pub fn init(info: PreMoveInfo) -> BitMove {
        let src = info.src.0 as u16;
        let dst = (info.dst.0 as u16) << 6;
        let flags = info.flags;
        let flag_bits: u16 = match flags {
            MoveFlag::Promotion { capture, prom } => {
                let p_bit: u16 = BitMove::promotion_piece_flag(prom);
                let cp_bit = if capture { 4 } else { 0 };
                p_bit + cp_bit + 8
            }
            MoveFlag::Capture { ep_capture } => {
                if ep_capture {
                    5
                } else {
                    4
                }
            }
            MoveFlag::Castle { king_side } => {
                if king_side {
                    2
                } else {
                    3
                }
            }
            MoveFlag::DoublePawnPush => 1,
            MoveFlag::QuietMove => 0,
        };
        BitMove {
            data: (flag_bits << 12) | src | dst,
        }
    }

    /// Creates a Null Move.
    ///
    /// # Safety
    ///
    /// A Null move is never a valid move to play. Using a Null move should only be used for search and
    /// evaluation purposes.
    #[inline]
    pub const fn null() -> Self {
        BitMove { data: 0 }
    }

    /// Returns if a `BitMove` is a Null Move.
    ///
    /// See `BitMove::null()` for more information on Null moves.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.data == 0
    }

    /// Returns if a `BitMove` captures an opponent's piece.
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        ((self.data & CP_MASK) >> 14) == 1
    }

    /// Returns if a `BitMove` is a Quiet Move, meaning it is not any of the following: a capture, promotion, castle, or double pawn push.
    #[inline(always)]
    pub const fn is_quiet_move(self) -> bool {
        self.flag() == 0
    }

    /// Returns if a `BitMove` is a promotion.
    #[inline(always)]
    pub const fn is_promo(self) -> bool {
        (self.data & PR_MASK) != 0
    }

    /// Returns the destination of a `BitMove`.
    #[inline(always)]
    pub const fn get_dest(self) -> SQ {
        SQ(self.get_dest_u8())
    }

    /// Returns the destination of a `BitMove`.
    #[inline(always)]
    pub const fn get_dest_u8(self) -> u8 {
        ((self.data & DST_MASK) >> 6) as u8
    }

    /// Returns the source square of a `BitMove`.
    #[inline(always)]
    pub const fn get_src(self) -> SQ {
        SQ(self.get_src_u8())
    }

    /// Returns the source square of a `BitMove`.
    #[inline(always)]
    pub const fn get_src_u8(self) -> u8 {
        (self.data & SRC_MASK) as u8
    }

    /// Returns if a `BitMove` is a castle.
    #[inline(always)]
    pub const fn is_castle(self) -> bool {
        (self.data >> 13) == 1
    }

    /// Returns if a `BitMove` is a Castle && it is a KingSide Castle.
    #[inline(always)]
    pub const fn is_king_castle(self) -> bool {
        self.flag() == BitMove::FLAG_KING_CASTLE
    }

    /// Returns if a `BitMove` is a Castle && it is a QueenSide Castle.
    #[inline(always)]
    pub const fn is_queen_castle(self) -> bool {
        self.flag() == BitMove::FLAG_QUEEN_CASTLE
    }

    /// Returns if a `BitMove` is an enpassant capture.
    #[inline(always)]
    pub const fn is_en_passant(self) -> bool {
        self.flag() == BitMove::FLAG_EP
    }

    /// Returns if a `BitMove` is a double push, and if so returns the Destination square as well.
    #[inline(always)]
    pub fn is_double_push(self) -> (bool, u8) {
        let is_double_push: u8 = self.flag() as u8;
        match is_double_push {
            1 => (true, self.get_dest().0 as u8),
            _ => (false, 64),
        }
    }

    /// Returns the `Rank` (otherwise known as row) that the destination square  of a `BitMove`
    /// lies on.
    #[inline(always)]
    pub fn dest_row(self) -> Rank {
        //        ALL_RANKS[(((self.data & DST_MASK) >> 6) as u8 / 8) as usize]
        self.get_dest().rank()
    }

    /// Returns the `File` (otherwise known as column) that the destination square of a `BitMove`
    /// lies on.
    #[inline(always)]
    pub fn dest_col(self) -> File {
        self.get_dest().file()
    }

    /// Returns the `Rank` (otherwise known as row) that the from-square of a `BitMove` lies on.
    #[inline(always)]
    pub fn src_row(self) -> Rank {
        self.get_src().rank()
    }

    /// Returns the `File` (otherwise known as column) that the from-square of a `BitMove` lies on.
    #[inline(always)]
    pub fn src_col(self) -> File {
        self.get_src().file()
    }

    /// Returns the Promotion Piece of a [BitMove].
    ///
    /// # Safety
    ///
    /// Method should only be used if the [BitMove] is a promotion. Otherwise, Undefined Behavior may result.
    #[inline(always)]
    pub fn promo_piece(self) -> PieceType {
        match (self.flag()) & 0b0011 {
            0 => PieceType::N,
            1 => PieceType::B,
            2 => PieceType::R,
            3 | _ => PieceType::Q,
        }
    }

    // TODO: Simply with (m >> 4) & 3
    /// Returns the `MoveType` of a `BitMove`.
    #[inline(always)]
    pub fn move_type(self) -> MoveType {
        if self.is_castle() {
            return MoveType::Castle;
        }
        if self.is_promo() {
            return MoveType::Promotion;
        }
        if self.is_en_passant() {
            return MoveType::EnPassant;
        }
        MoveType::Normal
    }

    /// Returns a String representation of a `BitMove`.
    ///
    /// Format goes "Source Square, Destination Square, (Promo Piece)". Moving a Queen from A1 to B8
    /// will stringify to "a1b8". If there is a pawn promotion involved, the piece promoted to will be
    /// appended to the end of the string, alike "a7a8q" in the case of a queen promotion.
    pub fn stringify(self) -> String {
        let src = self.get_src().to_string();
        let dst_sq = self.get_dest();

        let dst = if self.is_castle() {
            match dst_sq {
                SQ::A8 => String::from("c8"),
                SQ::A1 => String::from("c1"),
                SQ::H8 => String::from("g8"),
                SQ::H1 => String::from("g1"),
                _ => dst_sq.to_string(),
            }
        } else {
            dst_sq.to_string()
        };
        let mut s = format!("{}{}", src, dst);
        if self.is_promo() {
            let c = self.promo_piece().char_lower();
            s.push(c);
        }
        s
    }

    /// Returns the raw number representation of the move.
    #[inline(always)]
    pub const fn get_raw(self) -> u16 {
        self.data
    }

    /// Returns if the move has an incorrect flag inside, and therefore is invalid.
    #[inline(always)]
    pub fn incorrect_flag(self) -> bool {
        ((self.flag()) & 0b1110) == 0b0110
    }

    /// Returns the 4 bit flag of the `BitMove`.
    #[inline(always)]
    pub const fn flag(self) -> u16 {
        self.data >> 12
    }

    /// Returns if the move is within bounds, ala the to and from squares
    /// are not equal.
    #[inline(always)]
    pub const fn is_okay(self) -> bool {
        self.get_dest_u8() != self.get_src_u8()
    }

    /// Returns only from "from" and "to" squares of the move.
    #[inline(always)]
    pub const fn from_to(self) -> u16 {
        self.data & FROM_TO_MASK
    }
}

/// Structure containing both a score (represented as a i16) and a `BitMove`.
///
/// This is useful for tracking a list of moves alongside each of their scores.
#[derive(Eq, Copy, Clone, Debug)]
#[repr(C)]
pub struct ScoringMove {
    pub bit_move: BitMove,
    pub score: i16,
}

impl Default for ScoringMove {
    #[inline(always)]
    fn default() -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score: 0,
        }
    }
}

impl ScoringMove {
    /// Creates a new `ScoringMove` with a default score of 0.
    #[inline(always)]
    pub fn new(m: BitMove) -> Self {
        ScoringMove {
            bit_move: m,
            score: 0,
        }
    }

    /// Creates a new `ScoringMove`.
    #[inline(always)]
    pub fn new_score(m: BitMove, score: i16) -> Self {
        ScoringMove { bit_move: m, score }
    }

    /// Returns a `ScoringMove` containing a `BitMove::null()` and a user-defined score.
    #[inline(always)]
    pub fn blank(score: i16) -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score,
        }
    }

    /// Returns the move.
    #[inline(always)]
    pub fn bitmove(self) -> BitMove {
        self.bit_move
    }

    /// Returns the score.
    #[inline(always)]
    pub fn score(self) -> i16 {
        self.score
    }

    /// Negates the current score.
    #[inline(always)]
    pub fn negate(mut self) -> Self {
        self.score = self.score.wrapping_neg();
        self
    }

    /// Swaps the current move with another move.
    #[inline(always)]
    pub fn swap_move(mut self, mov: BitMove) -> Self {
        self.bit_move = mov;
        self
    }

    /// Returns a `ScoringMove` containing a `BitMove::null()` and a score of zero.
    #[inline(always)]
    pub const fn null() -> Self {
        ScoringMove {
            bit_move: BitMove::null(),
            score: 0,
        }
    }
}

impl Ord for ScoringMove {
    fn cmp(&self, other: &ScoringMove) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for ScoringMove {
    fn partial_cmp(&self, other: &ScoringMove) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScoringMove {
    fn eq(&self, other: &ScoringMove) -> bool {
        self.score == other.score
    }
}

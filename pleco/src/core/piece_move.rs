//! Module for the implementation and definition of a move to be played.
//!
//! We define a move as the struct `BitMove`. A move needs 16 bits to be stored, and
//! they are used as such:
//!
//! ```md,ignore
//! bits  0 - 5:  destination square (from 0 to 63)
//! bits  6 - 11: origin square (from 0 to 63)
//! bits 12 - 13: promotion piece type - 2 (from KNIGHT-2 to QUEEN-2)
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

use super::*;
use super::sq::SQ;
use std::fmt;

// Castles have the src as the king bit and the dst as the rook
static SRC_MASK: u16 = 0b0000_000000_111111;
static DST_MASK: u16 = 0b0000_111111_000000;
static PR_MASK: u16 = 0b1000_000000_000000;
static CP_MASK: u16 = 0b0100_000000_000000;
static FLAG_MASK: u16 = 0b1111_000000_000000;
static SP_MASK: u16 = 0b0011_000000_000000;

/// Represents a singular move. 
///
/// A `BitMove` consists of 16 bits, all of which to include a source square, destination square,
/// and special move-flags to differentiate types of moves. 
///
/// A `BitMove` should never be created directly, but rather instigated with a `PreMoveInfo`. This is because
/// the bits are in a special order, and manually creating moves risks creating an invalid move.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BitMove {
    data: u16,
}

/// Selected Meta-Data to accompany each move.
#[derive(Copy, Clone, PartialEq)]
pub enum MoveFlag {
    Promotion { capture: bool, prom: Piece },
    Castle { king_side: bool },
    DoublePawnPush,
    Capture { ep_capture: bool },
    QuietMove,
}

/// A Subset of `MoveFlag`, used to determine the overall classification of a move.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MoveType {
    Promotion,
    Castle,
    EnPassant,
    Normal,
}

/// Useful pre-encoding of a move's information before it is compressed into a `BitMove` struct.
#[derive(Copy, Clone, PartialEq)]
pub struct PreMoveInfo {
    pub src: SQ,
    pub dst: SQ,
    pub flags: MoveFlag,
}

impl fmt::Display for BitMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}


// https://chessprogramming.wikispaces.com/Encoding+Moves
impl BitMove {

    /// Creates a new BitMove from raw bits.
    ///
    /// # Safety
    ///
    /// Using this method cannot guarantee that the move is legal. The input bits must be encoding a legal
    /// move, or else there is Undefined Behavior if the resulting BitMove is used.
    pub fn new(input: u16) -> BitMove {
        BitMove { data: input }
    }

    /// Creates a BitMove from a [PreMoveInfo].
    #[inline]
    pub fn init(info: PreMoveInfo) -> BitMove {
        let src = info.src.0 as u16;
        let dst = (info.dst.0 as u16) << 6;
        let flags = info.flags;
        let flag_bits: u16 = match flags {
            MoveFlag::Promotion { capture, prom } => {
                let p_bit: u16 = match prom {
                    Piece::R => 2,
                    Piece::B => 1,
                    Piece::N => 0,
                    Piece::Q | _ => 3,
                };
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
        BitMove { data: (flag_bits << 12) | src | dst }
    }

    /// Creates a Null Move.
    ///
    /// # Safety
    ///
    /// A Null move is never a valid move to play. Using a Null move should onl be used for search and
    /// evaluation purposes. 
    pub fn null() -> Self {
        BitMove { data: 0 }
    }

    /// Returns if a [BitMove] is a Null Move.
    ///
    /// See [BitMove::null()] for more information on Null moves.
    pub fn is_null(&self) -> bool {
        self.data == 0
    }

    /// Returns if a [BitMove] captures an opponent's piece.
    #[inline(always)]
    pub fn is_capture(&self) -> bool {
        ((self.data & CP_MASK) >> 14) == 1
    }

    /// Returns if a [BitMove] is a Quiet Move, meaning it is not any of the following: a capture, promotion, castle, or double pawn push.
    #[inline(always)]
    pub fn is_quiet_move(&self) -> bool {
        ((self.data & FLAG_MASK) >> 12) == 0
    }

    /// Returns if a [BitMove] is a promotion.
    #[inline(always)]
    pub fn is_promo(&self) -> bool {
        (self.data & PR_MASK) != 0
    }

    /// Returns the destination of a [BitMove].
    #[inline(always)]
    pub fn get_dest(&self) -> SQ {
        SQ(self.get_dest_u8())
    }

    /// Returns the destination of a [BitMove].
    #[inline(always)]
    pub fn get_dest_u8(&self) -> u8 {
        ((self.data & DST_MASK) >> 6) as u8
    }

    /// Returns the source square of a [BitMove].
    #[inline(always)]
    pub fn get_src(&self) -> SQ {
        SQ(self.get_src_u8())
    }

    /// Returns the source square of a [BitMove].
    #[inline(always)]
    pub fn get_src_u8(&self) -> u8 {
        (self.data & SRC_MASK) as u8
    }

    /// Returns if a [BitMove] is a castle.
    #[inline(always)]
    pub fn is_castle(&self) -> bool {
        ((self.data & FLAG_MASK) >> 13) == 1
    }

    /// Returns if a [BitMove] is a Castle && it is a KingSide Castle.
    #[inline(always)]
    pub fn is_king_castle(&self) -> bool {
        ((self.data & FLAG_MASK) >> 12) == 2
    }

    /// Returns if a [BitMove] is a Castle && it is a QueenSide Castle.
    #[inline(always)]
    pub fn is_queen_castle(&self) -> bool {
        ((self.data & FLAG_MASK) >> 12) == 3
    }

    /// Returns if a [BitMove] is an enpassant capture.
    #[inline(always)]
    pub fn is_en_passant(&self) -> bool {
        (self.data & FLAG_MASK) >> 12 == 5
    }

    /// Returns if a [BitMove] is a double push, and if so returns the Destination square as well.
    #[inline(always)]
    pub fn is_double_push(&self) -> (bool, u8) {
        let is_double_push: u8 = ((self.data & FLAG_MASK) >> 12) as u8;
        match is_double_push {
            1 => (true, self.get_dest().0 as u8),
            _ => (false, 64),
        }
    }

    /// Returns the `Rank` (otherwise known as row) that the destination square  of a `BitMove`
    /// lies on.
    #[inline(always)]
    pub fn dest_row(&self) -> Rank {
//        ALL_RANKS[(((self.data & DST_MASK) >> 6) as u8 / 8) as usize]
        self.get_dest().rank()
    }

    /// Returns the `File` (otherwise known as column) that the destination square of a `BitMove`
    /// lies on.
    #[inline(always)]
    pub fn dest_col(&self) -> File {
        self.get_dest().file()
    }

    /// Returns the `Rank` (otherwise known as row) that the from-square of a `BitMove` lies on.
    #[inline(always)]
    pub fn src_row(&self) -> Rank {
        self.get_src().rank()
    }

    /// Returns the `File` (otherwise known as column) that the from-square of a `BitMove` lies on.
    #[inline(always)]
    pub fn src_col(&self) -> File {
        self.get_src().file()
    }

    /// Returns the Promotion Piece of a [BitMove].
    ///
    /// # Safety
    ///
    /// Method should only be used if the [BitMove] is a promotion. Otherwise, Undefined Behavior may result.
    #[inline(always)]
    pub fn promo_piece(&self) -> Piece {
        match (self.data >> 12) & 0b0011 {
            0 => Piece::N,
            1 => Piece::B,
            2 => Piece::R,
            3 | _ => Piece::Q,
        }
    }

    /// Returns the [MoveType] of a [BitMove].
    #[inline(always)]
    pub fn move_type(&self) -> MoveType {
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

    /// Returns a String representation of a [BitMove].
    ///
    /// Format goes "Source Square, Destination Square, (Promo Piece)". Moving a Queen from A1 to B8
    /// will stringify to "a1b8". If there is a pawn promotion involved, the piece promoted to will be
    /// appended to the end of the string, alike "a7a8q" in the case of a queen promotion.
    pub fn stringify(&self) -> String {
        let src = self.get_src().to_string();
        let dst = self.get_dest().to_string();
        let mut s = format!("{}{}", src, dst);
        if self.is_promo() {
            let c = self.promo_piece().char_lower();
            s.push(c);
        }
        s
    }

    /// Returns the raw number representation of the move.
    pub fn get_raw(&self) -> u16 {
        self.data
    }
}

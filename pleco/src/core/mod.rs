//! Contains various components and structures supporting the creation of a chessboard. This
//! includes `SQ`, `BitBoard`, `Player`, `Piece`, `GenTypes`, `Rank`, and `File`.

#[macro_use]
mod macros;

pub mod bit_twiddles;
pub mod bitboard;
pub mod masks;
pub mod mono_traits;
pub mod move_list;
pub mod piece_move;
pub mod score;
pub mod sq;

use self::bit_twiddles::*;
use self::bitboard::BitBoard;
use self::masks::*;
use self::sq::SQ;

use std::fmt;
use std::mem;
use std::ops::Not;

/// Array of all possible pieces, indexed by their enum value.
pub const ALL_PIECE_TYPES: [PieceType; PIECE_TYPE_CNT - 2] = [
    PieceType::P,
    PieceType::N,
    PieceType::B,
    PieceType::R,
    PieceType::Q,
    PieceType::K,
];

/// Array of both players, indexed by their enum value.
pub const ALL_PLAYERS: [Player; 2] = [Player::White, Player::Black];

/// Array of all `Files`s, indexed by their enum value.
pub static ALL_FILES: [File; FILE_CNT] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

/// Array of all `Rank`s, indexed by their enum value.
pub static ALL_RANKS: [Rank; RANK_CNT] = [
    Rank::R1,
    Rank::R2,
    Rank::R3,
    Rank::R4,
    Rank::R5,
    Rank::R6,
    Rank::R7,
    Rank::R8,
];

/// Enum to represent the Players White & Black.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    /// Returns the other player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::Player;
    ///
    /// let b = Player::Black;
    /// assert_eq!(b.other_player(), Player::White);
    /// ```
    #[inline(always)]
    pub fn other_player(self) -> Player {
        !(self)
    }

    /// Returns the relative square from a given square.
    #[inline(always)]
    pub fn relative_square(self, sq: SQ) -> SQ {
        assert!(sq.is_okay());
        sq ^ SQ((self) as u8 * 56)
    }

    /// Gets the direction of a pawn push for a given player.
    #[inline(always)]
    pub fn pawn_push(self) -> i8 {
        match self {
            Player::White => NORTH,
            Player::Black => SOUTH,
        }
    }

    /// Returns the relative rank of a square in relation to a player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::{Player,Rank};
    /// use pleco::core::sq::SQ;
    ///
    /// let w = Player::White;
    /// let b = Player::Black;
    ///
    /// assert_eq!(w.relative_rank_of_sq(SQ::A1), Rank::R1);
    /// assert_eq!(b.relative_rank_of_sq(SQ::H8), Rank::R1);
    /// assert_eq!(b.relative_rank_of_sq(SQ::A1), Rank::R8);
    /// ```
    #[inline(always)]
    pub fn relative_rank_of_sq(self, sq: SQ) -> Rank {
        self.relative_rank(sq.rank())
    }

    /// Returns the relative rank of a rank in relation to a player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::{Player,Rank};
    /// use pleco::core::sq::SQ;
    ///
    /// let w = Player::White;
    /// let b = Player::Black;
    ///
    /// assert_eq!(w.relative_rank(Rank::R1), Rank::R1);
    /// assert_eq!(b.relative_rank(Rank::R8), Rank::R1);
    /// assert_eq!(b.relative_rank(Rank::R1), Rank::R8);
    /// ```
    #[inline]
    pub fn relative_rank(self, rank: Rank) -> Rank {
        let r = (rank as u8) ^ (self as u8 * 7);
        debug_assert!(r < 8);
        //        ALL_RANKS[((rank as u8) ^ (*self as u8 * 7)) as usize]
        unsafe { mem::transmute::<u8, Rank>(r) }
    }
}

impl Not for Player {
    type Output = Player;

    fn not(self) -> Self::Output {
        let other: u8 = (self as u8) ^ 0b0000_0001;
        unsafe { mem::transmute(other) }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if self == &Player::White {
                "White"
            } else {
                "Black"
            }
        )
    }
}

/// Types of move generating options.
///
/// `GenTypes::All` -> All available moves.
///
/// `GenTypes::Captures` -> All captures and both capture/non-capture promotions.
///
/// `GenTypes::Quiets` -> All non captures and both capture/non-capture promotions.
///
/// `GenTypes::QuietChecks` -> Moves likely to give check.
///
/// `GenTypes::Evasions` -> Generates evasions for a board in check.
///
/// `GenTypes::NonEvasions` -> Generates all moves for a board not in check.
///
/// # Safety
///
/// `GenTypes::QuietChecks` and `GenTypes::NonEvasions` can only be used if the board
/// if not in check, while `GenTypes::Evasions` can only be used if the the board is
/// in check. The remaining `GenTypes` can be used legally whenever.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GenTypes {
    All,
    Captures,
    Quiets,
    QuietChecks,
    Evasions,
    NonEvasions,
}

/// All possible Types of Pieces on a chessboard.
///
/// For a representation of pieces considering color as well, see [`Piece`]
///
/// [`Piece`]: ./enum.Piece
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceType {
    None = 0,
    P = 1,
    N = 2,
    B = 3,
    R = 4,
    Q = 5,
    K = 6,
    All = 7,
}

impl PieceType {
    /// Returns the relative value of a piece.
    ///
    /// Used for sorting moves.
    #[inline]
    pub fn value(self) -> i8 {
        match self {
            PieceType::P => 1,
            PieceType::N | PieceType::B => 3,
            PieceType::R => 5,
            PieceType::Q => 8,
            _ => 0,
        }
    }

    /// Returns if the piece is `PieceType::None`
    #[inline(always)]
    pub fn is_none(self) -> bool {
        self == PieceType::None
    }

    /// Returns if the piece is not `PieceType::None`
    #[inline(always)]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }

    /// Checks if the piece is actually real, as in the Piece is not `None` or `All`.
    #[inline(always)]
    pub fn is_real(self) -> bool {
        self != PieceType::None && self != PieceType::All
    }

    /// Return the lowercase character of a `Piece`.
    #[inline]
    pub fn char_lower(self) -> char {
        match self {
            PieceType::P => 'p',
            PieceType::N => 'n',
            PieceType::B => 'b',
            PieceType::R => 'r',
            PieceType::Q => 'q',
            PieceType::K => 'k',
            _ => panic!(),
        }
    }

    /// Return the uppercase character of a `Piece`.
    #[inline]
    pub fn char_upper(self) -> char {
        match self {
            PieceType::P => 'P',
            PieceType::N => 'N',
            PieceType::B => 'B',
            PieceType::R => 'R',
            PieceType::Q => 'Q',
            PieceType::K => 'K',
            _ => panic!(),
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            PieceType::P => "Pawn",
            PieceType::N => "Knight",
            PieceType::B => "Bishop",
            PieceType::R => "Rook",
            PieceType::Q => "Queen",
            PieceType::K => "King",
            PieceType::All => "All",
            PieceType::None => "",
        };
        f.pad(s)
    }
}

// TODO: documentation

/// All possible Types of Pieces on a chessboard, for both colors.
///
/// For a representation of Only Pieces (with no color attached), see [`PieceType`]
///
/// [`Piece`]: ./enum.PieceType
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    None = 0b0000,
    WhitePawn = 0b0001,
    WhiteKnight = 0b0010,
    WhiteBishop = 0b0011,
    WhiteRook = 0b0100,
    WhiteQueen = 0b0101,
    WhiteKing = 0b0110,
    BlackPawn = 0b1001,
    BlackKnight = 0b1010,
    BlackBishop = 0b1011,
    BlackRook = 0b1100,
    BlackQueen = 0b1101,
    BlackKing = 0b1110,
}

impl Piece {
    /// Returns the `Player` of a piece, if any.
    ///
    /// For an unsafe, "lossy" determination of a `Player`, see [`Piece::player_lossy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,Player};
    ///
    /// let black_knight = Piece::BlackKnight;
    /// let player: Player = black_knight.player().unwrap();
    ///
    /// assert_eq!(player, Player::Black);
    /// ```
    ///
    /// The only discriminant that will return `None`:
    ///
    /// ```
    /// use pleco::{Piece,Player};
    ///
    /// let piece = Piece::None;
    /// assert!(piece.player().is_none());
    /// ```
    ///
    /// [`Piece::player_lossy`]: ./enum.Piece.html#method.player_lossy
    #[inline(always)]
    pub fn player(self) -> Option<Player> {
        if self as u8 & 0b0111 == 0 {
            None
        } else {
            Some(self.player_lossy())
        }
    }

    /// Returns the `Player` of a `Piece`.
    ///
    /// # Undefined Behavior
    ///
    /// If the discriminant is `Piece::None`, the returned `Player` will be undefined. This method
    /// must be used only when a returned `Piece` is guaranteed to exist.
    ///
    /// For a safer version of this method that accounts for an undetermined player,
    /// see [`Piece::player`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,Player};
    ///
    /// let white_pawn = Piece::WhitePawn;
    /// let player: Player = white_pawn.player_lossy();
    ///
    /// assert_eq!(player, Player::White);
    /// ```
    ///
    /// The following will invoke undefined behavior, so do not use it.
    ///
    /// ```
    /// use pleco::{Piece,Player};
    ///
    /// let piece = Piece::None;
    /// let fake_player: Player = piece.player_lossy();
    /// ```
    /// [`Piece::player`]: ./enum.Piece.html#method.player
    #[inline(always)]
    pub fn player_lossy(self) -> Player {
        unsafe { mem::transmute((self as u8 >> 3) & 0b1) }
    }

    /// Returns the `PieceType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,PieceType};
    ///
    /// let white_queen = Piece::WhiteQueen;
    /// let no_piece = Piece::None;
    ///
    /// assert_eq!(white_queen.type_of(), PieceType::Q);
    /// assert_eq!(no_piece.type_of(), PieceType::None);
    ///
    /// let black_queen = Piece::BlackQueen;
    /// assert_eq!(white_queen.type_of(), black_queen.type_of());
    /// ```
    #[inline(always)]
    pub fn type_of(self) -> PieceType {
        unsafe { mem::transmute(self as u8 & 0b111) }
    }

    /// Returns the `Player` and `PieceType` of this piece, if any. If the discriminant is
    /// `Piece::None`, `None` will be returned.
    ///
    /// For an unsafe, "lossy" determination of a `Player` and `Piece`,
    /// see [`Piece::player_piece_lossy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let white_queen = Piece::WhiteQueen;
    /// let (player, piece) = white_queen.player_piece().unwrap();
    ///
    /// assert_eq!(piece, PieceType::Q);
    /// assert_eq!(player, Player::White);
    /// ```
    ///
    /// [`Piece::player_piece_lossy`]: ./enum.Piece.html#method.player_piece_lossy
    #[inline(always)]
    pub fn player_piece(self) -> Option<(Player, PieceType)> {
        if self == Piece::None {
            None
        } else {
            Some(self.player_piece_lossy())
        }
    }

    /// Returns the `Player` and `PieceType` of a `Piece`.
    ///
    /// # Undefined Behavior
    ///
    /// If the discriminant is `Piece::None`, the returned `PieceType` will be correct, but
    /// the `Player` will be undefined.
    ///
    /// For a safer version of this method that accounts for an undetermined player,
    /// see [`Piece::player_piece`].
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let white_queen = Piece::WhiteQueen;
    /// let (player, piece) = white_queen.player_piece_lossy();
    ///
    /// assert_eq!(piece, PieceType::Q);
    /// assert_eq!(player, Player::White);
    /// ```
    ///
    /// Undefined behavior can be encountered by doing something akin to:
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let not_a_piece = Piece::None;
    ///
    /// let (wrong_player, piece) = not_a_piece.player_piece_lossy();
    /// ```
    ///
    /// [`Piece::player_piece`]: ./enum.Piece.html#method.player_piece
    #[inline(always)]
    pub fn player_piece_lossy(self) -> (Player, PieceType) {
        (self.player_lossy(), self.type_of())
    }

    /// Creates a `Piece` from a `Player` and `PieceType`. If the `PieceType` is either
    /// `PieceType::All` or `PieceType::None`, the returned value will be `None`.
    ///
    /// For an unsafe, lossy version of this method, see [`Piece::make_lossy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let black_knight = Piece::make(Player::Black, PieceType::N).unwrap();
    ///
    /// assert_eq!(black_knight.type_of(), PieceType::N);
    /// assert_eq!(black_knight.player().unwrap(), Player::Black);
    ///
    /// let illegal_piece = Piece::make(Player::White, PieceType::All);
    /// assert!(illegal_piece.is_none());
    /// ```
    /// [`Piece::make_lossy`]: ./enum.Piece.html#method.make_lossy
    #[inline(always)]
    pub fn make(player: Player, piece_type: PieceType) -> Option<Piece> {
        match piece_type {
            PieceType::None => Some(Piece::None),
            PieceType::All => None,
            _ => Some(Piece::make_lossy(player, piece_type)),
        }
    }

    /// Creates a `Piece` from a `Player` and `PieceType`.
    ///
    /// # Undefined Behavior
    ///
    /// If the `PieceType` is either `PieceType::All` or `PieceType::None`, undefined behavior will
    /// follow. See [`Piece::make`] for a safer version of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let black_knight = Piece::make_lossy(Player::Black, PieceType::N);
    ///
    /// assert_eq!(black_knight.type_of(), PieceType::N);
    /// assert_eq!(black_knight.player().unwrap(), Player::Black);
    /// ```
    ///
    /// The following code snippet will give undefined behavior:
    ///
    /// ```
    /// use pleco::{Piece,PieceType,Player};
    ///
    /// let illegal_piece = Piece::make_lossy(Player::Black, PieceType::All);
    /// ```
    /// [`Piece::make`]: ./enum.Piece.html#method.make
    #[inline(always)]
    pub fn make_lossy(player: Player, piece_type: PieceType) -> Piece {
        unsafe {
            let bits: u8 = (player as u8) << 3 | piece_type as u8;
            mem::transmute(bits)
        }
    }

    /// Returns the character of a `Piece`. If the Piece is `Piece::None`, `None` will be returned.
    #[inline]
    pub fn character(self) -> Option<char> {
        if self == Piece::None {
            None
        } else {
            Some(self.character_lossy())
        }
    }

    /// Returns the character of a `Piece`.
    ///
    /// # Panics
    ///
    /// If the Piece is `Piece::None`, a panic will occur.
    pub fn character_lossy(self) -> char {
        match self {
            Piece::None => panic!(),
            Piece::WhitePawn => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook => 'R',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteKing => 'K',
            Piece::BlackPawn => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook => 'r',
            Piece::BlackQueen => 'q',
            Piece::BlackKing => 'k',
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self != Piece::None {
            write!(f, "{}", self.character_lossy())
        } else {
            write!(f, "X")
        }
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Piece::None => "None",
            Piece::WhitePawn => "WhitePawn",
            Piece::WhiteKnight => "WhiteKnight",
            Piece::WhiteBishop => "WhiteBishop",
            Piece::WhiteRook => "WhiteRook",
            Piece::WhiteQueen => "WhiteQueen",
            Piece::WhiteKing => "WhiteKing",
            Piece::BlackPawn => "BlackPawn",
            Piece::BlackKnight => "BlackKnight",
            Piece::BlackBishop => "BlackBishop",
            Piece::BlackRook => "BlackRook",
            Piece::BlackQueen => "BlackQueen",
            Piece::BlackKing => "BlackKing",
        };
        write!(f, "{}", s)
    }
}

/// Enum for the Files of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, Ord, PartialOrd, Eq)]
pub enum File {
    A = 0, // eg a specific column
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    /// Returns the bit-set of all files to the left of the current file.
    #[inline]
    pub const fn left_side_mask(self) -> u8 {
        (1 << self as u8) - 1
    }

    /// Returns the bit-set of all files to the right of the current file.
    #[inline]
    pub const fn right_side_mask(self) -> u8 {
        !((1 << (self as u16 + 1)) - 1) as u8
    }

    /// Returns the minimum file.
    ///
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::File;
    ///
    /// let file_a = File::A;
    ///
    /// assert_eq!(file_a.min(File::C), File::A);
    /// ```
    #[inline]
    pub fn min(self, other: File) -> File {
        if (self as u8) < (other as u8) {
            self
        } else {
            other
        }
    }

    /// Returns the maximum file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::File;
    ///
    /// let file_a = File::A;
    ///
    /// assert_eq!(file_a.max(File::C), File::C);
    /// ```
    #[inline]
    pub fn max(self, other: File) -> File {
        if (self as u8) > (other as u8) {
            self
        } else {
            other
        }
    }

    /// Returns the distance to another `File`.
    pub fn distance(self, other: File) -> u8 {
        if self > other {
            self as u8 - other as u8
        } else {
            other as u8 - self as u8
        }
    }

    /// Returns the file `BitBoard`.
    pub fn bb(self) -> BitBoard {
        BitBoard(file_bb(self as u8))
    }
}

impl Not for File {
    type Output = File;

    fn not(self) -> File {
        unsafe {
            let f = self as u8 ^ File::H as u8;
            mem::transmute::<u8, File>(0b111 & f)
        }
    }
}

/// Enum for the Ranks of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, Eq, Ord, PartialOrd)]
pub enum Rank {
    // eg a specific row
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

impl Rank {
    /// Returns the distance to another `Rank`.
    pub fn distance(self, other: Rank) -> u8 {
        if self > other {
            self as u8 - other as u8
        } else {
            other as u8 - self as u8
        }
    }

    /// Returns the rank `BitBoard`.
    pub fn bb(self) -> BitBoard {
        BitBoard(rank_bb(self as u8))
    }
}

/// Types of Castling available to a player.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CastleType {
    KingSide = 0,
    QueenSide = 1,
}

#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Phase {
    MG = 0,
    EG = 1,
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding rank as a u64.
#[inline(always)]
pub fn rank_bb(s: u8) -> u64 {
    RANK_BB[rank_idx_of_sq(s) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank`.
#[inline(always)]
pub fn rank_of_sq(s: u8) -> Rank {
    unsafe { mem::transmute::<u8, Rank>((s >> 3) & 0b0000_0111) }
    //    ALL_RANKS[(s >> 3) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank` index.
#[inline(always)]
pub fn rank_idx_of_sq(s: u8) -> u8 {
    (s >> 3) as u8
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding file as a u64.
#[inline(always)]
pub fn file_bb(s: u8) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File`.
#[inline(always)]
pub fn file_of_sq(s: u8) -> File {
    unsafe { mem::transmute::<u8, File>(s & 0b0000_0111) }
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File` index.
#[inline(always)]
pub fn file_idx_of_sq(s: u8) -> u8 {
    (s & 0b0000_0111) as u8
}

/// Converts a singular bit of a u64 to it's index in the u64.
/// If there's more than one bit in the u64, this will be done for
/// the least significant bit.
///
/// # Safety
///
/// Undefined behavior if there are 0 bits in the input.
#[inline]
pub fn u64_to_u8(b: u64) -> u8 {
    debug_assert_eq!(popcount64(b), 1);
    bit_scan_forward(b)
}

/// Given a square (u8) that is valid, returns the bitboard representation
/// of that square.
///
/// # Safety
///
/// If the input is greater than 63, an empty u64 will be returned.
#[inline]
pub fn u8_to_u64(s: u8) -> u64 {
    debug_assert!(s < 64);
    1_u64.wrapping_shl(s as u32)
}

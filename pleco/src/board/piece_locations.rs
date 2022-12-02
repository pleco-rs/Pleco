//! Contains the `PieceLocations` structure that maps from squares of a board to a player / piece at that square.
//!
//! This is useful mainly for the [`Board`] to use internally for fast square lookups.
//!
//! [`Board`]: ../struct.Board.html
//! [`PieceLocations`]: struct.PieceLocations.html

use std::mem;

use super::FenBuildError;
use core::masks::*;
use core::sq::SQ;
use core::*;

/// Struct to allow fast lookups for any square. Given a square, allows for determining if there
/// is a piece currently there, and if so, allows for determining it's color and type of piece.
///
/// Piece Locations is a BLIND structure, Providing a function of  |sq| -> |Piece AND/OR Player|
/// The reverse cannot be done Looking up squares from a piece / player.
pub struct PieceLocations {
    // Pieces are represented by the following bit_patterns:
    // x000 -> Pawn (P)
    // x001 -> Knight(N)
    // x010 -> Bishop (B)
    // x011 -> Rook(R)
    // x100 -> Queen(Q)
    // x101 -> King (K)
    // x110 -> ??? Undefined ??
    // x111 -> None
    // 0xxx -> White Piece
    // 1xxx -> Black Piece

    // array of u8's, with standard ordering mapping index to square
    data: [Piece; SQ_CNT],
}

impl PieceLocations {
    /// Constructs a new `PieceLocations` with a default of no pieces on the board.
    pub const fn blank() -> PieceLocations {
        PieceLocations {
            data: [Piece::None; 64],
        }
    }

    /// Places a given piece for a given player at a certain square.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63 or the piece is `PieceType::{None || All}`
    #[inline]
    pub fn place(&mut self, square: SQ, player: Player, piece: PieceType) {
        debug_assert!(square.is_okay());
        debug_assert!(piece.is_real());
        self.data[square.0 as usize] = Piece::make_lossy(player, piece);
    }

    /// Removes a Square.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63.
    #[inline]
    pub fn remove(&mut self, square: SQ) {
        debug_assert!(square.is_okay());
        self.data[square.0 as usize] = Piece::None;
    }

    /// Returns the Piece at a `SQ`, Or None if the square is empty.
    ///
    /// # Panics
    ///
    /// Panics if square is of index higher than 63.
    #[inline]
    pub fn piece_at(&self, square: SQ) -> Piece {
        debug_assert!(square.is_okay());
        self.data[square.0 as usize]
    }

    /// Returns if a square is occupied.
    #[inline]
    pub fn at_square(&self, square: SQ) -> bool {
        assert!(square.is_okay());
        self.data[square.0 as usize] != Piece::None
    }

    /// Returns the first square (if any) that a piece / player is at.
    #[inline]
    pub fn first_square(&self, piece: PieceType, player: Player) -> Option<SQ> {
        let target = Piece::make_lossy(player, piece);
        for x in 0..64 {
            if target == self.data[x as usize] {
                return Some(SQ(x));
            }
        }
        None
    }

    /// Returns if the Board contains a particular piece / player.
    #[inline]
    pub fn contains(&self, piece: PieceType, player: Player) -> bool {
        self.first_square(piece, player).is_some()
    }

    /// Generates a `PieceLocations` from a partial fen. A partial fen is defined as the first part of a
    /// fen, where the piece positions are available.
    pub(crate) fn from_partial_fen(
        ranks: &[&str],
    ) -> Result<Vec<(SQ, Player, PieceType)>, FenBuildError> {
        let mut loc = Vec::with_capacity(64);
        for (i, rank) in ranks.iter().enumerate() {
            let min_sq = (7 - i) * 8;
            let max_sq = min_sq + 7;
            let mut idx = min_sq;
            for ch in rank.chars() {
                if idx < min_sq {
                    return Err(FenBuildError::SquareSmallerRank {
                        rank: i,
                        square: SQ(idx as u8).to_string(),
                    });
                } else if idx > max_sq {
                    return Err(FenBuildError::SquareLargerRank {
                        rank: i,
                        square: SQ(idx as u8).to_string(),
                    });
                }

                let dig = ch.to_digit(10);
                if let Some(digit) = dig {
                    idx += digit as usize;
                } else {
                    // if no space, then there is a piece here
                    let piece = match ch {
                        'p' | 'P' => PieceType::P,
                        'n' | 'N' => PieceType::N,
                        'b' | 'B' => PieceType::B,
                        'r' | 'R' => PieceType::R,
                        'q' | 'Q' => PieceType::Q,
                        'k' | 'K' => PieceType::K,
                        _ => return Err(FenBuildError::UnrecognizedPiece { piece: ch }),
                    };
                    let player = if ch.is_lowercase() {
                        Player::Black
                    } else {
                        Player::White
                    };
                    loc.push((SQ(idx as u8), player, piece));
                    idx += 1;
                }
            }
        }
        Ok(loc)
    }
}

impl Clone for PieceLocations {
    // Need to use transmute copy as [_;64] does not automatically implement Clone.
    fn clone(&self) -> PieceLocations {
        unsafe { mem::transmute_copy(&self.data) }
    }
}

impl PartialEq for PieceLocations {
    fn eq(&self, other: &PieceLocations) -> bool {
        for sq in 0..64 {
            if self.data[sq] != other.data[sq] {
                return false;
            }
        }
        true
    }
}

pub struct PieceLocationsIter {
    locations: PieceLocations,
    sq: SQ,
}

impl Iterator for PieceLocationsIter {
    type Item = (SQ, Piece);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let cur_sq = self.sq;
            if cur_sq >= SQ::NONE {
                return None;
            }
            let piece = self.locations.data[cur_sq.0 as usize];
            self.sq += SQ(1);
            if piece != Piece::None {
                return Some((cur_sq, piece));
            }
        }
    }
}

impl IntoIterator for PieceLocations {
    type Item = (SQ, Piece);
    type IntoIter = PieceLocationsIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        PieceLocationsIter {
            locations: self,
            sq: SQ(0),
        }
    }
}

#[cfg(test)]
mod test {
    use Board;

    #[test]
    fn stack_overflow_test() {
        let board = Board::start_pos();
        let piece_locations = board.get_piece_locations();
        let mut v = Vec::new();
        for (sq, _) in piece_locations {
            v.push(sq);
        }
        assert_eq!(v.len(), 32);
    }
}

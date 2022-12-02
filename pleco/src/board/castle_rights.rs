//! Module for the `Castling` structure, which helps provide an easy way for the
//! `Board` to keep track of the various castling rights available for each player.
//!
//! Alongside keeping track of castling rights, it also keeps track of if a player has castled.
//!
//! At it's core, a [`Castling`] is a simple u8 which sets bits for each possible castling right.
//! This is necessary to keep track of for a chess match due to determining future castlings.
//!
//! [`Castling`]: struct.Castling.html

use core::masks::*;
use core::*;
use std::fmt;

use core::sq::SQ;

const ALL_CASTLING: u8 = 0b0000_1111;

bitflags! {
    /// Structure to help with recognizing the various possibilities of castling.
    ///
    /// For internal use by the [`Board`] only.
    ///
    /// Keeps track of what sides are possible to castle from for each player.
    ///
    /// Does not guarantee that the player containing a castling bit can castle at that
    /// time. Rather marks that castling is a possibility, e.g. a Castling struct
    /// containing a bit marking WHITE_Q means that neither the White King or Queen-side
    /// rook has moved since the game started.
    ///
    /// [`Board`]: ../struct.Board.html
    pub struct Castling: u8 {
        const WHITE_K      = C_WHITE_K_MASK; // White has King-side Castling ability
        const WHITE_Q      = C_WHITE_Q_MASK; // White has Queen-side Castling ability
        const BLACK_K      = C_BLACK_K_MASK; // Black has King-side Castling ability
        const BLACK_Q      = C_BLACK_Q_MASK; // White has Queen-side Castling ability
        const WHITE_ALL    = Self::WHITE_K.bits // White can castle for both sides
                           | Self::WHITE_Q.bits;
        const BLACK_ALL    = Self::BLACK_K.bits // Black can castle for both sides
                           | Self::BLACK_Q.bits;
    }
}

impl Castling {
    /// Removes all castling possibility for a single player
    #[inline]
    pub fn remove_player_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= Self::BLACK_ALL.bits,
            Player::Black => self.bits &= Self::WHITE_ALL.bits,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn all_castling() -> Self {
        Castling { bits: ALL_CASTLING }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn empty_set() -> Self {
        Castling { bits: 0 }
    }

    /// Removes King-Side castling possibility for a single player
    #[inline]
    pub fn remove_king_side_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= !Self::WHITE_K.bits,
            Player::Black => self.bits &= !Self::BLACK_K.bits,
        }
    }

    /// Removes Queen-Side castling possibility for a single player
    #[inline]
    pub fn remove_queen_side_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= !Self::WHITE_Q.bits,
            Player::Black => self.bits &= !Self::BLACK_Q.bits,
        }
    }

    /// Returns if a player can castle for a given side
    #[inline]
    pub fn castle_rights(self, player: Player, side: CastleType) -> bool {
        match player {
            Player::White => match side {
                CastleType::KingSide => self.contains(Self::WHITE_K),
                CastleType::QueenSide => self.contains(Self::WHITE_Q),
            },
            Player::Black => match side {
                CastleType::KingSide => self.contains(Self::BLACK_K),
                CastleType::QueenSide => self.contains(Self::BLACK_Q),
            },
        }
    }

    #[inline]
    pub fn player_can_castle(self, player: Player) -> Castling {
        Castling {
            bits: self.bits & (Castling::WHITE_ALL.bits >> (2 * player as u16)),
        }
    }

    /// Returns if both players have lost their ability to castle
    #[inline]
    pub fn no_castling(self) -> bool {
        !self.contains(Castling::WHITE_K)
            && !self.contains(Castling::WHITE_Q)
            && !self.contains(Castling::BLACK_K)
            && !self.contains(Castling::BLACK_Q)
    }

    #[inline]
    pub fn update_castling(&mut self, to: SQ, from: SQ) -> u8 {
        let mask_change: u8 = to.castle_rights_mask() | from.castle_rights_mask();
        let to_return: u8 = self.bits & mask_change;
        self.bits &= !mask_change;
        to_return
    }

    /// Adds the Right to castle based on an `char`.
    ///
    /// ```md
    /// `K` -> Add White King-side Castling bit.
    /// `Q` -> Add White Queen-side Castling bit.
    /// `k` -> Add Black King-side Castling bit.
    /// `q` -> Add Black Queen-side Castling bit.
    /// `-` -> Do nothing.
    /// ```
    ///
    /// # Panics
    ///
    /// Panics of the char is not `K`, `Q`, `k`, `q`, or `-`.
    pub fn add_castling_char(&mut self, c: char) {
        self.bits |= match c {
            'K' => Castling::WHITE_K.bits,
            'Q' => Castling::WHITE_Q.bits,
            'k' => Castling::BLACK_K.bits,
            'q' => Castling::BLACK_Q.bits,
            '-' => 0,
            _ => panic!(),
        };
    }

    /// Returns a pretty String representing the castling state
    ///
    /// Used for FEN Strings, with (`K` | `Q`) representing white castling abilities,
    /// and (`k` | `q`) representing black castling abilities. If there are no bits set,
    /// returns a String containing "-".
    pub fn pretty_string(self) -> String {
        if self.no_castling() {
            "-".to_owned()
        } else {
            let mut s = String::default();
            if self.contains(Castling::WHITE_K) {
                s.push('K');
            }
            if self.contains(Castling::WHITE_Q) {
                s.push('Q');
            }

            if self.contains(Castling::BLACK_K) {
                s.push('k');
            }

            if self.contains(Castling::BLACK_Q) {
                s.push('q');
            }
            assert!(!s.is_empty());
            s
        }
    }
}

impl fmt::Display for Castling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn const_test() {
        let c = Castling::all();
        let c_const = Castling::all_castling();
        assert_eq!(c, c_const);
    }
}


use core::*;
use std::mem;
use core::sq::SQ;


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
    data: [u8; 64],
}


impl PieceLocations {
    /// Constructs a new Piece Locations with a defaulty of no pieces on the board
    pub fn blank() -> PieceLocations {
        PieceLocations { data: [0b0111; 64] }
    }

    /// Constructs a new Piece Locations with the memory at a default of Zeros
    ///
    /// This function is unsafe as Zeros represent Pawns, and therefore care mus be taken
    /// to iterate through every square and ensure the correct piece or lack of piece
    /// is placed
    pub unsafe fn default() -> PieceLocations {
        PieceLocations { data: [0; 64] }
    }

    /// Places a given piece for a given player at a certain square
    ///
    /// # Panics
    /// Panics if Square is of index higher than 63
    pub fn place(&mut self, square: SQ, player: Player, piece: Piece) {
        assert!(square.is_okay());
        self.data[square.0 as usize] = self.create_sq(player, piece);
    }

    /// Removes a Square
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn remove(&mut self, square: SQ) {
        assert!(square.is_okay());
        self.data[square.0 as usize] = 0b0111
    }

    /// Returns the Piece at a square, Or None if the square is empty.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63.
    pub fn piece_at(&self, square: SQ) -> Option<Piece> {
        assert!(square.is_okay());
        let byte: u8 = self.data[square.0 as usize] & 0b0111;
        match byte {
            0b0000 => Some(Piece::P),
            0b0001 => Some(Piece::N),
            0b0010 => Some(Piece::B),
            0b0011 => Some(Piece::R),
            0b0100 => Some(Piece::Q),
            0b0101 => Some(Piece::K),
            0b0110 => unreachable!(), // Undefined
            0b0111 => None,
            _ => unreachable!(),
        }
    }

    /// Returns the Piece at a square for a given player.
    ///
    /// If there is no piece at that square, or there is a piece of another player at that square,
    /// returns None.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn piece_at_for_player(&self, square: SQ, player: Player) -> Option<Piece> {
        let op = self.player_piece_at(square);
        if op.is_some() {
            let p = op.unwrap();
            if p.0 == player {
                Some(p.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the player (if any) is occupying a square
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn player_at(&self, square: SQ) -> Option<Player> {
        let byte: u8 = self.data[square.0 as usize];
        if byte == 0b0111 || byte == 0b1111 {
            return None;
        }
        if byte < 8 {
            Some(Player::White)
        } else {
            Some(Player::Black)
        }
    }

    /// Returns a Tuple of (Player,Piece) of the player and associated piece at a
    /// given square. Returns None if the square is unoccupied.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn player_piece_at(&self, square: SQ) -> Option<(Player, Piece)> {
        let byte: u8 = self.data[square.0 as usize];
        match byte {
            0b0000 => Some((Player::White, Piece::P)),
            0b0001 => Some((Player::White, Piece::N)),
            0b0010 => Some((Player::White, Piece::B)),
            0b0011 => Some((Player::White, Piece::R)),
            0b0100 => Some((Player::White, Piece::Q)),
            0b0101 => Some((Player::White, Piece::K)),
            0b0110 => unreachable!(), // Undefined
            0b0111 | 0b1111 => None,
            0b1000 => Some((Player::Black, Piece::P)),
            0b1001 => Some((Player::Black, Piece::N)),
            0b1010 => Some((Player::Black, Piece::B)),
            0b1011 => Some((Player::Black, Piece::R)),
            0b1100 => Some((Player::Black, Piece::Q)),
            0b1101 => Some((Player::Black, Piece::K)),
            0b1110 => unreachable!(), // Undefined
            _ => unreachable!(),
        }
    }



    /// Helper method to return the bit representation of a given piece and player
    fn create_sq(&self, player: Player, piece: Piece) -> u8 {
        let mut loc: u8 = match piece {
            Piece::P => 0b0000,
            Piece::N => 0b0001,
            Piece::B => 0b0010,
            Piece::R => 0b0011,
            Piece::Q => 0b0100,
            Piece::K => 0b0101,
        };
        if player == Player::Black {
            loc |= 0b1000;
        }
        loc
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
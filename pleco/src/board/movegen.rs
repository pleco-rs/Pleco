//! Module for generating moves from a `Board`. Allow for generating Legal and Pseudo-Legal moves
//! of various types.
//!
//! # Generation Types
//!
//! The Types of moves that can be generated from a [`Board`] are:
//!
//! `All`, `Captures`, `Quiets`, `QuietChecks`, `Evasions`, `NonEvasions`
//!
//! There are all derived from the [`GenTypes`] enum.
//!
//! Generating all moves is legal to do no matter the position. However, `Captures`, `Quiets`,
//! `QuietChecks`, and `NonEvasions` can only be done if the board is in NOT in check. Likewise,
//! `Evasions` can only be done when the board is currently in check.
//!
//! `All` will generating all moves, while any other option will generate all moves, except for under-promotions.
//!
//! # `Legal` vs. `PseudoLegal` Moves
//!
//! For the generation type, moves can either be generated to be Legal, Or Pseudo-Legal. A Legal
//! move is, for as the name implies, a legal move for the current side to play for a given position.
//! A Pseudo-Legal move is a move that is "likely" to be legal for the current position, but cannot
//! be guaranteed.
//!
//! Why would someone ever want to generate moves that might not be legal? Performance. Based on
//! some benchmarking, generating all Pseudo-Legal moves is around twice as fast as generating all
//! Legal moves. So, if you are fine with generating moves and then checking them post-generation
//! with a [`Board::legal_move`], then the performance boost is potentially worth it.
//!
//! # Examples
//!
//! Generating all legal moves:
//!
//! ```ignore
//! let moves: MoveList = board.generate_moves();
//! ```
//!
//! Generating all pseudo-legal moves:
//!
//! ```ignore
//! let moves: MoveList = board.generate_pseudolegal_moves();
//! ```
//!
//! Generating all pseudo-legal captures:
//!
//! ```ignore
//! let moves: MoveList = board.generate_moves_of_type(GenTypes::Captures);
//! ```
//!
//! [`GenTypes`]: ../../core/enum.GenTypes.html
//! [`Board`]: ../struct.Board.html
//! [`Board::legal_move`]: ../struct.Board.html#method.legal_move

use std::hint::unreachable_unchecked;
use std::mem;
use std::ops::Index;
use std::ptr;

use board::*;

use core::mono_traits::GenTypeTrait;
use core::move_list::{MVPushable, MoveList, ScoringMoveList};
use core::piece_move::{BitMove, MoveFlag, PreMoveInfo, ScoringMove};

use {BitBoard, PieceType, Player, SQ};

//                   Legal    PseudoLegal
//         All:  10,172 ns  |  9,636 ns
// NonEvasions:   8,381 ns  |  4,179 ns
//    Captures:   2,491 ns  |  2,230 ns
//      Quiets:   2,491 ns  |  4,506 n
// QuietChecks:   7,988 ns  |  3,411 ns
//    Evasions:   4,034 ns  |  2,689 ns
//
//
//      With Full Player MonoMorphization
//
//                   Legal    PseudoLegal
//         All:   9,275 ns  |  4,814 ns
// NonEvasions:   8,421 ns  |  4,179 ns
//    Captures:   2,550 ns  |  2,230 ns
//      Quiets:   2,491 ns  |  4,506 n
// QuietChecks:   6,124 ns  |  3,411 ns
//    Evasions:   3,930 ns  |  2,649 ns
//
// With Full Player MonoMorphization

/// Determines the if the moves generated are `PseudoLegal` or `Legal` moves.
/// PseudoLegal moves require that a move's legality is determined before applying
/// to a `Board`.
pub trait Legality {
    /// Returns if the only legal moves should be generated.
    fn gen_legal() -> bool;
}

/// Dummy Struct to represent the generation of `Legal` Moves.
pub struct Legal {}

/// Dummy Struct to represent the generation of `PseudoLegal` Moves.
pub struct PseudoLegal {}

impl Legality for Legal {
    fn gen_legal() -> bool {
        true
    }
}

impl Legality for PseudoLegal {
    fn gen_legal() -> bool {
        false
    }
}

// Pieces to generate moves with inter-changably
const STANDARD_PIECES: [PieceType; 4] = [PieceType::B, PieceType::N, PieceType::R, PieceType::Q];
const DEFAULT_MOVES_LENGTH: usize = 32;

/// Public move generator.
///
/// This is a wrapper type around `InnerMoveGen`, allowing for a more friendly API
pub struct MoveGen {}

impl MoveGen {
    /// Returns `MoveList` of all moves for a given board, Legality & GenType.
    #[inline]
    pub fn generate<L: Legality, G: GenTypeTrait>(chessboard: &Board) -> MoveList {
        let mut movelist = MoveList::default();
        unsafe {
            let ptr: *mut BitMove = movelist.as_mut_ptr();
            let new_ptr = InnerMoveGen::<MoveList>::generate::<L, G>(chessboard, ptr);
            let new_size = (new_ptr as usize - ptr as usize) / mem::size_of::<BitMove>();
            movelist.unchecked_set_len(new_size);
        }
        movelist
    }

    /// Returns a `ScoringMoveList` of all moves for a given board, Legality & GenType.
    #[inline]
    pub fn generate_scoring<L: Legality, G: GenTypeTrait>(chessboard: &Board) -> ScoringMoveList {
        let mut movelist = ScoringMoveList::default();
        unsafe {
            let ptr: *mut ScoringMove = movelist.as_mut_ptr();
            let new_ptr = InnerMoveGen::<ScoringMoveList>::generate::<L, G>(chessboard, ptr);
            let new_size = (new_ptr as usize - ptr as usize) / mem::size_of::<ScoringMove>();
            movelist.unchecked_set_len(new_size);
        }
        movelist
    }

    /// Extends the current list of moves of a certain Legality, and Generation type. This method
    /// will correctly set the new length of the list.
    ///
    /// # Safety
    ///
    /// Unsafe due to possible overwriting, as it is unaware of the current list's length.
    #[inline]
    pub unsafe fn extend<L: Legality, G: GenTypeTrait, MP: MVPushable>(
        chessboard: &Board,
        movelist: &mut MP,
    ) where
        <MP as Index<usize>>::Output: Sized,
    {
        let begin: *mut MP::Output = movelist.list_ptr();
        let ptr: *mut MP::Output = movelist.over_bounds_ptr();
        let new_ptr: *mut MP::Output = InnerMoveGen::<MP>::generate::<L, G>(chessboard, ptr);
        let new_size = (new_ptr as usize - begin as usize) / mem::size_of::<MP::Output>();
        movelist.unchecked_set_len(new_size);
    }

    /// Extends the current list of moves of a certain Legality, and Generation type. Takes in a pointer
    /// to the next available (empty) index, and returns the pointer to the next open index after generating
    /// the moves
    ///
    /// # Safety
    ///
    /// Obviously, this is extremely unsafe to use, as there is a possibility of both overwriting valid memory,
    /// or otherwise pushing in an invalid pointer.
    ///
    /// Also, this does not update the size of the movelist inputted. It's recommended to use the method
    /// `MVPushable::unchecked_set_len(...)` to set the size manually after this method.
    #[inline(always)]
    pub unsafe fn extend_from_ptr<L: Legality, G: GenTypeTrait, MP: MVPushable>(
        chessboard: &Board,
        ptr: *mut MP::Output,
    ) -> *mut MP::Output
    where
        <MP as Index<usize>>::Output: Sized,
    {
        InnerMoveGen::<MP>::generate::<L, G>(chessboard, ptr)
    }
}

/// Structure to generate moves from. Stores the current state of the board, and other
/// references to help generating all possible moves. This structure shouldn't be used
/// normally.
struct InnerMoveGen<'a, MP: MVPushable + 'a> {
    ptr: *mut MP::Output,
    board: &'a Board,
    occ: BitBoard,
    // Squares occupied by all
    us_occ: BitBoard,
    // squares occupied by player to move
    them_occ: BitBoard, // Squares occupied by the opposing player
}

impl<'a, MP: MVPushable> InnerMoveGen<'a, MP>
where
    <MP as Index<usize>>::Output: Sized,
{
    /// Returns a pointer to the last element of all moves for a given board, Legality & GenType.
    #[inline(always)]
    fn generate<L: Legality, G: GenTypeTrait>(
        chessboard: &Board,
        movelist: *mut MP::Output,
    ) -> *mut MP::Output {
        match chessboard.turn() {
            Player::White => {
                InnerMoveGen::<MP>::generate_helper::<L, G, WhiteType>(chessboard, movelist)
            }
            Player::Black => {
                InnerMoveGen::<MP>::generate_helper::<L, G, BlackType>(chessboard, movelist)
            }
        }
    }

    // Helper function to setup the MoveGen structure.
    #[inline(always)]
    fn get_self(chessboard: &'a Board, ptr: *mut MP::Output) -> Self {
        InnerMoveGen {
            ptr,
            board: chessboard,
            occ: chessboard.occupied(),
            us_occ: chessboard.get_occupied_player(chessboard.turn()),
            them_occ: chessboard.get_occupied_player(chessboard.turn().other_player()),
        }
    }

    /// Directly generates the moves.
    fn generate_helper<L: Legality, G: GenTypeTrait, P: PlayerTrait>(
        chessboard: &Board,
        ptr: *mut MP::Output,
    ) -> *mut MP::Output {
        let mut movegen = InnerMoveGen::<MP>::get_self(chessboard, ptr);
        let gen_type = G::gen_type();
        if gen_type == GenTypes::Evasions {
            movegen.generate_evasions::<L, P>();
        } else if gen_type == GenTypes::QuietChecks {
            movegen.generate_quiet_checks::<L, P>();
        } else if gen_type == GenTypes::All {
            if movegen.board.in_check() {
                movegen.generate_evasions::<L, P>();
            } else {
                movegen.generate_non_evasions::<L, NonEvasionsGenType, P>();
            }
        } else {
            movegen.generate_non_evasions::<L, G, P>();
        }
        movegen.ptr
    }

    /// Generates non-evasions, ala the board is in check.
    fn generate_non_evasions<L: Legality, G: GenTypeTrait, P: PlayerTrait>(&mut self) {
        assert_ne!(G::gen_type(), GenTypes::All);
        assert_ne!(G::gen_type(), GenTypes::QuietChecks);
        assert_ne!(G::gen_type(), GenTypes::Evasions);
        assert!(!self.board.in_check());

        // target = bitboard of squares the generator should aim for
        let target: BitBoard = match G::gen_type() {
            GenTypes::NonEvasions => !self.us_occ,
            GenTypes::Captures => self.them_occ,
            GenTypes::Quiets => !(self.us_occ | self.them_occ),
            _ => unsafe { unreachable_unchecked() },
        };

        self.generate_all::<L, G, P>(target);
    }

    /// Generates all moves of a certain legality, `GenType`, and player. The target is the
    /// bitboard of the squares where moves should be generated.
    fn generate_all<L: Legality, G: GenTypeTrait, P: PlayerTrait>(&mut self, target: BitBoard) {
        self.generate_pawn_moves::<L, G, P>(target);
        self.moves_per_piece::<L, P, KnightType>(target);
        self.moves_per_piece::<L, P, BishopType>(target);
        self.moves_per_piece::<L, P, RookType>(target);
        self.moves_per_piece::<L, P, QueenType>(target);

        if G::gen_type() != GenTypes::QuietChecks && G::gen_type() != GenTypes::Evasions {
            self.moves_per_piece::<L, P, KingType>(target);
        }

        if G::gen_type() != GenTypes::Captures
            && G::gen_type() != GenTypes::Evasions
            && (self.board.can_castle(P::player(), CastleType::KingSide)
                || self.board.can_castle(P::player(), CastleType::QueenSide))
        {
            self.generate_castling::<L, P>();
        }
    }

    /// Generates quiet checks.
    fn generate_quiet_checks<L: Legality, P: PlayerTrait>(&mut self) {
        assert!(!self.board.in_check());
        let mut disc_check: BitBoard = self.board.discovered_check_candidates();
        let target: BitBoard = !self.board.occupied();

        // discovered check candidates
        while let Some(from) = disc_check.pop_some_lsb() {
            let piece: PieceType = self.board.piece_at_sq(from).type_of();
            if piece != PieceType::P {
                let mut b: BitBoard = self.moves_bb(piece, from) & target;
                if piece == PieceType::K {
                    b &= queen_moves(BitBoard(0), self.board.king_sq(P::opp_player()))
                }
                self.move_append_from_bb_flag::<L>(&mut b, from, BitMove::FLAG_QUIET);
            }
        }
        self.generate_all::<L, QuietChecksGenType, P>(target);
    }

    // Helper function to generate evasions
    fn generate_evasions<L: Legality, P: PlayerTrait>(&mut self) {
        assert!(self.board.in_check());

        let ksq: SQ = self.board.king_sq(P::player());
        let mut slider_attacks: BitBoard = BitBoard(0);

        // Pieces that could possibly attack the king with sliding attacks
        let mut sliders: BitBoard = self.board.checkers()
            & !self
                .board
                .piece_two_bb_both_players(PieceType::P, PieceType::N);

        // This is getting all the squares that are attacked by sliders
        while let Some((check_sq, check_sq_bb)) = sliders.pop_some_lsb_and_bit() {
            slider_attacks |= line_bb(check_sq, ksq) ^ check_sq_bb;
        }

        // Possible king moves, Where the king cannot move into a slider / own pieces
        let k_moves: BitBoard = king_moves(ksq) & !slider_attacks & !self.us_occ;

        // Separate captures and non captures
        let mut captures_bb: BitBoard = k_moves & self.them_occ;
        let mut non_captures_bb: BitBoard = k_moves & !self.them_occ;
        self.move_append_from_bb_flag::<L>(&mut captures_bb, ksq, BitMove::FLAG_CAPTURE);
        self.move_append_from_bb_flag::<L>(&mut non_captures_bb, ksq, BitMove::FLAG_QUIET);

        // If there is only one checking square, we can block or capture the piece
        if !(self.board.checkers().more_than_one()) {
            let checking_sq: SQ = self.board.checkers().bit_scan_forward();

            // Squares that allow a block or capture of the sliding piece
            let target: BitBoard = between_bb(checking_sq, ksq) | checking_sq.to_bb();
            self.generate_all::<L, EvasionsGenType, P>(target);
        }
    }

    // Generate king moves with a given target
    fn generate_king_moves<L: Legality, P: PlayerTrait>(&mut self, target: BitBoard) {
        self.moves_per_piece::<L, P, KingType>(target);
    }

    // Generates castling for both sides
    fn generate_castling<L: Legality, P: PlayerTrait>(&mut self) {
        self.castling_side::<L, P>(CastleType::QueenSide);
        self.castling_side::<L, P>(CastleType::KingSide);
    }

    // Generates castling for a single side
    fn castling_side<L: Legality, P: PlayerTrait>(&mut self, side: CastleType) {
        // Make sure we can castle AND the space between the king / rook is clear AND the piece at castling_side is a Rook
        if !self.board.castle_impeded(side)
            && self.board.can_castle(P::player(), side)
            && self
                .board
                .piece_at_sq(self.board.castling_rook_square(side))
                .type_of()
                == PieceType::R
        {
            let king_side: bool = { side == CastleType::KingSide };

            let ksq: SQ = self.board.king_sq(P::player());
            let r_from: SQ = self.board.castling_rook_square(side);
            let k_to = P::player().relative_square(if king_side { SQ::G1 } else { SQ::C1 });

            let enemies: BitBoard = self.them_occ;
            let direction: fn(SQ) -> SQ = if king_side {
                |x: SQ| x - SQ(1)
            } else {
                |x: SQ| x + SQ(1)
            };

            let mut s: SQ = k_to;
            let mut can_castle: bool = true;

            // Loop through all the squares the king goes through
            // If any enemies attack that square, cannot castle
            'outer: while s != ksq {
                let attackers: BitBoard = self.board.attackers_to(s, self.occ) & enemies;
                if attackers.is_not_empty() {
                    can_castle = false;
                    break 'outer;
                }
                s = direction(s);
            }
            if can_castle {
                self.check_and_add::<L>(BitMove::init(PreMoveInfo {
                    src: ksq,
                    dst: r_from,
                    flags: MoveFlag::Castle { king_side },
                }));
            }
        }
    }

    // Generate non-pawn and non-king moves for a target
    fn gen_non_pawn_king<L: Legality, P: PlayerTrait>(&mut self, target: BitBoard) {
        self.moves_per_piece::<L, P, KnightType>(target);
        self.moves_per_piece::<L, P, BishopType>(target);
        self.moves_per_piece::<L, P, RookType>(target);
        self.moves_per_piece::<L, P, QueenType>(target);
    }

    // Get the captures and non-captures for a piece
    fn moves_per_piece<L: Legality, PL: PlayerTrait, P: PieceTrait>(&mut self, target: BitBoard) {
        let mut piece_bb: BitBoard = self.board.piece_bb(PL::player(), P::piece_type());
        while let Some(src) = piece_bb.pop_some_lsb() {
            let moves_bb: BitBoard = self.moves_bb2::<P>(src) & !self.us_occ & target;
            let mut captures_bb: BitBoard = moves_bb & self.them_occ;
            let mut non_captures_bb: BitBoard = moves_bb & !self.them_occ;
            self.move_append_from_bb_flag::<L>(&mut captures_bb, src, BitMove::FLAG_CAPTURE);
            self.move_append_from_bb_flag::<L>(&mut non_captures_bb, src, BitMove::FLAG_QUIET);
        }
    }

    // Generate pawn moves
    fn generate_pawn_moves<L: Legality, G: GenTypeTrait, P: PlayerTrait>(
        &mut self,
        target: BitBoard,
    ) {
        let (rank_8, rank_7, rank_3): (BitBoard, BitBoard, BitBoard) =
            if P::player() == Player::White {
                (BitBoard::RANK_8, BitBoard::RANK_7, BitBoard::RANK_3)
            } else {
                (BitBoard::RANK_1, BitBoard::RANK_2, BitBoard::RANK_6)
            };

        let all_pawns: BitBoard = self.board.piece_bb(P::player(), PieceType::P);

        let mut empty_squares = BitBoard(0);

        // separate these two for promotion moves and non promotions
        let pawns_rank_7: BitBoard = all_pawns & rank_7;
        let pawns_not_rank_7: BitBoard = all_pawns & !rank_7;

        let enemies: BitBoard = if G::gen_type() == GenTypes::Evasions {
            self.them_occ & target
        } else if G::gen_type() == GenTypes::Captures {
            target
        } else {
            self.them_occ
        };

        // Single and Double Pawn Pushes
        if G::gen_type() != GenTypes::Captures {
            empty_squares =
                if G::gen_type() == GenTypes::Quiets || G::gen_type() == GenTypes::QuietChecks {
                    target
                } else {
                    !self.board.occupied()
                };

            let mut push_one: BitBoard = empty_squares & P::shift_up(pawns_not_rank_7);
            let mut push_two: BitBoard = P::shift_up(push_one & rank_3) & empty_squares;

            if G::gen_type() == GenTypes::Evasions {
                push_one &= target;
                push_two &= target;
            }

            if G::gen_type() == GenTypes::QuietChecks {
                let ksq: SQ = self.board.king_sq(P::opp_player());
                push_one &= pawn_attacks_from(ksq, P::opp_player());
                push_two &= pawn_attacks_from(ksq, P::opp_player());

                let dc_candidates: BitBoard = self.board.discovered_check_candidates();
                if (pawns_not_rank_7 & dc_candidates).is_not_empty() {
                    let dc1: BitBoard = P::shift_up(pawns_not_rank_7 & dc_candidates)
                        & empty_squares
                        & !ksq.file_bb();
                    let dc2: BitBoard = P::shift_up(rank_3 & dc1) & empty_squares;

                    push_one |= dc1;
                    push_two |= dc2;
                }
            }

            while let Some(dst) = push_one.pop_some_lsb() {
                let src: SQ = P::down(dst);
                self.check_and_add::<L>(BitMove::make_quiet(src, dst));
            }

            while let Some(dst) = push_two.pop_some_lsb() {
                let src: SQ = P::down(P::down(dst));
                self.check_and_add::<L>(BitMove::make_pawn_push(src, dst));
            }
        }

        // Promotions
        if pawns_rank_7.is_not_empty()
            && G::gen_type() != GenTypes::Quiets
            && (G::gen_type() != GenTypes::Evasions || (target & rank_8).is_not_empty())
        {
            if G::gen_type() == GenTypes::Captures {
                empty_squares = !self.occ;
            } else if G::gen_type() == GenTypes::Evasions {
                empty_squares &= target;
            }

            let mut no_promo: BitBoard = P::shift_up(pawns_rank_7) & empty_squares;
            let mut left_cap_promo: BitBoard = P::shift_up_left(pawns_rank_7) & enemies;
            let mut right_cap_promo: BitBoard = P::shift_up_right(pawns_rank_7) & enemies;

            while let Some(dst) = no_promo.pop_some_lsb() {
                self.create_all_non_cap_promos::<L>(dst, P::down(dst));
            }

            if G::gen_type() != GenTypes::Quiets {
                while let Some(dst) = left_cap_promo.pop_some_lsb() {
                    self.create_all_cap_promos::<L>(dst, P::down_right(dst));
                }

                while let Some(dst) = right_cap_promo.pop_some_lsb() {
                    self.create_all_cap_promos::<L>(dst, P::down_left(dst));
                }
            }
        }

        // Captures
        if G::gen_type() == GenTypes::Captures
            || G::gen_type() == GenTypes::Evasions
            || G::gen_type() == GenTypes::NonEvasions
            || G::gen_type() == GenTypes::All
        {
            let mut left_cap: BitBoard = P::shift_up_left(pawns_not_rank_7) & enemies;
            let mut right_cap: BitBoard = P::shift_up_right(pawns_not_rank_7) & enemies;

            while let Some(dst) = left_cap.pop_some_lsb() {
                let src: SQ = P::down_right(dst);
                self.check_and_add::<L>(BitMove::make_capture(src, dst));
            }

            while let Some(dst) = right_cap.pop_some_lsb() {
                let src: SQ = P::down_left(dst);
                self.check_and_add::<L>(BitMove::make_capture(src, dst));
            }

            if self.board.ep_square() != NO_SQ {
                let ep_sq: SQ = self.board.ep_square();
                assert_eq!(ep_sq.rank(), P::player().relative_rank(Rank::R6));

                // An en passant capture can be an evasion only if the checking piece
                // is the double pushed pawn and so is in the target. Otherwise this
                // is a discovery check and we are forced to do otherwise.
                if G::gen_type() != GenTypes::Evasions
                    || (target & P::down(ep_sq).to_bb()).is_not_empty()
                {
                    left_cap = pawns_not_rank_7 & pawn_attacks_from(ep_sq, P::opp_player());

                    while let Some(src) = left_cap.pop_some_lsb() {
                        self.check_and_add::<L>(BitMove::make_ep_capture(src, ep_sq));
                    }
                }
            }
        }
    }

    #[inline]
    fn create_all_non_cap_promos<L: Legality>(&mut self, dst: SQ, src: SQ) {
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_N, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_B, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_R, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_Q, src, dst));
    }

    #[inline]
    fn create_all_cap_promos<L: Legality>(&mut self, dst: SQ, src: SQ) {
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_CAP_N, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_CAP_B, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_CAP_R, src, dst));
        self.check_and_add::<L>(BitMove::make(BitMove::FLAG_PROMO_CAP_Q, src, dst));
    }

    // Return the moves Bitboard
    #[inline]
    fn moves_bb(&self, piece: PieceType, square: SQ) -> BitBoard {
        debug_assert!(square.is_okay());
        debug_assert_ne!(piece, PieceType::P);
        match piece {
            PieceType::P => panic!(),
            PieceType::N => knight_moves(square),
            PieceType::B => bishop_moves(self.occ, square),
            PieceType::R => rook_moves(self.occ, square),
            PieceType::Q => queen_moves(self.occ, square),
            PieceType::K => king_moves(square),
            _ => BitBoard(0),
        }
    }

    // Note: Does including this truly decrease MoveGen timing?

    /// Return the moves Bitboard
    #[inline]
    fn moves_bb2<P: PieceTrait>(&self, square: SQ) -> BitBoard {
        debug_assert!(square.is_okay());
        debug_assert_ne!(P::piece_type(), PieceType::P);
        match P::piece_type() {
            PieceType::P => panic!(),
            PieceType::N => knight_moves(square),
            PieceType::B => bishop_moves(self.occ, square),
            PieceType::R => rook_moves(self.occ, square),
            PieceType::Q => queen_moves(self.occ, square),
            PieceType::K => king_moves(square),
            _ => BitBoard(0),
        }
    }

    #[inline]
    fn move_append_from_bb_flag<L: Legality>(
        &mut self,
        bits: &mut BitBoard,
        src: SQ,
        flag_bits: u16,
    ) {
        while let Some(dst) = bits.pop_some_lsb() {
            let b_move = BitMove::make(flag_bits, src, dst);
            self.check_and_add::<L>(b_move);
        }
    }

    /// Checks if the move is legal, and if so adds to the move list.
    #[inline]
    fn check_and_add<L: Legality>(&mut self, b_move: BitMove) {
        if !L::gen_legal() || self.board.legal_move(b_move) {
            unsafe {
                let b_ptr = mem::transmute::<*mut MP::Output, *mut BitMove>(self.ptr);
                ptr::write(b_ptr, b_move);
                self.ptr = self.ptr.add(1);
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Legal, MoveGen};
    use board::fen::ALL_FENS;
    use board::Board;
    use core::mono_traits::AllGenType;

    #[test]
    fn movegen_legal_pseudo() {
        let boards = Board::random().pseudo_random(2627288300002).many(10);

        boards.iter().for_each(|b| {
            let b_legal = b.generate_moves();
            let b_plegal = b.generate_pseudolegal_moves();
            assert!(b_legal.len() <= b_plegal.len());
            for mov in b_legal {
                assert!(b_plegal.contains(&mov));
            }
        });
    }

    #[test]
    fn movelist_basic() {
        let b = Board::start_pos();
        let mut m = b.generate_moves();
        let mut i = 0;
        for _d in m.iter() {
            i += 1;
        }
        {
            let borrow = &m;
            assert_eq!(i, borrow.len());
        }
        {
            let borrow_mut = &mut m;
            assert_eq!(i, borrow_mut.len());
        }

        let m2 = m.to_vec();
        assert_eq!(m2.len(), m.len());
    }

    #[test]
    fn movegen_list_sim_basic() {
        let b = Board::start_pos();
        let mb = b.generate_moves();
        let ms = MoveGen::generate_scoring::<Legal, AllGenType>(&b);
        assert_eq!(mb.len(), ms.len());
        let iter = mb.iter().zip(ms.iter());
        iter.for_each(|(mb, ms)| assert_eq!(*mb, ms.bit_move));
    }

    #[test]
    fn movegen_list_sim_all() {
        let boards: Vec<Board> = ALL_FENS
            .iter()
            .map(|f| Board::from_fen(f).unwrap())
            .collect();

        boards.iter().for_each(|b| {
            let mb = b.generate_moves();
            let ms = MoveGen::generate_scoring::<Legal, AllGenType>(b);
            assert_eq!(mb.len(), ms.len());
            let iter = mb.iter().zip(ms.iter());
            iter.for_each(|(mb, ms)| assert_eq!(*mb, ms.bit_move));
        });
    }
}

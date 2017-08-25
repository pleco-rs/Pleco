use templates::*;
use board::*;
use piece_move::{MoveFlag, BitMove, PreMoveInfo};
use bit_twiddles::*;
use magic_helper::MagicHelper;

// MoveGen Classifications:
// Evasions, Captures, Quiets, Quiet_checks, Evasions, Non Evasions, Legal
//
// Private generation type to match requested type
// Legal -> All Moves
// Captures -> Captures only, even if in check
// Quiets -> Non captures, even if in check
// Evasions -> In check, any move that will get out of check
// Non Evasions -> Not in Check, generate all moves
// Quiet Checks -> Non captures that give check

#[derive(Copy, Clone, Debug, PartialEq)]
enum PriGenType {
    Legal,
    Captures,
    Quiets,
    Evasions,
    NonEvasions,
    QuietChecks,
}


// Public GenTypes are:
//     All        --> All moves
//     Captures   --> Captures only
//     Quiets     --> Non captures
//     Checks     --> Moves potentially giving check (Note, board cannot be in check)


// Pieces to generate moves with inter changably
const STANDARD_PIECES: [Piece; 4] = [Piece::B, Piece::N, Piece::R, Piece::Q];

// Struct to house some basic information about the board, quick access to magic helper,
// current turn, etc.
pub struct MoveGen<'a> {
    movelist: Vec<BitMove>,
    board: &'a Board,
    magic: &'static MagicHelper<'static, 'static>,
    turn: Player,
    them: Player,
    occ: BitBoard, // Squares occupied by all
    us_occ: BitBoard, // squares occupied by player to move
    them_occ: BitBoard, // Squares occupied by the opposing player
}

impl<'a> MoveGen<'a> {
    // Returns vector of all moves for a given board & GenType
    pub fn generate(chessboard: &Board, gen_type: GenTypes) -> Vec<BitMove> {
        let mut movegen = MoveGen::get_self(&chessboard);

        // target = Bitboard of squares the generator should aim for
        let target: BitBoard = match gen_type {
            GenTypes::All => u64::max_value(),
            GenTypes::Captures => movegen.them_occ,
            GenTypes::Quiets => !movegen.them_occ,
            GenTypes::QuietChecks => u64::max_value(),
        };

        if chessboard.in_check() {
            // We are in check, so we should not be looking for checks at this time
            assert_ne!(gen_type, GenTypes::QuietChecks);
            // Generate evasions
            movegen.generate_evasions(target);

        } else if gen_type == GenTypes::QuietChecks {
            unimplemented!();
        } else {

            // Capture moves cannot include castling
            if gen_type != GenTypes::Captures {
                movegen.generate_castling();
            }

            movegen.gen_non_pawn_king(target);
            movegen.generate_pawn_moves(
                target,
                match gen_type {
                    GenTypes::All => PriGenType::Legal,
                    GenTypes::Captures => PriGenType::Captures,
                    GenTypes::Quiets => PriGenType::Quiets,
                    GenTypes::QuietChecks => unreachable!(),
                },
            );
            movegen.generate_king_moves(target);
        }
        movegen.movelist
    }


    // Helper function to setup the MoveGen structure
    fn get_self(chessboard: &'a Board) -> Self {
        MoveGen {
            movelist: Vec::with_capacity(25),
            board: &chessboard,
            magic: chessboard.magic_helper,
            turn: chessboard.turn(),
            them: other_player(chessboard.turn()),
            occ: chessboard.get_occupied(),
            us_occ: chessboard.get_occupied_player(chessboard.turn()),
            them_occ: chessboard.get_occupied_player(other_player(chessboard.turn())),
        }
    }

    // Helper function to generate evasions
    fn generate_evasions(&mut self, target: BitBoard) {
        let ksq: SQ = self.board.king_sq(self.turn);

        let mut slider_attacks: BitBoard = 0;

        // Pieces that could possibly attack the king with sliding attacks
        let mut sliders = self.board.checkers() &
            (self.board.piece_bb(self.them, Piece::Q) | self.board.piece_bb(self.them, Piece::R) |
                 self.board.piece_bb(self.them, Piece::B));

        // This is getting all the squares that are attacked by sliders
        while sliders != 0 {
            let check_sq: SQ = bit_scan_forward(sliders);
            slider_attacks |= self.magic.line_bb(check_sq, ksq) ^ sq_to_bb(check_sq);
            sliders &= !(sq_to_bb(check_sq));
        }

        // Possible king moves, Where the king cannot move into a slider / own pieces
        let k_moves: BitBoard = self.magic.king_moves(ksq) & !slider_attacks & !self.us_occ;

        // Seperate captures and non captures
        let mut captures_bb: BitBoard = k_moves & self.them_occ & target;
        let mut non_captures_bb: BitBoard = k_moves & !self.them_occ & target;
        self.move_append_from_bb(
            &mut captures_bb,
            ksq,
            MoveFlag::Capture { ep_capture: false },
        );
        self.move_append_from_bb(&mut non_captures_bb, ksq, MoveFlag::QuietMove);

        // If there is only one checking square, we can block or capture the piece
        if !more_than_one(self.board.checkers()) {
            let checking_sq: SQ = bit_scan_forward(self.board.checkers());

            // Squares that allow a block or capture of the sliding piece
            let new_target: BitBoard =
                (self.magic.between_bb(checking_sq, ksq) | sq_to_bb(checking_sq)) & target;

            self.generate_pawn_moves(new_target, PriGenType::Evasions);
            self.gen_non_pawn_king(new_target);
        }
    }

    // Generate king moves with a given target
    fn generate_king_moves(&mut self, target: BitBoard) {
        self.moves_per_piece(Piece::K, target);
    }

    // Generates castling for both sides
    fn generate_castling(&mut self) {
        self.castling_side(CastleType::QueenSide);
        self.castling_side(CastleType::KingSide);
    }

    // Generates castling for a single side
    fn castling_side(&mut self, side: CastleType) {
        // Make sure we can castle AND the space between the king / rook is clear AND the piece at castling_side is a Rook
        if !self.board.castle_impeded(side) && self.board.can_castle(self.turn, side) &&
            self.board
                .piece_at_sq(self.board.castling_rook_square(side)) == Some(Piece::R)
        {

            let king_side: bool = { side == CastleType::KingSide };

            let ksq: SQ = self.board.king_sq(self.turn);
            let r_from: SQ = self.board.castling_rook_square(side);
            let k_to = relative_square(
                self.turn,
                if king_side {
                    Square::G1 as SQ
                } else {
                    Square::C1 as SQ
                },
            );

            let enemies: BitBoard = self.them_occ;
            let direction: fn(SQ) -> SQ = if king_side {
                |x: SQ| x.wrapping_sub(1)
            } else {
                |x: SQ| x.wrapping_add(1)
            };

            let mut s: SQ = k_to;
            let mut can_castle: bool = true;

            // Loop through all the squares the king goes through
            // If any enemies attack that square, cannot castle
            'outer: while s != ksq {
                let attackers = self.board.attackers_to(s, self.occ) & enemies;
                if attackers != 0 {
                    can_castle = false;
                    break 'outer;
                }
                s = direction(s);
            }
            if can_castle {
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: ksq,
                    dst: r_from,
                    flags: MoveFlag::Castle { king_side: king_side },
                }));
            }

        }
    }

    // Generate non-pawn and non-king moves for a target
    fn gen_non_pawn_king(&mut self, target: BitBoard) {
        for piece in STANDARD_PIECES.into_iter() {
            self.moves_per_piece(piece.clone(), target);
        }
    }

    // Get the captures and non-captures for a piece
    fn moves_per_piece(&mut self, piece: Piece, target: BitBoard) {
        let mut piece_bb: BitBoard = self.board.piece_bb(self.turn, piece);
        while piece_bb != 0 {
            let b: BitBoard = lsb(piece_bb);
            let src: SQ = bb_to_sq(b);
            let moves_bb: BitBoard = self.moves_bb(piece, src) & !self.us_occ & target;
            let mut captures_bb: BitBoard = moves_bb & self.them_occ;
            let mut non_captures_bb: BitBoard = moves_bb & !self.them_occ;
            self.move_append_from_bb(
                &mut captures_bb,
                src,
                MoveFlag::Capture { ep_capture: false },
            );
            self.move_append_from_bb(&mut non_captures_bb, src, MoveFlag::QuietMove);
            piece_bb &= !b;
        }
    }

    // Generate pawn moves
    fn generate_pawn_moves(&mut self, target: BitBoard, gen_type: PriGenType) {
        let rank_8: BitBoard = if self.turn == Player::White {
            RANK_8
        } else {
            RANK_1
        };
        let rank_7: BitBoard = if self.turn == Player::White {
            RANK_7
        } else {
            RANK_2
        };
        let rank_3: BitBoard = if self.turn == Player::White {
            RANK_3
        } else {
            RANK_6
        };

        let (down, up, left_down, right_down, shift_up, shift_left_up, shift_right_up): (
            fn(SQ) -> SQ,
            fn(SQ) -> SQ,
            fn(SQ) -> SQ,
            fn(SQ) -> SQ,
            fn(u64) -> u64,
            fn(u64) -> u64,
            fn(u64) -> u64,
        ) = if self.turn == Player::White {
            (
                |x: SQ| x.wrapping_sub(8), // Down
                |x: SQ| x.wrapping_add(8), // Up
                |x: SQ| x.wrapping_sub(9), // left_down
                |x: SQ| x.wrapping_sub(7), // right_down
                |x: u64| x.wrapping_shl(8), // Shift_up
                |x: u64| (x & !FILE_A).wrapping_shl(7), // Shift_left_up
                |x: u64| (x & !FILE_H).wrapping_shl(9),
            ) // Shift_Right_up
        } else {
            (
                |x: SQ| x.wrapping_add(8),
                |x: SQ| x.wrapping_sub(8),
                |x: SQ| x.wrapping_add(9),
                |x: SQ| x.wrapping_add(7),
                |x: u64| x.wrapping_shr(8),
                |x: u64| (x & !FILE_H).wrapping_shr(7),
                |x: u64| (x & !FILE_A).wrapping_shr(9),
            )
        };

        let all_pawns: BitBoard = self.board.piece_bb(self.turn, Piece::P);
        let pawns_rank_7: BitBoard = all_pawns & rank_7;
        let pawns_not_rank_7: BitBoard = all_pawns & !rank_7;

        let mut empty_squares: BitBoard = 0;

        let enemies: BitBoard = if gen_type == PriGenType::Evasions {
            self.them_occ & target
        } else if gen_type == PriGenType::Captures {
            target
        } else {
            self.them_occ
        };

        // Single and Double Pawn Pushes
        if gen_type != PriGenType::Captures {
            empty_squares =
                if gen_type == PriGenType::Quiets || gen_type == PriGenType::QuietChecks {
                    target
                } else {
                    !self.occ
                };

            let mut push_one: BitBoard = empty_squares & shift_up(pawns_not_rank_7);
            let mut push_two: BitBoard = shift_up(push_one & rank_3) & empty_squares;

            if gen_type == PriGenType::Evasions {
                push_one &= target;
                push_two &= target;
            }

            if gen_type == PriGenType::QuietChecks {
                let ksq: SQ = self.board.king_sq(self.them);
                push_one &= self.magic.pawn_attacks_from(ksq, self.them);
                push_two &= self.magic.pawn_attacks_from(ksq, self.them);

                let dc_candidates: BitBoard = self.board.discovered_check_candidates();
                if pawns_not_rank_7 & dc_candidates != 0 {
                    let dc1: BitBoard = shift_up(pawns_not_rank_7 & dc_candidates) &
                        empty_squares & !file_bb(ksq);
                    let dc2: BitBoard = shift_up(rank_3 & dc_candidates) & empty_squares;

                    push_one |= dc1;
                    push_two |= dc2;
                }
            }

            while push_one != 0 {
                let bit: BitBoard = lsb(push_one);
                let dst: SQ = bb_to_sq(bit);
                let src: SQ = down(dst);
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::QuietMove,
                }));
                push_one &= !bit;
            }

            while push_two != 0 {
                let bit: BitBoard = lsb(push_two);
                let dst: SQ = bb_to_sq(bit);
                let src: SQ = down(down(dst));
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::DoublePawnPush,
                }));
                push_two &= !bit;
            }
        }

        // Promotions
        if pawns_rank_7 != 0 && (gen_type != PriGenType::Evasions || (target & rank_8) != 0) {
            if gen_type == PriGenType::Captures {
                empty_squares = !self.occ;
            } else if gen_type == PriGenType::Evasions {
                empty_squares &= target;
            }

            let mut no_promo: BitBoard = shift_up(pawns_rank_7) & empty_squares;
            let mut left_cap_promo: BitBoard = shift_left_up(pawns_rank_7) & enemies;
            let mut right_cap_promo: BitBoard = shift_right_up(pawns_rank_7) & enemies;

            while no_promo != 0 {
                let bit = lsb(no_promo);
                let dst: SQ = bb_to_sq(bit);
                self.create_promotions(dst, down(dst), false);
                no_promo &= !bit;
            }

            while left_cap_promo != 0 {
                let bit = lsb(left_cap_promo);
                let dst: SQ = bb_to_sq(bit);
                self.create_promotions(dst, right_down(dst), true);
                left_cap_promo &= !bit;
            }

            while right_cap_promo != 0 {
                let bit = lsb(right_cap_promo);
                let dst: SQ = bb_to_sq(bit);
                self.create_promotions(dst, left_down(dst), true);
                right_cap_promo &= !bit;
            }
        }

        // Captures
        if gen_type == PriGenType::Captures || gen_type == PriGenType::Evasions ||
            gen_type == PriGenType::NonEvasions || gen_type == PriGenType::Legal
        {

            let mut left_cap: BitBoard = shift_left_up(pawns_not_rank_7) & enemies;
            let mut right_cap: BitBoard = shift_right_up(pawns_not_rank_7) & enemies;

            while left_cap != 0 {
                let bit = lsb(left_cap);
                let dst: SQ = bb_to_sq(bit);
                let src: SQ = right_down(dst);
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Capture { ep_capture: false },
                }));
                left_cap &= !bit;
            }

            while right_cap != 0 {
                let bit = lsb(right_cap);
                let dst: SQ = bb_to_sq(bit);
                let src: SQ = left_down(dst);
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Capture { ep_capture: false },
                }));
                right_cap &= !bit;
            }

            if self.board.ep_square() != NO_SQ {
                let ep_sq: SQ = self.board.ep_square();
                assert_eq!(rank_of_sq(ep_sq), relative_rank(self.turn, Rank::R6));
                if gen_type != PriGenType::Evasions || target & sq_to_bb(down(ep_sq)) != 0 {
                    left_cap = pawns_not_rank_7 & self.magic.pawn_attacks_from(ep_sq, self.them);

                    while left_cap != 0 {
                        let bit = lsb(left_cap);
                        let src: SQ = bb_to_sq(bit);
                        self.check_and_add(BitMove::init(PreMoveInfo {
                            src: src,
                            dst: ep_sq,
                            flags: MoveFlag::Capture { ep_capture: true },
                        }));
                        left_cap &= !bit;
                    }
                }
            }
        }
    }

    // Helper function for creating promotions
    fn create_promotions(&mut self, dst: SQ, src: SQ, is_capture: bool) {
        let prom_pieces = [Piece::Q, Piece::N, Piece::R, Piece::B];
        for piece in prom_pieces.into_iter() {
            if is_capture {
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Promotion {
                        capture: true,
                        prom: piece.clone(),
                    },
                }));
            } else {
                self.check_and_add(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Promotion {
                        capture: false,
                        prom: piece.clone(),
                    },
                }));
            }
        }
    }

    // Return the moves Bitboard
    #[inline]
    fn moves_bb(&self, piece: Piece, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        assert_ne!(piece, Piece::P);
        match piece {
            Piece::P => panic!(),
            Piece::N => self.magic.knight_moves(square),
            Piece::B => self.magic.bishop_moves(self.occ, square),
            Piece::R => self.magic.rook_moves(self.occ, square),
            Piece::Q => self.magic.queen_moves(self.occ, square),
            Piece::K => self.magic.king_moves(square),
        }
    }

    #[inline]
    fn move_append_from_bb(&mut self, bits: &mut BitBoard, src: SQ, move_flag: MoveFlag) {
        while *bits != 0 {
            let bit: BitBoard = lsb(*bits);
            let b_move = BitMove::init(PreMoveInfo {
                src: src,
                dst: bb_to_sq(bit),
                flags: move_flag,
            });
            self.check_and_add(b_move);
            *bits &= !bit;
        }
    }

    fn check_and_add(&mut self, b_move: BitMove) {
        if self.board.legal_move(b_move) {
            self.movelist.push(b_move);
        }
    }
}

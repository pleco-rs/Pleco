use templates::*;
use board::*;
use piece_move::{MoveFlag, BitMove, PreMoveInfo};
use bit_twiddles::*;
use magic_helper::MagicHelper;

// TODO:
// MoveGen Classifications:
// Evasions, Captures, Quiets, Quiet_checks, Evasions, Non Evasions, Legal
//
// Evasions: Board is currently in check; Generate moves that move the king or Block the attack
// Captures: This Move captures something;
// Quiets: Moves that do not capture a piece
// Non-Evasions: Board is currently in check; Generate moves that do not move the king

// Pieces not needed special considerations when generating (basically everything but pawns)
const STANDARD_PIECES: [Piece; 5] = [Piece::B, Piece::N, Piece::R, Piece::Q, Piece::K];

pub struct MoveGen<'a> {
    movelist: Vec<BitMove>,
    board: &'a Board,
    magic: &'static MagicHelper<'static,'static>,
    turn: Player,
    them: Player,
    occ: BitBoard,
    us_occ: BitBoard,
    them_occ: BitBoard,
}

impl <'a> MoveGen<'a> {
    pub fn generate_all(chessboard: &Board) -> Vec<BitMove> {
        let mut movegen = MoveGen::get_self(&chessboard);
        movegen.gen_all();
        movegen.movelist
    }

    fn get_self(chessboard: &'a Board) -> Self {
        MoveGen {
            movelist: Vec::with_capacity(25),
            board: &chessboard,
            magic: chessboard.magic_helper,
            turn: chessboard.turn(),
            them: other_player(chessboard.turn()),
            occ: chessboard.get_occupied(),
            us_occ: chessboard.get_occupied_player(chessboard.turn()),
            them_occ: chessboard.get_occupied_player(other_player(chessboard.turn()))}
    }

    fn generate_castling(&mut self) {
        unimplemented!();
    }

    fn create_promotions(&mut self, dst: SQ, src: SQ, is_capture: bool) {
        let prom_pieces = [Piece::Q, Piece::N, Piece::R, Piece::B];
        for piece in prom_pieces.into_iter() {
            if is_capture {
                self.movelist.push(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Promotion {capture: true, prom: piece.clone()},
                }));
            } else {
                self.movelist.push(BitMove::init(PreMoveInfo {
                    src: src,
                    dst: dst,
                    flags: MoveFlag::Promotion {capture: false, prom: piece.clone()},
                }));
            }
        }
    }

    fn pawn_moves(&mut self) {
        unimplemented!();
    }

    // Return the moves Bitboard
    #[inline]
    fn moves_bb(&self, piece: Piece, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        assert_ne!(piece, Piece::P);
        match piece {
            Piece::P => panic!(),
            Piece::N => self.magic.knight_moves(square),
            Piece::B => self.magic.bishop_moves(self.occ,square),
            Piece::R => self.magic.rook_moves(self.occ,square),
            Piece::Q => self.magic.queen_moves(self.occ,square),
            Piece::K => self.magic.king_moves(square)
        }
    }

    fn gen_all(&mut self) {
        for piece in STANDARD_PIECES.into_iter() {
            self.gen_non_pawn_moves(piece.clone());
        }
        self.generate_castling();
        self.pawn_moves();
    }

    fn gen_non_pawn_moves(&mut self, piece: Piece) {
        let mut piece_bb: BitBoard = self.board.piece_bb(self.turn, piece);
        while piece_bb != 0 {
            let b: BitBoard = lsb(piece_bb);
            let src: SQ = bb_to_sq(b);
            let moves_bb: BitBoard = self.moves_bb(piece, src) & !self.us_occ;
            let mut captures_bb: BitBoard = moves_bb & self.them_occ;
            let mut non_captures_bb: BitBoard = moves_bb & !self.them_occ;
            self.move_append_from_bb(&mut captures_bb, src, MoveFlag::Capture {ep_capture:false});
            self.move_append_from_bb(&mut non_captures_bb, src, MoveFlag::QuietMove );
            piece_bb &= !b;
        }
    }

    #[inline]
    fn move_append_from_bb(&mut self, bits: &mut BitBoard, src: SQ, move_flag: MoveFlag) {
        while *bits != 0 {
            let bit: BitBoard = lsb(*bits);
            let b_move = BitMove::init(PreMoveInfo{
                src: src,
                dst: bb_to_sq(bit),
                flags: move_flag,
            });
            if self.board.legal_move(b_move) {
                self.movelist.push(b_move);
            }
            *bits &= !bit;
        }
    }
}

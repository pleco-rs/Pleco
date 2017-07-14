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

    fn create_promotions(&mut self, dst: SQ, src: SQ) {
        unimplemented!();
    }

    fn pawn_moves(&mut self) {
        unimplemented!();
    }

    // Return the moves Bitboard
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





//fn gen_queen_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
//    let mut p_bb: BitBoard = board.piece_bb(move_info.us, Piece::Q);
//    while p_bb != 0 {
//        let b: BitBoard = lsb(p_bb);
//        let src: SQ = bb_to_sq(b);
//        let moves_bb: BitBoard = board.magic_helper.queen_moves(move_info.occupied & !b, src) & !move_info.us_occupied;
//        let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
//        let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
//
//        pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
//        pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
//
//        p_bb &= !b;
//    }
//    list
//}








#[inline]
fn pre_move_info_from_bb(pre_move_list: &mut Vec<PreMoveInfo>, source_sq: SQ, mut move_bb: BitBoard, flag: MoveFlag) {
    while move_bb != 0 {
        let bit: BitBoard = lsb(move_bb);
        pre_move_list.push(PreMoveInfo {src: source_sq, dst: bb_to_sq(bit), flags: flag});
        move_bb ^= bit;
    }
}


pub fn bit_scan_forward_list(input_bits: u64, list: &mut Vec<u8>) {
    let mut bits = input_bits;
    while bits != 0 {
        let pos = bit_scan_forward(bits);
        list.push(pos);
        let pos = (1u64).checked_shl(pos as u32).unwrap();
        bits &= !(pos) as u64;
    }
}

use templates::*;
use board::*;
use piece_move::{MoveFlag, BitMove, PreMoveInfo};
use bit_twiddles::*;


// Struct to store repeatedly used information
pub struct MoveInfos {
    occupied: BitBoard,
    us_occupied: BitBoard,
    them_occupied: BitBoard,
    us: Player,
    them: Player,
}



impl MoveInfos {
    pub fn new(board: &Board) -> MoveInfos {
        let us_p: Player = board.turn;
        let them_p: Player = other_player(us_p);
        let us_occ = board.get_occupied_player(us_p);
        let them_occ = board.get_occupied_player(them_p);
        MoveInfos {
            occupied: us_occ | them_occ,
            us_occupied: us_occ,
            them_occupied: them_occ,
            us: us_p,
            them: them_p
        }
    }
}

// TODO:
// MoveGen Classifications:
// Evasions, Captures, Quiets, Quiet_checks, Evasions, Non Evasions, Legal
//
// Evasions: Board is currently in check; Generate moves that block the check or move away
// Captures:


pub fn get_moves(board: &Board) -> Vec<BitMove> {
    let move_info = MoveInfos::new(&board);
//    let pseduo_moves = get_pseudo_moves(&board, &move_info);

    unimplemented!();
}

//fn get_pseudo_moves(board: &Board, move_info: &MoveInfos) -> Vec<PreMoveInfo> {
//    let mut vec = Vec::with_capacity(256);
//    get_pawn_moves(&board, move_info.us, &mut vec);
//    vec
//}

fn gen_queen_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
    let mut p_bb: BitBoard = board.piece_bb(move_info.us, Piece::Q);
    while p_bb != 0 {
        let b: BitBoard = lsb(p_bb);
        let src: SQ = bb_to_sq(b);
        let moves_bb: BitBoard = board.magic_helper.queen_moves(move_info.occupied & !b, src) & !move_info.us_occupied;
        let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
        let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
        pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
        pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
        p_bb &= !b;
    }
    list
}


fn gen_rook_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
    let mut p_bb: BitBoard = board.piece_bb(move_info.us, Piece::R);
    while p_bb != 0 {
        let b: BitBoard = lsb(p_bb);
        let src: SQ = bb_to_sq(b);
        let moves_bb: BitBoard = board.magic_helper.rook_moves(move_info.occupied & !b, src) & !move_info.us_occupied;
        let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
        let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
        pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
        pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
        p_bb &= !b;
    }
    list
}

fn gen_bishop_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
    let mut p_bb: BitBoard = board.piece_bb(move_info.us, Piece::B);
    while p_bb != 0 {
        let b: BitBoard = lsb(p_bb);
        let src: SQ = bb_to_sq(b);
        let moves_bb: BitBoard = board.magic_helper.bishop_moves(move_info.occupied & !b, src) & !move_info.us_occupied;
        let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
        let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
        pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
        pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
        p_bb &= !b;
    }
    list
}

fn gen_knight_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
    let mut p_bb: BitBoard = board.piece_bb(move_info.us, Piece::N);
    while p_bb != 0 {
        let b: BitBoard = lsb(p_bb);
        let src: SQ = bb_to_sq(b);
        let moves_bb: BitBoard = board.magic_helper.knight_moves(src) & !move_info.us_occupied;
        let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
        let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
        pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
        pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
        p_bb &= !b;
    }
    list
}

fn gen_king_moves(board: &Board, move_info: &MoveInfos, mut list: Vec<PreMoveInfo>) -> Vec<PreMoveInfo> {
    let p_bb: BitBoard = board.piece_bb(move_info.us, Piece::K);
    let b: BitBoard = lsb(p_bb);
    let src: SQ = bb_to_sq(b);
    let moves_bb: BitBoard = board.magic_helper.knight_moves(src) & !move_info.us_occupied;
    let captures_bb: BitBoard = moves_bb & move_info.them_occupied;
    let non_captures_bb: BitBoard = moves_bb & !move_info.them_occupied;
    pre_move_info_from_bb(&mut list, src, captures_bb, MoveFlag::Capture {ep_capture: false});
    pre_move_info_from_bb(&mut list, src, non_captures_bb, MoveFlag::QuietMove);
    list
}




// Gets pawn attacks from a square
pub fn pawn_attacks_from(sq: SQ, player: Player) -> BitBoard {
    match player {
        Player::White => {
            let mut board: u64 = 0;
            if sq < 56 {
                let file = file_of_sq(sq);
                if file != 0 {
                    board |= (1 as u64).wrapping_shl((sq + 7) as u32);
                }
                if file != 7 {
                    board |= (1 as u64).wrapping_shl((sq + 9) as u32);
                }
            }
            board
        },
        Player::Black => {
            let mut board: u64 = 0;
            if sq > 7 {
                let file = file_of_sq(sq);
                if file != 0 {
                    board |= (1 as u64).wrapping_shl((sq - 9) as u32);
                }
                if file != 7 {
                    board |= (1 as u64).wrapping_shl((sq - 7) as u32);
                }
            }
            board
        }
    }
}




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

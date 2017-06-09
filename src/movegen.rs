use templates::*;
use board::*;
use piece_move::{MoveFlag, BitMove, PreMoveInfo};
use bit_twiddles::{bit_scan_forward};


// Struct to store repeatedly used information
pub struct MoveInfos {
    board: &Board,
    occupied: BitBoard,
    us_occupied: BitBoard,
    them_occupied: BitBoard,
    us: Player,
    them: Player,
}

impl MoveInfos {
    pub fn new(board: &Board) -> MoveInfos {
        let us_p: Player = board.turn;
        let them_p: other_player(us_p);
        let us_occ = board.get_occupied_player(us_p);
        let them_occ = board.get_occupied_player(them_p);
        MoveInfos {
            board: board,
            occupied: us_occ | them_occ,
            us_occupied: us_occ,
            them_occupied: them_occ,
            us: us_p,
            them: them_p
        }
    }

    pub fn gen_moves(&self) -> Vec<BitMove> {

    }

    fn pseudo_moves(&self) -> Vec<PreMoveInfo> {

    }



}

pub fn get_moves(board: &Board) -> Vec<BitMove> {
    MoveInfos::new(board).gen_moves()
}

fn get_pseudo_moves(move_info: &MoveInfos, board: &Board, player: Player) -> Vec<PreMoveInfo> {
    let mut vec = Vec::with_capacity(40);
    get_pawn_moves(&board, player, &mut vec);
    vec
}

fn in_check(board: &Board, bit_move: BitMove) -> bool {
    unimplemented!();
}



fn get_pawn_moves(board: &Board, player: Player, list: &mut Vec<PreMoveInfo>) {
    #[allow(unused)]
    let THEM: Player = match player {
        Player::White => Player::Black,
        Player::Black => Player::White
    };
    #[allow(unused)]
    let TRANK8BB: u64 = match player {
        Player::White => RANK_8,
        Player::Black => RANK_1
    };
    #[allow(unused)]
    let TRANK7BB: u64 = match player {
        Player::White => RANK_7,
        Player::Black => RANK_2
    };
    #[allow(unused)]
    let TRANK5BB: u64 = match player {
        Player::White => RANK_5,
        Player::Black => RANK_4
    };
    let TRANK3BB: u64 = match player {
        Player::White => RANK_3,
        Player::Black => RANK_6
    };

    let pawn_bits = board.get_bitboard(player, Piece::P);
    let occupied = board.get_occupied();

    // get single and double pushes
    let single_push: BitBoard = safe_u_shift(pawn_bits, player) & !occupied;
    let double_push: BitBoard = safe_u_shift(single_push & TRANK3BB, player) & !occupied;

    // Single Moves
    let mut single_push_list = Vec::new();
    bit_scan_forward_list(single_push, &mut single_push_list);
    while !single_push_list.is_empty() {
        let dest = single_push_list.pop().unwrap();
        let sorc = match player {
            Player::White => dest - 8,
            Player::Black => dest + 8,
        };
        if 1<< dest & TRANK8BB != 0 {
            list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::Promotion {capture: false, prom:Piece::B} });
            list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::Promotion {capture: false, prom:Piece::R} });
            list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::Promotion {capture: false, prom:Piece::N} });
            list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::Promotion {capture: false, prom:Piece::Q} });
        } else {
            list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::QuietMove });
        }
    }

    let mut double_push_list = Vec::new();
    bit_scan_forward_list(double_push, &mut double_push_list);

    // Double Moves
    while !double_push_list.is_empty() {
        let dest = double_push_list.pop().unwrap();
        let sorc = match player {
            Player::White => dest - 8,
            Player::Black => dest + 8,
        };
        list.push(PreMoveInfo { src: sorc, dst: dest, flags: MoveFlag::DoublePawnPush });
    }

    let ep_square = board.en_passant;
    if ep_square != 64 {
        let ep_bit: BitBoard = 1<<ep_square;
        let ep_mask: BitBoard = ep_bit >> 1 | ep_bit << 1;
        let pawns_possible_to_ep = ep_mask & pawn_bits & TRANK5BB;
        if pawns_possible_to_ep != 0 {
            let dest = bit_scan_forward(safe_u_shift(ep_bit, player));
            if safe_l_shift(ep_bit, player) & pawns_possible_to_ep != 0 {
                if safe_l_shift(ep_bit, player) & TRANK7BB != 0 {
                    list.push(PreMoveInfo { src: bit_scan_forward(safe_l_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Promotion {capture: true, prom:Piece::B} });
                    list.push(PreMoveInfo { src: bit_scan_forward(safe_l_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Promotion {capture: true, prom:Piece::R} });
                    list.push(PreMoveInfo { src: bit_scan_forward(safe_l_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Promotion {capture: true, prom:Piece::N} });
                    list.push(PreMoveInfo { src: bit_scan_forward(safe_l_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Promotion {capture: true, prom:Piece::Q} });
                }
                list.push(PreMoveInfo { src: bit_scan_forward(safe_l_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Capture{ep_capture: true} });
            }
            if (safe_r_shift(ep_bit, player)) & pawns_possible_to_ep != 0 {
                list.push(PreMoveInfo { src: bit_scan_forward(safe_r_shift(ep_bit, player)), dst: dest, flags: MoveFlag::Capture{ep_capture: true} });
            }
        };
    }

//    let left_file: u64 = match player {
//        Player::White => FILE_A,
//        Player::White => FILE_H,
//    };
//
//    let right_file: u64 = match player {
//        Player::White => FILE_H,
//        Player::White => FILE_A,
//    };
//
//    let opp_pieces: u64 = board.get_occupied_player(them);
//    let mut left_attacks: u64 = ((pawn_bits & !left_file) << (LEFT + UP)) & opp_pieces;
//    let mut right_attacks: u64 = ((pawn_bits & !right_file) << (RIGHT + UP)) & opp_pieces;
//    while left_attacks != 0 {
//        let attacked_sq = bit_scan_forward(bits);
//        let dst_bits: u64 = (1u64).checked_shl(attacked_sq as u32).unwrap();
//        let srq_sq = bit_scan_forward(dst_bits + RIGHT + DOWN);
//        left_attacks &= !(dst) as u64;
//        list.push(PreMoveInfo { src: srq_sq), dst: attacked_sq), flags: MoveFlag::Capture{ep_capture: true} });
//    }
//    while right_attacks != 0 {
//        let attacked_sq = bit_scan_forward(bits);
//        let dst_bits: u64 = (1u64).checked_shl(attacked_sq as u32).unwrap();
//        let srq_sq = bit_scan_forward(dst_bits + LEFT + DOWN);
//        left_attacks &= !(dst) as u64;
//        list.push(PreMoveInfo { src: srq_sq), dst: attacked_sq), flags: MoveFlag::Capture{ep_capture: true} });
//    }
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

// TODO: Implement Knight Attacks
// TODO: Implement King Attacks
// TODO: Implement Diagonal Attacks
// TODO: Implement Sliding Attacks
// TODO: Implement Knight Attacks

// TODO: Implement Move Checker Attacks



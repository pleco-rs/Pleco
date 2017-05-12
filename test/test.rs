

use board::{Board,AllBitBoards};
use templates::{Piece,Player};
use movegen::*;
use piece_move::*;
use std::*;






#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counts() {
        let board = Board::new();

        let count_w_p = board.count_piece(Player::White, Piece::P);
        assert_eq!(count_w_p,8);

        let count_w_n = board.count_piece(Player::White, Piece::N);
        assert_eq!(count_w_n,2);

        let count_w_b = board.count_piece(Player::White, Piece::B);
        assert_eq!(count_w_b,2);

        let count_w_r = board.count_piece(Player::White, Piece::R);
        assert_eq!(count_w_r,2);

        let count_w_k = board.count_piece(Player::White, Piece::K);
        assert_eq!(count_w_k,1);

        let count_w_q = board.count_piece(Player::White, Piece::Q);
        assert_eq!(count_w_q,1);

        let count_b_p = board.count_piece(Player::Black, Piece::P);
        assert_eq!(count_b_p,8);

        let count_b_n = board.count_piece(Player::Black, Piece::N);
        assert_eq!(count_b_n,2);

        let count_b_b = board.count_piece(Player::Black, Piece::B);
        assert_eq!(count_b_b,2);

        let count_b_r = board.count_piece(Player::Black, Piece::R);
        assert_eq!(count_b_r,2);

        let count_b_k = board.count_piece(Player::Black, Piece::K);
        assert_eq!(count_b_k,1);

        let count_b_q = board.count_piece(Player::Black, Piece::Q);
        assert_eq!(count_b_q,1);
    }


    #[test]
    fn check_two_piece_one_square() {
        let board = Board::new();
        let xor = board.bit_boards.into_iter().fold(0, |sum, x| sum ^ x);
        let or = board.bit_boards.into_iter().fold(0, |sum, x| sum | x);
        assert_eq!(or,xor);

    }

    #[test]
    fn test_bit_scan() {
        assert_eq!(movegen::bit_scan_forward(2),1);
        assert_eq!(movegen::bit_scan_forward(4),2);
        assert_eq!(movegen::bit_scan_forward(8),3);
        assert_eq!(movegen::bit_scan_forward(16),4);
        assert_eq!(movegen::bit_scan_forward(32),5);
        assert_eq!(movegen::bit_scan_forward(31),0);
    }

    #[test]
    fn test_pawn_gen() {
        let board = Board::new();
        let vector = movegen::get_pseudo_moves(&board, Player::White);
        assert_eq!(vector.len(),16);
    }

}
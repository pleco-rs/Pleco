extern crate pleco;

use pleco::board::Board;
use pleco::piece_move::BitMove;


fn main() {
    let mut board = Board::default(); // create a board of the starting position
    let moves: Vec<BitMove> = board.generate_moves(); // generate all possible legal moves
    board.apply_move(moves[0]);
    assert_eq!(board.moves_played(), 1);
    board.pretty_print();
}
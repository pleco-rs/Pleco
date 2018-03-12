extern crate pleco;
extern crate rand;

use pleco::{Board,BitMove};
use pleco::board::{RandBoard};


// Testing that applying / undoing a move leads to the same zobriust hash
#[test]
fn zobrist_correctness() {
    for _x in 0..15 {
        let mut board = RandBoard::default().one().shallow_clone();
        randomize(&mut board);
    }
}

fn randomize(board: &mut Board) {
    let list = board.generate_moves();
    let num_iterations = ((rand::random::<usize>() % 6) + 3)
        .min(list.len());

    let mut moves = Vec::with_capacity(num_iterations);
    for _x in 0..num_iterations {
        moves.push(list[rand::random::<usize>() % list.len()]);
    }

    while let Some(mov) = moves.pop() {
        let depth: usize = (rand::random::<usize>() % 9) + 6;
        board.apply_move(mov);
        randomize_inner(board, depth);
        board.undo_move();
    }
}


fn randomize_inner(board: &mut Board, depth: usize) {
    check_zob(&board);
    if depth != 0 {
        let moves = board.generate_moves();
        if moves.len() == 0 {
            return;
        }

        let rn = rand::random::<usize>() % moves.len();
        board.apply_move( moves[rn % moves.len()]);
        randomize_inner(board, depth - 1);
        board.undo_move();

        if rn > 3 && rn % 4 == 0 && depth > 4 {
            board.apply_move( moves[rn - 1]);
            randomize_inner(board, depth - 2);
            board.undo_move();
        }
    }
}


fn check_zob(board: &Board) {
    let zobrist = board.zobrist();
    let fen = board.fen();
    let fen_board = Board::from_fen(&fen).unwrap();
    let post_zob = fen_board.zobrist();

    if board.depth() > 0 && zobrist != post_zob {
        let last_move_played = board.last_move().unwrap_or(BitMove::null());
        let mut prev_board: Board = board.parallel_clone();
        prev_board.undo_move();
        let prev_fen = prev_board.fen();
        panic!("\nBoard did not have correct zobrist before and after! ply: {} \n\
                current fen: {}\n\
                last move played: {}, flags: {:b} \n\
                previous fen: {}\n\
                pretty: \n\
                {} \n
                previous pretty: \n\
                {} \n",
               board.depth(), fen, last_move_played, last_move_played.get_raw() >> 12,
               prev_fen, board.pretty_string(), prev_board.pretty_string());
    }

}

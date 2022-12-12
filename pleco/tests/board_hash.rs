extern crate pleco;
extern crate rand;

use pleco::board::RandBoard;
use pleco::{BitMove, Board};

trait HashCorrect {
    fn get_hash(board: &Board) -> u64;

    fn check_hash(board: &Board) {
        if board.depth() > 0 {
            let pre_hash: u64 = Self::get_hash(board);
            let fen = board.fen();
            let fen_board = Board::from_fen(&fen).unwrap();
            let post_hash: u64 = Self::get_hash(&fen_board);

            if pre_hash != post_hash {
                Self::print_hash(board, &fen);
            }
        }
    }

    fn print_hash(board: &Board, fen: &str) {
        let last_move_played = board.last_move().unwrap_or(BitMove::null());
        let mut prev_board: Board = board.parallel_clone();
        prev_board.undo_move();
        let prev_fen = prev_board.fen();
        panic!(
            "\nBoard did not have correct zobrist before and after! ply: {} \n\
                current fen: {}\n\
                last move played: {}, flags: {:b} \n\
                previous fen: {}\n\
                pretty: \n\
                {} \n
                previous pretty: \n\
                {} \n",
            board.depth(),
            fen,
            last_move_played,
            last_move_played.get_raw() >> 12,
            prev_fen,
            board.pretty_string(),
            prev_board.pretty_string()
        );
    }
}

struct ZobHashCorrect {}
struct MaterialHashCorrect {}
struct PawnHashCorrect {}

impl HashCorrect for ZobHashCorrect {
    fn get_hash(board: &Board) -> u64 {
        board.zobrist()
    }
}
impl HashCorrect for MaterialHashCorrect {
    fn get_hash(board: &Board) -> u64 {
        board.material_key()
    }
}

impl HashCorrect for PawnHashCorrect {
    fn get_hash(board: &Board) -> u64 {
        board.pawn_key()
    }
}

// Testing that applying / undoing a move leads to the same zobriust hash
#[test]
fn zobrist_correctness() {
    for _x in 0..15 {
        let mut board = RandBoard::default().one().shallow_clone();
        randomize::<ZobHashCorrect>(&mut board);
    }
}

#[test]
fn pawn_key_correctness() {
    for _x in 0..15 {
        let mut board = RandBoard::default().one().shallow_clone();
        randomize::<PawnHashCorrect>(&mut board);
    }
}

#[test]
fn material_key_correctness() {
    for _x in 0..15 {
        let mut board = RandBoard::default().one().shallow_clone();
        randomize::<MaterialHashCorrect>(&mut board);
    }
}

fn randomize<H: HashCorrect>(board: &mut Board) {
    let list = board.generate_moves();
    let num_iterations = ((rand::random::<usize>() % 6) + 3).min(list.len());

    let mut moves = Vec::with_capacity(num_iterations);
    for _x in 0..num_iterations {
        moves.push(list[rand::random::<usize>() % list.len()]);
    }

    while let Some(mov) = moves.pop() {
        let depth: usize = (rand::random::<usize>() % 9) + 6;
        board.apply_move(mov);
        randomize_inner::<H>(board, depth);
        board.undo_move();
    }
}

fn randomize_inner<H: HashCorrect>(board: &mut Board, depth: usize) {
    H::check_hash(board);
    if depth != 0 {
        let moves = board.generate_moves();
        if moves.is_empty() {
            return;
        }

        let rn = rand::random::<usize>() % moves.len();
        board.apply_move(moves[rn % moves.len()]);
        randomize_inner::<H>(board, depth - 1);
        board.undo_move();

        if rn > 3 && rn % 4 == 0 && depth > 4 {
            board.apply_move(moves[rn - 1]);
            randomize_inner::<H>(board, depth - 2);
            board.undo_move();
        }
    }
}

//
//fn check_zob(board: &Board) {
//    let zobrist = board.zobrist();
//    let fen = board.fen();
//    let fen_board = Board::from_fen(&fen).unwrap();
//    let post_zob = fen_board.zobrist();
//
//    if board.depth() > 0 && zobrist != post_zob {
//        let last_move_played = board.last_move().unwrap_or(BitMove::null());
//        let mut prev_board: Board = board.parallel_clone();
//        prev_board.undo_move();
//        let prev_fen = prev_board.fen();
//        panic!("\nBoard did not have correct zobrist before and after! ply: {} \n\
//                current fen: {}\n\
//                last move played: {}, flags: {:b} \n\
//                previous fen: {}\n\
//                pretty: \n\
//                {} \n
//                previous pretty: \n\
//                {} \n",
//               board.depth(), fen, last_move_played, last_move_played.get_raw() >> 12,
//               prev_fen, board.pretty_string(), prev_board.pretty_string());
//    }
//
//}

#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use pleco::{Player,Board,BitMove,MoveList};
use pleco::board::RandBoard;
use pleco::tools::prng::PRNG;

pub const SEED: u64 = 5363310003543;

use test::{black_box, Bencher};

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        RandBoard::default().min_moves(5).pseudo_random(SEED).many(100)
    };
}


#[bench]
fn bench_board_100_clone(b: &mut Bencher) {

    b.iter(|| {
        for board in RAND_BOARDS.iter() {
            black_box(board.shallow_clone());
        }
    })
}

#[bench]
fn bench_find(b: &mut Bencher) {
    b.iter(|| {
        black_box({
            for board in RAND_BOARDS.iter() {
                black_box( board.king_sq(Player::Black));
            }
        })
    })
}

#[bench]
fn bench_apply_100_move(b: &mut Bencher) {
    let mut prng = PRNG::init(SEED);
    let mut board_move: Vec<(Board, BitMove)> = Vec::with_capacity(100);

    for board in RAND_BOARDS.iter() {
        let moves: Vec<BitMove> = MoveList::into(board.generate_moves());
        let bit_move = *moves.get(prng.rand() as usize % moves.len()).unwrap();
        board_move.push((board.parallel_clone(),bit_move));
    }

    b.iter(|| {
        black_box({
            for t in board_move.iter_mut() {
                let b: &mut Board = &mut (t.0);
                black_box(black_box(b.parallel_clone()).apply_move(t.1));
            }
        })
    })
}

#[bench]
fn bench_undo_100_move(b: &mut Bencher) {
    let mut boards: Vec<Board> = Vec::with_capacity(100);
    for board in RAND_BOARDS.iter() {
        boards.push(board.parallel_clone());
    }

    b.iter(|| {
        black_box({
            for b in boards.iter_mut() {
                black_box(black_box(b.parallel_clone()).undo_move());
            }
        })
    })
}



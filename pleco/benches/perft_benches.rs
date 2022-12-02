use std::time::Duration;

use criterion::{black_box, Bencher, Criterion, Fun};

use pleco::board::perft::*;
use pleco::board::Board;

fn perft_3(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter(|| {
        for board in boards.iter() {
            black_box(perft(board, 3));
        }
    })
}

fn perft_4(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter(|| {
        for board in boards.iter() {
            black_box(perft(board, 4));
        }
    })
}

fn perft_all(c: &mut Criterion) {
    let rand_boards: Vec<Board> = RAND_BOARDS_ALL
        .iter()
        .map(|b| Board::from_fen(b).unwrap())
        .collect();

    let perft_3_f = Fun::new("Perft 3", perft_3);
    let perft_4_f = Fun::new("Perft 4", perft_4);

    let funs = vec![perft_3_f, perft_4_f];

    c.bench_functions("Perft All", funs, rand_boards);
}

criterion_group!(name = perft_benches;
     config = Criterion::default()
        .sample_size(12)
        .warm_up_time(Duration::from_millis(20));
    targets = perft_all
);

static RAND_BOARDS_ALL: [&str; 6] = [
    "rn2k3/pp1qPppr/5n2/1b2B3/8/4NP2/3NP1PP/R2K1B1R b q - 0 23",
    "r1bqkbnr/ppp2ppp/2np4/4p3/4PQ2/2NP4/PPP1NPPP/R1B1KB1R w KQkq e6 0 8",
    "r1bqkb1r/pp2pp2/2p2n2/6Q1/7p/2N4P/PP1B1PP1/R3KBNR w KQkq - 0 14",
    "3k4/6b1/1p5p/4p3/5rP1/6K1/8/ w - - 0 40",
    "1k6/1p1n4/p6p/4P3/2P5/1R6/5K1P/4R b - - 2 33",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
];

use std::time::Duration;

use criterion::{black_box, Bencher, Criterion, Fun};

use lazy_static;
use pleco::bot_prelude::*;
use pleco::tools::Searcher;
use pleco::Board;

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        let mut vec = Vec::new();
        vec.push(Board::start_pos());
        vec
    };
}

fn bench_searcher<S: Searcher>(b: &mut Bencher, data: &(&Vec<Board>, u16)) {
    b.iter(|| {
        for board in data.0.iter() {
            black_box(S::best_move(board.shallow_clone(), data.1));
        }
    })
}

fn bench_all_searchers_4_ply(c: &mut Criterion) {
    lazy_static::initialize(&RAND_BOARDS);
    let minimax = Fun::new("MiniMax", bench_searcher::<MiniMaxSearcher>);
    let parallel_minimax = Fun::new("ParallelMiniMax", bench_searcher::<ParallelMiniMaxSearcher>);
    let alpha_beta = Fun::new("AlphaBeta", bench_searcher::<AlphaBetaSearcher>);
    let jamboree = Fun::new("Jamboree", bench_searcher::<JamboreeSearcher>);

    let funs = vec![minimax, parallel_minimax, alpha_beta, jamboree];

    c.bench_functions("Searcher Benches 4 ply", funs, (&RAND_BOARDS, 4));
}

criterion_group!(name = bot_benches;
    config = Criterion::default().sample_size(11).warm_up_time(Duration::from_millis(100));
    targets = bench_all_searchers_4_ply
);

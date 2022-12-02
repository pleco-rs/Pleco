use criterion::{black_box, BatchSize, Bencher, Criterion};
use std::time::Duration;

use pleco::Board;

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::threadpool::*;
use pleco_engine::time::uci_timer::PreLimits;

use super::*;

const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

fn search_kiwipete_3moves_engine<D: DepthLimit>(b: &mut Bencher) {
    let mut pre_limit = PreLimits::blank();
    pre_limit.depth = Some(D::depth());
    let mut searcher = PlecoSearcher::init(false);
    let limit = pre_limit.create();
    let board_kwi: Board = Board::from_fen(KIWIPETE).unwrap();
    b.iter_batched(
        || {
            threadpool().clear_all();
            searcher.clear_tt();
            board_kwi.shallow_clone()
        },
        |mut board| {
            let mov = black_box(threadpool().search(&board, &limit));
            board.apply_move(mov);
            let mov = black_box(threadpool().search(&board, &limit));
            board.apply_move(mov);
            black_box(threadpool().search(&board, &limit));
        },
        BatchSize::PerIteration,
    )
}

fn search_startpos_3moves_engine<D: DepthLimit>(b: &mut Bencher) {
    let mut pre_limit = PreLimits::blank();
    pre_limit.depth = Some(D::depth());
    let mut searcher = PlecoSearcher::init(false);
    let limit = pre_limit.create();
    b.iter_batched(
        || {
            threadpool().clear_all();
            searcher.clear_tt();
            Board::start_pos()
        },
        |mut board| {
            let mov = black_box(threadpool().search(&board, &limit));
            board.apply_move(mov);
            let mov = black_box(threadpool().search(&board, &limit));
            board.apply_move(mov);
            black_box(threadpool().search(&board, &limit));
        },
        BatchSize::PerIteration,
    )
}

fn bench_engine_evaluations(c: &mut Criterion) {
    c.bench_function(
        "Search MultiMove Depth 5",
        search_startpos_3moves_engine::<Depth5>,
    );
    c.bench_function(
        "Search MultiMove Depth 6",
        search_startpos_3moves_engine::<Depth6>,
    );
    c.bench_function(
        "Search MultiMove Depth 7",
        search_startpos_3moves_engine::<Depth7>,
    );
    c.bench_function(
        "Search MultiMove Depth 8",
        search_startpos_3moves_engine::<Depth8>,
    );
    c.bench_function(
        "Search KiwiPete MultiMove Depth 5",
        search_kiwipete_3moves_engine::<Depth5>,
    );
    c.bench_function(
        "Search KiwiPete MultiMove Depth 6",
        search_kiwipete_3moves_engine::<Depth6>,
    );
    c.bench_function(
        "Search KiwiPete MultiMove Depth 7",
        search_kiwipete_3moves_engine::<Depth7>,
    );
    c.bench_function(
        "Search KiwiPete MultiMove Depth 8",
        search_kiwipete_3moves_engine::<Depth8>,
    );
}

criterion_group!(name = search_multimove;
     config = Criterion::default()
        .sample_size(35)
        .warm_up_time(Duration::from_millis(150));
    targets = bench_engine_evaluations
);

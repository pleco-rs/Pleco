#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use pleco::core::GenTypes;
use pleco::board::{RandBoard,Board};

use test::{black_box, Bencher};

lazy_static! {
    pub static ref RAND_BOARDS_NON_CHECKS: Vec<Board> = {
        RandBoard::default().no_check().pseudo_random(83436).many(30)
    };

    pub static ref RAND_BOARDS_CHECKS: Vec<Board> = {
        RandBoard::default().in_check().pseudo_random(446631).many(30)
    };

    pub static ref RAND_BOARDS_ANY: Vec<Board> = {
        RandBoard::default().pseudo_random(2310085).many(30)
    };
}


#[bench]
fn bench_movegen_any_legal(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_ANY.iter() {
            black_box(board.generate_moves());
        }
    })
}

#[bench]
fn bench_movegen_any_pseudolegal(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_ANY.iter() {
            black_box(board.generate_pseudolegal_moves());
        }
    })
}


#[bench]
fn bench_movegen_legal_all(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_moves());
        }
    })
}

#[bench]
fn bench_movegen_legal_captures(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_moves_of_type(GenTypes::Captures));
        }
    })
}

#[bench]
fn bench_movegen_legal_quiets(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_moves_of_type(GenTypes::Quiets));
        }
    })
}

#[bench]
fn bench_movegen_legal_quiet_checks(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_moves_of_type(GenTypes::QuietChecks));
        }
    })
}

#[bench]
fn bench_movegen_pslegal_all(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_pseudolegal_moves());
        }
    })
}

#[bench]
fn bench_movegen_pslegal_captures(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_pseudolegal_moves_of_type(GenTypes::Captures));
        }
    })
}

#[bench]
fn bench_movegen_pslegal_quiets(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_pseudolegal_moves_of_type(GenTypes::Quiets));
        }
    })
}

#[bench]
fn bench_movegen_pslegal_quiet_checks(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_NON_CHECKS.iter() {
            black_box(board.generate_pseudolegal_moves_of_type(GenTypes::QuietChecks));
        }
    })
}

#[bench]
fn bench_movegen_in_check_legal_evasions(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_CHECKS.iter() {
            black_box(board.generate_moves());
        }
    })
}

#[bench]
fn  bench_movegen_in_check_pslegal_evasions(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS_CHECKS.iter() {
            black_box(board.generate_pseudolegal_moves());
        }
    })
}



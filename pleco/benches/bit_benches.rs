use std::time::Duration;

use criterion::{black_box, Bencher, Criterion, Fun};
use pleco::core::bit_twiddles::*;
use pleco::core::bitboard::{BitBoard, RandBitBoard};

fn popcount_rust(b: &mut Bencher, data: &Vec<BitBoard>) {
    b.iter(|| {
        {
            for bits in data.iter() {
                black_box({
                    black_box(black_box(bits.0)).count_ones();
                })
            }
        };
        black_box(())
    });
}

fn popcount_old_8(b: &mut Bencher, data: &Vec<BitBoard>) {
    b.iter(|| {
        {
            for bits in data.iter() {
                black_box({
                    popcount_table(black_box(bits.0));
                })
            }
        };
        black_box(())
    });
}

fn popcount(c: &mut Criterion) {
    let bit_set_dense_100: Vec<BitBoard> = RandBitBoard::default()
        .pseudo_random(2661634)
        .avg(6)
        .max(11)
        .many(1000);

    let popcnt_rust = Fun::new("Popcount Rust", popcount_rust);
    let popcnt_old = Fun::new("Popcount Old", popcount_old_8);
    let funs = vec![popcnt_rust, popcnt_old];

    c.bench_functions("PopCount", funs, bit_set_dense_100);
}

criterion_group!(name = bit_benches;
     config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(1));
    targets = popcount
);

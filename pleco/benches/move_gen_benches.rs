use std::time::Duration;

use criterion::{black_box, Bencher, Criterion, Fun};

use pleco::board::movegen::{Legal, Legality, MoveGen, PseudoLegal};
use pleco::core::mono_traits::*;
use pleco::Board;

fn movegen_ty<L: Legality, G: GenTypeTrait>(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter(|| {
        for board in boards.iter() {
            black_box(MoveGen::generate::<L, G>(board));
        }
    })
}

fn all_movegen(c: &mut Criterion) {
    let boards_any: Vec<Board> = RAND_BOARD_ANY_GEN
        .iter()
        .map(|b| Board::from_fen(b).unwrap())
        .collect();

    let boards_no_check: Vec<Board> = RAND_BOARD_NON_CHECKS_GEN
        .iter()
        .map(|b| Board::from_fen(b).unwrap())
        .collect();

    let boards_in_check: Vec<Board> = RAND_BOARD_IN_CHECKS_GEN
        .iter()
        .map(|b| Board::from_fen(b).unwrap())
        .collect();

    let all_legal = Fun::new("MoveGen All Legal", movegen_ty::<Legal, AllGenType>);
    let all_pslegal = Fun::new(
        "MoveGen All PseudoLegal",
        movegen_ty::<PseudoLegal, AllGenType>,
    );

    let nochk_legal = Fun::new(
        "MoveGen NonEvasions - Legal",
        movegen_ty::<Legal, NonEvasionsGenType>,
    );
    let nochk_pslegal = Fun::new(
        "MoveGen NonEvasions - PseudoLegal",
        movegen_ty::<PseudoLegal, NonEvasionsGenType>,
    );

    let nochk_captures_legal = Fun::new(
        "MoveGen Captures - Legal",
        movegen_ty::<Legal, CapturesGenType>,
    );
    let nochk_captures_pslegal = Fun::new(
        "MoveGen Captures - PseudoLegal",
        movegen_ty::<PseudoLegal, CapturesGenType>,
    );

    let nochk_quiets_legal = Fun::new("MoveGen Quiets - Legal", movegen_ty::<Legal, QuietsGenType>);
    let nochk_quiets_pslegal = Fun::new(
        "MoveGen Quiets - PseudoLegal",
        movegen_ty::<PseudoLegal, QuietsGenType>,
    );

    let nochk_quietchecks_legal = Fun::new(
        "MoveGen QuietChecks - Legal",
        movegen_ty::<Legal, QuietChecksGenType>,
    );
    let nochk_quietchecks_pslegal = Fun::new(
        "MoveGen QuietChecks - PseudoLegal",
        movegen_ty::<PseudoLegal, QuietChecksGenType>,
    );

    //    let chk_legal = Fun::new("MoveGen Evasions - Legal", movegen_ty::<Legal, AllGenType>);
    //    let chk_pslegal = Fun::new("MoveGen Evasions - PseudoLegal", movegen_ty::<PseudoLegal, AllGenType>);

    let chk_legal = Fun::new(
        "MoveGen Evasions - Legal",
        movegen_ty::<Legal, EvasionsGenType>,
    );
    let chk_pslegal = Fun::new(
        "MoveGen Evasions - PseudoLegal",
        movegen_ty::<PseudoLegal, EvasionsGenType>,
    );

    let all_funcs = vec![all_legal, all_pslegal];

    let nochk_funcs = vec![
        nochk_legal,
        nochk_pslegal,
        nochk_captures_legal,
        nochk_captures_pslegal,
        nochk_quiets_legal,
        nochk_quiets_pslegal,
        nochk_quietchecks_legal,
        nochk_quietchecks_pslegal,
    ];

    let chk_funcs = vec![chk_legal, chk_pslegal];

    c.bench_functions("Any Board", all_funcs, boards_any);
    c.bench_functions("No Checks Board", nochk_funcs, boards_no_check);
    c.bench_functions("In Checks Board", chk_funcs, boards_in_check);
}

criterion_group!(name = movegen_benches;
     config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(10));
    targets = all_movegen
);

const RAND_BOARD_ANY_GEN: [&str; 30] = [
    "3qkb1r/ppp2ppp/4bn2/8/4P3/1PNB1K1P/P1PP1PP1/R6R b k - 0 13",
    "4k1n1/3b2p1/8/1p2p3/1Q1n3r/4P3/5P1P/q3NK1R b - - 0 28",
    "rn2k3/pp1qPppr/5n2/1b2B3/8/4NP2/3NP1PP/R2K1B1R b q - 0 23",
    "rnbqkbnr/pp1ppp1p/2p5/3N2p1/8/7P/PPPPPPP1/R1BQKBNR b KQkq - 0 3",
    "3rkb1r/pp1bpppp/8/3P4/4N3/2Nq4/PP3PPP/3RK2R b Kk - 3 15",
    "3qkb1r/2pn1ppp/p1p1p3/3p1Q2/2rP4/P3P3/1PPBNPPP/1R3RK w k - 0 14",
    "2r1kb1r/1p2nppp/p2pb3/3p2P1/8/PPN2N1P/2PBBP2/R2QK2R w KQ - 3 18",
    "7r/Q5pp/3b1pk1/8/3Pq3/8/1P1PB1PP/2B1K2n w - - 2 24",
    "r3k1r1/p2b1ppp/2p5/8/p5P1/1P3P1P/1K2n1q1/4R w q - 0 31",
    "rq3b1r/p2Bpk1p/1p3p2/2pR4/2P1n2B/PN3NP1/4PP1P/4K2R w K - 1 21",
    "7r/8/p5rp/P1p1k3/2Pbpp2/8/2R2PPP/1R4K w - - 0 39",
    "r2qkbnr/1pp1pppp/p1n5/3N2B1/2PP2b1/5N2/PP2PPPP/R2QKB1R b KQkq c3 0 6",
    "r2Bk1nr/4pp2/6p1/p2p3p/3b2b1/P3PN2/1PP2PPP/R2Q1RK b kq - 0 15",
    "r1bqkbnr/ppp2ppp/2np4/4p3/4PQ2/2NP4/PPP1NPPP/R1B1KB1R w KQkq e6 0 8",
    "r1b5/ppp2Q2/3kp3/1P3ppp/3p4/3K2P1/3NP2P/3q2NR w - - 2 22",
    "r3kbnr/ppp2pp1/4p3/3pqb2/8/PPN1P3/4K1PP/Q4B1R w kq - 0 14",
    "r3kbnr/pp2ppp1/2p5/3p4/1q4N1/4PP2/3N2PP/4KB1R b kq - 2 18",
    "r1bqkb1r/pp2pp2/2p2n2/6Q1/7p/2N4P/PP1B1PP1/R3KBNR w KQkq - 0 14",
    "r3k2r/1p2pp2/8/2p4p/3q4/P1p3Q1/5PPP/3bKBNR w Kkq - 0 18",
    "3r4/3p1p2/2pk1bp1/1p1p4/p6p/P1PB1N2/1P1N1PPP/4RRK b - - 3 25",
    "5knr/p4p1p/1p4p1/3PP3/P1BPbQ1P/2P5/7P/4K b - - 3 31",
    "r4bnr/2B1pk1p/1N3p2/p2b2p1/2pP4/4P3/PPP2PPP/RQ3RK b - - 0 18",
    "5r1r/1pp2pkp/p5p1/3nn3/6q1/P7/4NR1P/3RK b - - 3 29",
    "2r1k2r/p2pbppp/1p3n2/8/3P4/P3KNP1/1q2R2P/ b k - 2 23",
    "1Qb2rk1/p2p1ppp/1p6/8/3N4/6p1/4P2P/2BNKB1R b K - 1 23",
    "8/p1p2p1r/3kb1p1/8/6p1/PP2P2P/2PpKP2/6q w - - 0 30",
    "3k4/6b1/1p5p/4p3/5rP1/6K1/8/ w - - 0 40",
    "8/1bpk1r1p/1p4R1/2n2q2/3p4/PrP1R3/4P3/5BK b - - 0 38",
    "1k6/1p1n4/p6p/4P3/2P5/1R6/5K1P/4R b - - 2 33",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
];

const RAND_BOARD_NON_CHECKS_GEN: [&str; 30] = [
    "3qkb1r/ppp2ppp/4bn2/8/4P3/1PNB1K1P/P1PP1PP1/R6R b k - 0 13",
    "4k1n1/3b2p1/8/1p2p3/1Q1n3r/4P3/5P1P/q3NK1R b - - 0 28",
    "rn2k3/pp1qPppr/5n2/1b2B3/8/4NP2/3NP1PP/R2K1B1R b q - 0 23",
    "rnbqkbnr/pp1ppp1p/2p5/3N2p1/8/7P/PPPPPPP1/R1BQKBNR b KQkq - 0 3",
    "3rkb1r/pp1bpppp/8/3P4/4N3/2Nq4/PP3PPP/3RK2R b Kk - 3 15",
    "3qkb1r/2pn1ppp/p1p1p3/3p1Q2/2rP4/P3P3/1PPBNPPP/1R3RK w k - 0 14",
    "2r1kb1r/1p2nppp/p2pb3/3p2P1/8/PPN2N1P/2PBBP2/R2QK2R w KQ - 3 18",
    "7r/Q5pp/3b1pk1/8/3Pq3/8/1P1PB1PP/2B1K2n w - - 2 24",
    "r3k1r1/p2b1ppp/2p5/8/p5P1/1P3P1P/1K2n1q1/4R w q - 0 31",
    "rq3b1r/p2Bpk1p/1p3p2/2pR4/2P1n2B/PN3NP1/4PP1P/4K2R w K - 1 21",
    "7r/8/p5rp/P1p1k3/2Pbpp2/8/2R2PPP/1R4K w - - 0 39",
    "r2qkbnr/1pp1pppp/p1n5/3N2B1/2PP2b1/5N2/PP2PPPP/R2QKB1R b KQkq c3 0 6",
    "r2Bk1nr/4pp2/6p1/p2p3p/3b2b1/P3PN2/1PP2PPP/R2Q1RK b kq - 0 15",
    "r1bqkbnr/ppp2ppp/2np4/4p3/4PQ2/2NP4/PPP1NPPP/R1B1KB1R w KQkq e6 0 8",
    "r1b5/ppp2Q2/3kp3/1P3ppp/3p4/3K2P1/3NP2P/3q2NR w - - 2 22",
    "r3kbnr/ppp2pp1/4p3/3pqb2/8/PPN1P3/4K1PP/Q4B1R w kq - 0 14",
    "r3kbnr/pp2ppp1/2p5/3p4/1q4N1/4PP2/3N2PP/4KB1R b kq - 2 18",
    "r1bqkb1r/pp2pp2/2p2n2/6Q1/7p/2N4P/PP1B1PP1/R3KBNR w KQkq - 0 14",
    "r3k2r/1p2pp2/8/2p4p/3q4/P1p3Q1/5PPP/3bKBNR w Kkq - 0 18",
    "3r4/3p1p2/2pk1bp1/1p1p4/p6p/P1PB1N2/1P1N1PPP/4RRK b - - 3 25",
    "5knr/p4p1p/1p4p1/3PP3/P1BPbQ1P/2P5/7P/4K b - - 3 31",
    "r4bnr/2B1pk1p/1N3p2/p2b2p1/2pP4/4P3/PPP2PPP/RQ3RK b - - 0 18",
    "5r1r/1pp2pkp/p5p1/3nn3/6q1/P7/4NR1P/3RK b - - 3 29",
    "2r1k2r/p2pbppp/1p3n2/8/3P4/P3KNP1/1q2R2P/ b k - 2 23",
    "1Qb2rk1/p2p1ppp/1p6/8/3N4/6p1/4P2P/2BNKB1R b K - 1 23",
    "8/p1p2p1r/3kb1p1/8/6p1/PP2P2P/2PpKP2/6q w - - 0 30",
    "3k4/6b1/1p5p/4p3/5rP1/6K1/8/ w - - 0 40",
    "8/1bpk1r1p/1p4R1/2n2q2/3p4/PrP1R3/4P3/5BK b - - 0 38",
    "1k6/1p1n4/p6p/4P3/2P5/1R6/5K1P/4R b - - 2 33",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
];

const RAND_BOARD_IN_CHECKS_GEN: [&str; 30] = [
    "3r4/2pk2p1/p1n1p3/8/P1PP2q1/4b3/3K4/ w - - 0 38",
    "3Q2Q1/5p2/1P3k2/5P2/P7/8/5R1p/3K b - - 3 44",
    "8/5P1p/2R3pk/1p6/8/1PP3PP/rBK1r3/2R w - - 1 31",
    "Q4k2/5prp/1p4p1/3p4/1P1N4/2Pq1PP1/3B3P/4KR b - - 10 30",
    "r7/ppr1kp2/2pR1Q2/8/8/PPP2NPP/4BP2/4KR b - - 12 36",
    "1k6/6bp/4B3/7P/PP5P/2rRp3/2K5/ w - - 0 49",
    "5k1r/2P2p1p/Q7/1B6/8/5r2/5K2/7q w - - 1 39",
    "5rk1/pb3p1p/1p2p3/2bpq3/8/8/5K2/ w - - 2 43",
    "3rk2r/3b4/p2B1pp1/2p5/4Q3/PP3PP1/7P/3RKBN b - - 3 29",
    "4k1nQ/1pp1pp2/p5p1/P7/1PP1q2p/8/K2rn3/7R w - - 7 40",
    "2r1kbnr/ppp1pppp/2n5/1N1p4/8/P2NPq2/1PPP1P1P/R1BKR w k - 2 12",
    "3kr3/p2rbp2/2N3p1/5p2/P4P2/7P/6R1/3R1K b - - 2 37",
    "1k6/8/4N3/8/5P2/P1P5/4P1BP/RQ2K2R b KQ - 4 36",
    "8/2kR2b1/4Q2p/2p1p1r1/P1B1P1P1/2P1PN1R/8/4K b - - 2 37",
    "3rkr2/pppb1ppp/4p2b/8/P3P1P1/7P/5q1K/R1B w - - 1 31",
    "8/4kp2/8/p1p5/P1n5/3r4/1b6/2KQ w - - 5 48",
    "3k3r/2pnb1pp/8/8/p7/2Bn4/1K6/1N1q w - - 14 46",
    "3r1rk1/1p3pb1/2p2p1p/2q5/5p2/5N1P/3B1K2/5B1R w - - 6 35",
    "5b2/p3k1p1/4pp2/8/8/2P2P2/3QN3/q3KB w - - 2 36",
    "r3kb1r/pp1npp2/7p/1B1p4/3P2PP/6P1/1q4K1/7R w kq - 3 24",
    "8/R4p2/4k3/7p/4P3/PP4B1/q5KP/ w - - 2 40",
    "r2qkr2/2p2p2/p1p1b3/3p4/P4P2/1P2bPPp/2P2K1P/3N2R w q - 0 26",
    "7r/7p/2p5/6R1/1k6/2P5/5PPP/4R1K b - - 0 34",
    "4kr2/p3q3/4p3/1P1p3p/1P1p2rP/R2Pn3/8/6KR w - - 0 46",
    "r4k2/1p4pp/2p1b1p1/1p1p4/8/7q/2n5/4K w - - 4 40",
    "3k4/8/8/6p1/P2Q2N1/6P1/2P2P2/3RKB1r b - - 0 35",
    "7r/1R1b3p/3kp2b/4P3/5Pn1/P4NP1/7P/6KR b - - 0 37",
    "3k4/5p2/8/2N5/8/4PP2/4NK1r/7b w - - 3 39",
    "8/8/2P2R2/8/p3P3/P1Q5/1Pk5/7K b - - 2 40",
    "6rk/3R3p/6p1/5p2/1P3P1P/2KB2P1/6b1/q w - - 4 39",
];

use criterion::{black_box, BatchSize, Bencher, Criterion, Fun};
use std::time::Duration;

use pleco::core::mono_traits::WhiteType;
use pleco::{Board, Player};
use pleco_engine::tables::material::{Material, MaterialEntry};
use pleco_engine::tables::pawn_table::{PawnEntry, PawnTable};

use pleco_engine::search::eval::Evaluation;

fn bench_100_pawn_evals(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter_batched(
        PawnTable::new,
        |mut t| {
            #[allow(unused_variables)]
            let mut score: i64 = 0;
            for board in boards.iter() {
                let entry: &mut PawnEntry = black_box(t.probe(board));
                score += black_box(entry.pawns_score(Player::White)).0 as i64;
                score += black_box(entry.pawns_score(Player::Black)).0 as i64;
            }
        },
        BatchSize::PerIteration,
    )
}

fn bench_100_pawn_king_evals(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter_batched(
        PawnTable::new,
        |mut t| {
            #[allow(unused_variables)]
            let mut score: i64 = 0;
            for board in boards.iter() {
                let entry: &mut PawnEntry = black_box(t.probe(board));
                score += black_box(entry.pawns_score(Player::White)).0 as i64;
                score += black_box(entry.pawns_score(Player::Black)).0 as i64;
                score += black_box(
                    entry.king_safety::<WhiteType>(board, board.king_sq(Player::White)),
                )
                .0 as i64;
            }
        },
        BatchSize::PerIteration,
    )
}

fn bench_100_material_eval(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter_batched(
        Material::new,
        |mut t| {
            #[allow(unused_variables)]
            let mut score: i64 = 0;
            for board in boards.iter() {
                let entry: &mut MaterialEntry = black_box(t.probe(board));
                score += black_box(entry.value) as i64;
            }
        },
        BatchSize::PerIteration,
    )
}

fn bench_100_eval(b: &mut Bencher, boards: &Vec<Board>) {
    b.iter_batched(
        || {
            let tp: PawnTable = black_box(PawnTable::new());
            let tm: Material = black_box(Material::new());
            (tp, tm)
        },
        |(mut tp, mut tm)| {
            #[allow(unused_variables)]
            let mut score: i64 = 0;
            for board in boards.iter() {
                score += black_box(Evaluation::evaluate(board, &mut tp, &mut tm)) as i64;
            }
        },
        BatchSize::PerIteration,
    )
}

fn bench_engine_evaluations(c: &mut Criterion) {
    let boards: Vec<Board> = RAND_BOARD_NON_CHECKS_100
        .iter()
        .map(|b| Board::from_fen(b).unwrap())
        .collect();

    let pawn_evals = Fun::new("Pawn Evaluations", bench_100_pawn_evals);
    let pawn_king_evals = Fun::new("Pawn & King Evaluations", bench_100_pawn_king_evals);
    let material_evals = Fun::new("Material Evaluations", bench_100_material_eval);
    let full_evals = Fun::new("Full Evaluation", bench_100_eval);

    let funcs = vec![pawn_evals, pawn_king_evals, material_evals, full_evals];

    c.bench_functions("Engine Evaluations", funcs, boards);
}

criterion_group!(name = eval_benches;
     config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(20));
    targets = bench_engine_evaluations
);

static RAND_BOARD_NON_CHECKS_100: [&str; 100] = [
    "3qkb1r/3ppp2/3r1np1/2Q4p/5P2/1P3B2/P1P1PP1P/R2NK2R b k - 0 22",
    "r3kb1r/1p1bpp2/1p3n1p/q2p2p1/8/PQ6/1P1NPPPP/R3KBNR w KQkq - 2 14",
    "r2qkbnr/pp2p1pp/2p1b3/3pNpB1/3P4/8/PP1NPPPP/R2QKB1R w KQkq - 2 8",
    "r1bqk2r/pppp3p/5b2/1P6/5p2/P5P1/1QP1P2P/RN2KB1R b KQkq - 2 16",
    "3rr3/2pkb3/2p1p3/p1Pn1p2/P1QP1P2/1P1KPP1p/7P/1R w - - 12 39",
    "3k3r/1r5p/6p1/1B6/1P2K3/P7/5RPP/ b - - 0 28",
    "r1bqkbnr/ppppppp1/n7/3P2p1/Q4P2/2P5/PP2P1PP/RN2KBNR b KQkq - 2 6",
    "3rk2r/pppb3p/2n1p3/1B6/3bP3/P4P2/3N2PP/4K2R b Kk - 0 22",
    "rn2kb1r/1ppqpbpp/5n2/p3Q3/8/PP1P4/1BPP1PPP/R2NKB1R b KQkq - 3 13",
    "r2qkbnr/ppp1Bppp/2n5/3p1b2/3P4/2N5/PPP1PPPP/R2QKBNR b KQkq - 0 4",
    "r3k1nr/pp1n1pbp/1qp1p1p1/6B1/P2PP1P1/1Pp2N2/2P2P2/R2QKB1R b KQkq - 0 13",
    "2r1r3/3k4/1qpn1p2/8/RP1pP3/3R1PPp/1p5P/1N4K w - - 2 39",
    "r1bqkb1r/ppp1pppp/2n5/3p2B1/P2Pn3/1P6/2P1PPPP/RN1QKBNR w KQkq - 2 5",
    "r2nk2r/1p2bppp/p3p3/8/P4nB1/1P1P2N1/2QN1PbP/R1B1K1R b Qkq - 7 21",
    "2r1k2r/pp1n2p1/5p1p/2P5/4PP2/8/PPb3PP/4KBNR b Kk - 0 19",
    "rkb4r/pp1pnppp/2npp3/8/P5P1/1P1N1N1P/3PPP2/2RQKB1R w K - 4 20",
    "7r/3b3p/Q2b1k2/2pq2p1/5p2/2P5/PP1NBPPP/3R1KR w - - 4 22",
    "r2qk1nr/1pp2pBp/8/3p4/pb1P2b1/2N5/PPP1PPPP/R2QKB1R b KQkq - 0 9",
    "8/5k1p/2p3p1/1p1p4/p4b2/5B1P/8/5K b - - 4 38",
    "2kr4/2pnr3/3p4/1p1P1B2/P3P2P/2K4P/2R5/R w - - 0 42",
    "8/pp5p/3r1bp1/1Pp1kbP1/P1B1p2P/4P3/2P2P2/3NKR w - - 5 25",
    "5rk1/3rbp1p/4p3/1N5p/5P2/1PNP2P1/1BK4P/4R b - - 3 35",
    "r1bq1b1r/p2pkppp/2p2n2/1n2N3/4p3/PPP1P3/3P1PPP/R1BQKB1R b KQ - 0 10",
    "3qkb1r/p3pppp/1r3n2/2pBn3/8/2N2PP1/PPPP1P1P/1RBQKR w k - 9 12",
    "1n1bk2r/2p3pp/p3bp2/4p3/K7/P1q2NPP/4PPB1/3R b k - 1 29",
    "r2qk2r/pppb1pp1/2n1p2p/8/1B1Pn2P/5NP1/PPP2P2/R2QKB1R b KQkq - 0 11",
    "r2qkr2/ppp1n3/6b1/7p/2P4Q/1P6/P3PPPP/R3KB1R w KQq - 3 19",
    "2N2knr/1p3ppp/2qPp3/8/p7/2PQ4/1P1P1PPP/R1B1KBNR b KQ - 0 17",
    "r1bqkbnr/pppppppp/8/6B1/1n6/8/PPP1PPPP/RN1QKBNR w KQkq - 2 5",
    "r2k1b1r/pp2ppp1/2p2n1p/3p1b2/1P1P4/q1N1PN2/2nBKPPP/2RQ1B1R w - - 0 12",
    "r4rk1/np2bppp/p3p3/2p5/2PP1q2/P3R3/1P2PPBP/R1BQK w Q - 1 17",
    "r3kb2/pppqpppr/5n1p/3p4/3P4/2N5/PPPBPPPP/R2QKB1R w KQq - 0 9",
    "r1bqkbnr/pppppppp/2n5/6B1/3P4/8/PPP1PPPP/RN1QKBNR b KQkq - 2 2",
    "r2qkr2/1bpppp2/p7/1p4pp/1P1Q4/PP2BP1P/1N2P1P1/1N2KnR w q - 0 18",
    "r1bqkr2/ppppn1pp/5p2/4p3/3P3B/1PQ2N2/P1P1PPPP/R3KB1R w KQq - 1 12",
    "r1b1kb1r/ppqp1p2/2n1p3/1B4pn/4P3/2P5/PP1N1PPP/R2QK2R b KQkq - 1 11",
    "r3kbnr/pppqpppp/6b1/1N2P3/3p2P1/8/PPPP1P1P/R1BQKB1R b KQkq - 0 8",
    "2Q5/4k1b1/6p1/5p1p/pP1P1P2/2P5/5RPP/5RK w - - 5 45",
    "r3k1nr/p1p1pp2/2N3qp/8/5pb1/bP6/3NPPPP/1R1QKB1R w Kkq - 1 16",
    "r3k1r1/4np1p/4p3/4n3/6P1/P4N1P/2NPPP2/4KB1R w K - 0 21",
    "5R2/2k5/pppr4/P3Q3/P2P2P1/2P2N2/3NP1P1/R3KB w Q - 0 33",
    "4k2r/pp2pppp/7n/3b4/8/2P4P/P3KP2/1R b - - 1 20",
    "4kr2/6R1/1pn4p/p1p1p3/2P5/P4P2/1P2BP1P/4K w - - 2 25",
    "3rkb1r/p2nnp1p/5qp1/4p3/8/1P1PN1P1/PB1PPPBP/R2QK1R w Qk - 0 16",
    "r7/pbkp1Np1/4rp1p/1Q2P3/8/P7/2P1P1PP/4KB1R b K - 0 20",
    "r1b1k2r/1p1pbp2/7p/p7/2p1P3/P5B1/1q1N1PP1/R2QKB1R w KQkq - 1 18",
    "3qkb1r/p1pnpp2/7p/6p1/3P4/PrP1PQ2/3B1PPP/R4RK w k - 0 17",
    "r3kb1r/p2ppp1p/5np1/8/1p2b3/1N2Q3/Pq3PPP/3RKBNR b Kkq - 1 14",
    "r7/pk6/2b1p3/5p2/P6P/4P2N/2P3r1/2K3n w - - 0 36",
    "r3kb1r/p3pppp/2p2n2/Rp6/3P1B2/1PP2b2/1P2PPPP/3QKB1R w Kkq - 0 14",
    "4k2r/2Pb1ppp/4p3/3p4/1P1P1B2/r3P1PB/1R1K1P1P/ w - - 1 25",
    "4kb2/p1p1pp1r/6p1/8/PP1qn3/1p3Q2/6PP/R4R1K w - - 0 24",
    "4r1k1/p4p1p/2p3p1/1p6/1P2B3/P3BP2/1P2KP1P/5R b - - 1 25",
    "5b1r/1N2nppp/5k2/2q5/p3Q3/P1P5/1P3PPP/R4RK b - - 4 22",
    "8/ppQ5/k7/5p2/7P/P3P1P1/3NP3/R3KB b Q - 3 36",
    "3k1b2/4p1p1/2B5/5p2/Pr6/2N5/R1P2PPP/4K2R b K - 4 26",
    "r2qkb2/1ppbpp2/p6r/3p4/6P1/1PP1P1QP/P2N1P2/RN2KB1R b KQq - 4 20",
    "4kb1B/4pp2/7p/8/1p6/2N1q3/2K1N1PP/7R w - - 0 26",
    "8/4p1bk/2P1Q3/5P1p/1q2P3/8/2P1K2P/6r w - - 3 36",
    "r2qk2r/Q2ppp1p/1p4p1/2P5/8/P2BPN2/1RP3PP/1N2K2R w Kk - 1 22",
    "5q1r/p1p1k2p/2p1bb1Q/3pp3/P1P5/1r6/3N2PP/5RK w - - 2 25",
    "3qkr2/2p2pb1/2p1ppN1/3p4/3P2P1/4r2P/2pK1P2/1R1Q1R b - - 1 21",
    "3k4/2pqpp2/6p1/pp2n3/8/8/Pb2QPB1/5K b - - 3 31",
    "3r4/1p3kbp/1p2p1p1/5q2/2Pp4/PP2PP2/4Q1PP/R3K2R w KQ - 0 20",
    "r5kr/pp2b1p1/2p5/2P4p/5B2/P5P1/1P2qPB1/1RR3K w - - 1 21",
    "1B2kb1r/p2p2p1/b1q1p3/5n1p/P1p1N3/2P2PP1/7P/R2QK1NR b k - 0 20",
    "3r1k1r/Q4ppp/3b4/8/6N1/PP1P2P1/K2N1P1P/4R2R w - - 3 28",
    "3rkb1r/p2nnp2/6pp/8/7P/PPP1P3/2qB1P2/R3K1R w Qk - 2 22",
    "r2qk2r/p1pb1ppp/p4n2/4p3/3bP3/Pn1P3P/1P3PP1/2B2KNR w kq - 1 14",
    "2kr4/1pp5/2b1p1pr/p2pPpRp/P4P1P/1PP1P3/3K4/2RQ b - - 0 31",
    "r7/pp1n3p/2b2p2/2b4Q/5PP1/6kP/PB2P2R/R3KB b Q - 2 22",
    "R7/p2kn2R/2p1p3/2b5/1P2p3/8/1PP2P2/1K1N1B b - - 0 34",
    "r2qkr1b/2pppp2/1pQ3p1/p5Bp/8/2P2N2/PP2PPPP/1N1RKB1R b Kq - 0 14",
    "r3k1n1/pp6/2p5/q2pb2r/4p1K1/1P2P1B1/P4PPP/1Q3R b q - 3 22",
    "8/k7/ppp5/2b1n1p1/8/2P3PP/2B1K3/2R w - - 0 44",
    "1r1kr3/p2p2pp/4Bn2/5P2/NP6/8/5PPP/3RK2R b K - 0 22",
    "3r2kr/5p1p/8/3Pn1PB/2p5/7q/7P/3RK1R b - - 1 35",
    "r3kb1Q/1p1bppp1/8/4q3/p3N3/n4PP1/P1p1B1KP/3R w q - 0 29",
    "r3kr2/pppn1pQ1/8/4p3/3pNq2/1P1B4/P2R1PPP/4KR b q - 1 24",
    "8/8/5k2/5p2/P7/3P1PP1/3QPP2/4KB w - - 6 39",
    "6nr/pQbk4/2N1ppB1/1N5p/3P4/P7/1P1P1PPP/R1B1K2R b KQ - 4 21",
    "r3kr2/p3pp2/1qbp2p1/1p5p/1P3B2/P1P2PP1/1N5P/R2Q1K b q - 0 24",
    "2r5/3k1p2/2p2P1p/p2r4/3P4/P1p4P/4R3/3BK w - - 1 39",
    "5r2/b4kp1/p1p4p/1p3P2/1P2R3/2N1PK1P/5P2/2N w - - 1 28",
    "2n1qk2/p4pb1/6p1/1p2P3/5P2/4P1P1/PPP2K1R/R4Q b - - 0 22",
    "r2qkb2/p5pQ/5p1p/4p3/4p3/P3PN2/1P1B2PP/3NKB1R w Kq - 0 27",
    "3k4/3rn3/1p6/p6p/1r2PP2/2R3P1/PP5P/1K2Q1R b - - 1 31",
    "3qkb2/2p1pp2/p4n1p/3P4/5r2/4NP2/P3PKPP/2R2B1R w - - 5 22",
    "r1b1k2r/p3b2p/8/1P6/2p1NP2/4Pp2/PP3P1P/2R1KBR w kq - 0 25",
    "5k2/5p2/p2p3p/8/2pP4/5N2/7P/1K w - - 0 43",
    "r7/p1p5/2k2p2/3p2rp/4PNP1/P1P2P2/7P/3R2K b - - 0 33",
    "7r/6kp/2n2p2/p4r2/1P2NP2/P5KP/5RP1/3R w - - 2 45",
    "8/7p/5pk1/6p1/8/3R4/p1K5/6r w - - 0 43",
    "4k2r/R4p2/4p2p/2Pq2p1/3P1bP1/7P/5BB1/6KR b - - 2 34",
    "5r2/prk5/2pn3p/5pp1/6R1/1P4P1/PKP4P/7R w - f6 0 35",
    "5k2/1pp2p2/6p1/p4n2/5R2/2N2P2/6PP/4K b - - 2 34",
    "r3r3/2Q2pp1/p6k/3p4/2p1pP1P/4P1P1/2RKB3/1q w - - 0 41",
    "r6k/pp5p/6p1/2p2b2/2n5/8/7P/K w - - 0 39",
    "5r2/1b1rkp2/3pp3/2p3R1/p3P3/5PP1/q3BKP1/4Q1N b - - 1 38",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
];

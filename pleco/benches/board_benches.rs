use criterion::{black_box, Criterion};
use lazy_static;
use std::time::Duration;

use pleco::tools::prng::PRNG;
use pleco::{BitMove, Board, MoveList, Player};

pub const SEED: u64 = 5363310003543;

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        let mut prng = PRNG::init(SEED);
        let mut boards = RAND_BOARD_FENS
            .iter()
            .map(|b| Board::from_fen(b).unwrap())
            .collect::<Vec<Board>>();

        boards.iter_mut().for_each(|b| {
            let moves = b.generate_moves();
            b.apply_move(moves[prng.rand() as usize % moves.len()]);
        });
        boards
    };
}

fn bench_board_100_clone(c: &mut Criterion) {
    lazy_static::initialize(&RAND_BOARDS);
    c.bench_function("Board Clone 100", |b| {
        b.iter(|| {
            for board in RAND_BOARDS.iter() {
                black_box(board.shallow_clone());
            }
        })
    });
}

fn bench_find(c: &mut Criterion) {
    lazy_static::initialize(&RAND_BOARDS);
    c.bench_function("Board find King SQ", |b| {
        b.iter(|| {
            {
                for board in RAND_BOARDS.iter() {
                    black_box(board.king_sq(Player::Black));
                }
            };
            black_box(())
        })
    });
}

fn bench_apply_100_move(c: &mut Criterion) {
    lazy_static::initialize(&RAND_BOARDS);
    c.bench_function("Board Apply 100 Move", |b| {
        let mut prng = PRNG::init(SEED);
        let mut board_move: Vec<(Board, BitMove)> = Vec::with_capacity(100);

        for board in RAND_BOARDS.iter() {
            let moves: Vec<BitMove> = MoveList::into(board.generate_moves());
            let bit_move = *moves.get(prng.rand() as usize % moves.len()).unwrap();
            board_move.push((board.parallel_clone(), bit_move));
        }

        b.iter(|| {
            {
                for t in board_move.iter() {
                    let board: &Board = &(t.0);
                    black_box(board.clone()).apply_move(t.1);
                    black_box(());
                }
            };
            black_box(())
        })
    });
}

fn bench_undo_100_move(c: &mut Criterion) {
    lazy_static::initialize(&RAND_BOARDS);
    c.bench_function("Board Undo 100 Move", |b| {
        let mut boards: Vec<Board> = Vec::with_capacity(100);
        for board in RAND_BOARDS.iter() {
            boards.push(board.parallel_clone());
        }

        b.iter(|| {
            {
                for board in boards.iter_mut() {
                    black_box(board.parallel_clone()).undo_move();
                    black_box(());
                }
            };
            black_box(())
        })
    });
}

criterion_group!(name = board_benches;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_millis(10));
    targets =
        bench_board_100_clone,
        bench_find,
        bench_apply_100_move,
        bench_undo_100_move
);

static RAND_BOARD_FENS: [&str; 100] = [
    "r3kb1r/3qpppp/p1Qp4/1p6/3Pp3/7P/PP1PP2P/R1B1KB1R w KQkq - 2 13",
    "r4rk1/p3q2n/1pQ1P3/5p2/8/2B5/PPP2P1P/3RKR w - - 1 26",
    "rnbqkbnr/ppp1pppp/8/3p4/8/P1N5/1PPPPPPP/R1BQKBNR b KQkq - 0 2",
    "7r/1B2r1kp/1p4p1/2p5/2P1P3/6R1/P4PP1/5RK w - - 2 33",
    "2r4r/pp2k2p/3p4/3P1QBp/5P2/P1P4P/3K4/6R b - - 2 31",
    "2b1r3/1pkn4/1b2B2p/2p1p3/4P1P1/5N1P/6P1/5K w - - 2 33",
    "r1bqkb1r/pppnnppp/3pp3/1Q4B1/8/P1P2P2/1P2PKPP/RN3BNR b kq - 3 11",
    "r1b2k1r/1p4pp/p2p4/2p5/2Q2B2/P7/1P2PqPP/R2K1B1R b - - 1 22",
    "r2qk1nr/pppnbppp/8/4pbN1/3p4/P2P1NP1/1PP1PP1P/R1BQKB1R b KQkq - 0 7",
    "8/5P2/k7/8/p1P5/P3N2p/5P1P/6K w - - 0 43",
    "r3kb1r/p1pp1ppp/1p6/3b4/8/P1P5/1q1NN1PP/R2QKB1R w kq - 0 14",
    "rnb1kb1r/pp1pp1pp/5n2/q7/2P2p2/2N1BP2/P1PKP1PP/1R1Q1BNR w kq - 0 9",
    "r2qkbnr/ppp1pppp/2n5/3p1bB1/3P4/2N5/PPP1PPPP/R2QKBNR w KQkq - 2 4",
    "r4b1r/p1qpkpp1/1p3p1p/3P4/1p6/8/RP1NPPPP/4KBNR w K - 2 18",
    "rn2k1nr/pppb1ppp/8/6b1/3qP2P/PPNP4/3PBPP1/R1BQK2R b KQkq - 0 11",
    "r2qkbnr/ppp1pp2/2n4p/1B1p2p1/3P1B2/2N1PQ2/PPP2PPP/R3K2R w KQkq - 0 9",
    "6nr/R3pkb1/2p2pp1/1p1r3p/1P1PNB2/2P1P3/1P1N1P2/3QKB1q w - - 1 23",
    "r4r2/pp1nkpbp/2p3p1/3p4/PP1N3P/5NP1/2p2P2/R3KR b Q - 1 23",
    "rnbqkbnr/ppp1pp1p/6p1/3p4/8/2N2N2/PPPPPPPP/R1BQKB1R w KQkq - 0 3",
    "R4b1r/3k2pp/2p1p3/6P1/4PN2/1Pp5/3r1P1P/4K2R b K - 1 25",
    "r3kb1r/p1Rbpp2/5n1p/3p2p1/1n1N4/8/1P1NPPPP/4KB1R b Kkq - 3 15",
    "r3kb1r/1pp1pppp/p2q1n2/3p2B1/8/P1N3P1/1PP1BP1P/R2QK2R w KQkq - 1 10",
    "1r2k3/6pp/2p5/p7/P2p3P/1P4P1/4KP2/1R w - - 0 36",
    "r1bqkb2/pp2p2r/3p3p/3Q1pp1/8/1N6/nPP1PPPP/R3KBNR b KQq - 0 12",
    "7Q/p1k5/1q5p/3B4/3PP1Pp/R4N2/5P2/5RK b - - 0 32",
    "r2k3r/2p1b1p1/p1p1Q3/4p1p1/1P6/P2q1P2/6PP/3R1R1K b - - 1 24",
    "rn1qkbnr/ppp1pppp/8/3p4/6b1/2N2N2/PPPPPPPP/R1BQKB1R w KQkq - 2 3",
    "1rb2k1r/pp2bp1p/2nQ1n2/4p1p1/4P3/1PB3P1/P1P1NPBP/R3K2R b - - 0 14",
    "5q1k/r1p2p1p/p7/P1RP2Q1/3P4/8/r4PPP/5RK w - - 1 28",
    "r4rk1/1Qp1bppp/8/p7/3q2b1/P7/1P1NPPPP/R3KB1R b KQ - 0 13",
    "B3kbnr/p3pppp/3p4/5q2/3p4/8/PPPPPP1P/R1BQK1R b Q - 2 12",
    "5N2/2k5/8/8/3PQ3/2P5/PP1BPP2/3RKB w - - 7 40",
    "r2qkbnr/4pppp/p2p4/3Q1bP1/3P4/2P2N2/PP2PP1P/R1B1K2R w KQkq - 0 13",
    "r2k1r2/p1pbn1pQ/1n6/1p6/4P3/1P4P1/P2N1P1P/R3KR b Q - 0 25",
    "r2qkb1r/ppp1np1p/4b1p1/8/2N5/1P3N2/R3PPPP/4KB1R w Kkq - 0 13",
    "r1b1kb1r/p2ppppp/1p2q3/2p5/5P2/P1QP4/3NNBPP/R3KB1R w KQkq - 2 15",
    "rnbqkbnr/pp1ppppp/2p5/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2",
    "r4k2/pp2p1pr/4p2p/3n4/1P6/P3PPBP/3R2P1/6K b - - 4 31",
    "r3k2r/1p2pp1p/pB3n1b/8/1PP1nq2/P3K3/7P/2R4R w kq - 0 23",
    "r1bqkb1r/pppppppp/5n2/4n3/8/4P3/PPPPBPPP/RNBQK1NR w KQkq - 5 4",
    "r1b2knr/pppp4/5Q1p/3P4/1n6/2K1P3/PPP2PPP/R4B1R b - - 0 15",
    "r1bqkbnr/pp1pp1pp/8/2Pp1pB1/8/P4N2/2PNPPPP/R2QKB1R b KQkq - 1 8",
    "R1kr2n1/2p5/2n5/1p4p1/3P2N1/8/1PP1PPPP/3QKB1R b K - 2 16",
    "3rkb1r/1pp1nppp/p2p4/4p1P1/4N2q/1P1P1P2/1PPBKR2/R2Q w k - 1 22",
    "1r2kr2/p2R1pbp/6p1/8/4p3/P3p1P1/2P1P2P/4KBNR b K - 0 23",
    "rn1qkbnr/ppp1pppp/8/3p4/6b1/8/PPPPPPPP/RNBQKBNR w KQkq - 2 3",
    "7r/1r2b3/2p2k2/1p2p3/P5p1/1QP1B3/1P2PPPP/R3KB1R b KQ a3 0 21",
    "3rkb1r/pp1npppp/2p2n2/8/8/1P6/P1qPBPPP/BN1RKR b k - 1 12",
    "r4rk1/1ppb1p1p/n2qpp2/8/8/1PP5/3NPPPP/R2QKBNR b K - 0 15",
    "r4bkr/p5pp/1p2p3/2p5/P7/1PP5/4qNPP/5RKR w - - 0 22",
    "r4rk1/ppp2ppp/2np4/2b2bPB/N7/P3P3/1PPN1PP1/R2QKR w Q - 2 17",
    "r1b1k3/pppp3Q/8/8/P3n3/4P3/2PB2PP/1R1K1R w - - 1 29",
    "1r2k2r/p2b2pp/6p1/8/1P3p2/2p1P3/5PP1/3RKB1R b Kk - 0 24",
    "r2qkr2/1Bp2p1p/p2p1bp1/8/2P3n1/PP4P1/3PP2P/RNBQK1NR b q - 0 18",
    "3q2r1/r2bk2p/1pn2p1B/pN2p3/2PP4/1P3B2/P2Q1PPP/R3KR b Q - 3 21",
    "r3kbnr/pp1np2p/2q2pp1/5b2/3PP3/2P2Q2/PP1N1P1P/R1B1KB1R b KQkq - 1 14",
    "3N1R2/pp1kp3/2p5/4P1p1/6r1/2P5/PP4PP/R5K b - - 0 22",
    "4k2r/2pr3p/2p2p2/B4R2/R2bp3/1P6/4PPP1/4KBN w - - 1 28",
    "1q2k2B/p2bnp1p/8/3p4/1bp5/5N2/RP1NPPPP/4KB1R w K - 0 15",
    "5rk1/p4pnp/1p4p1/1Pp1P3/8/PP2pN1P/3rP1P1/2R1KB1R b K - 4 21",
    "3rkb1r/p5pp/2p1ppb1/2N1P1B1/3P4/2P3N1/P2K2PP/R4B1R w k - 0 23",
    "3k4/3qp3/1p6/2p1P1p1/Q1P3P1/2P2R2/3B2K1/ w - - 0 36",
    "7r/3nkppp/2Qb4/4p3/8/1P6/P3PPPP/R2K1B1R b - - 4 18",
    "r6r/pppn2p1/2k4p/3pP3/3P4/b1N1P1Pq/2PB3P/1K1R1R b - - 0 19",
    "b3k3/p3ppb1/8/2p5/4N3/2P2P2/1P2P3/4K2B b - - 0 24",
    "r3k1nr/3qpp2/1p1p3p/p2P4/P5PB/5BPP/6K1/4Q2R w kq - 1 29",
    "1q2kr2/3b1p1p/4pnp1/Q1pp4/1P1P1P2/1PN1P3/5K1P/3B2NR w - - 0 22",
    "Q7/1p4pk/5p2/3B3P/3P4/7P/5P2/4K w - - 0 35",
    "rn6/pp6/1k5p/1B6/1P2R1p1/4PK2/4N1PP/2R w - - 0 37",
    "r1b4r/pp1k3p/4pP2/3pB3/1Pn5/8/R3PPP1/3Q1KNR w - - 1 16",
    "r2qk2r/pp2pp2/2p2n1p/3pP2P/7n/PPPB4/3NKP1P/R2Q2R w kq - 5 18",
    "r4kn1/p2Q1p1p/6r1/8/4P2P/P4pP1/5P2/6K w - - 1 41",
    "r2qkb1r/p1pnnp1p/1p2b1p1/2P5/3P4/8/PP1BQPPP/R3KB1R b KQkq - 0 14",
    "r3k2r/Pp3p2/7p/5bp1/2P5/1K2b1p1/4P1BP/7n b kq - 3 29",
    "1k5r/pp3p2/5n2/3Pp3/P1R4P/1K1B4/6P1/7R b - - 2 26",
    "8/6pp/p6k/2p5/4BbP1/PP5P/2K5/7R w - - 1 34",
    "3rr3/p1k4p/p1p3p1/3pP3/5P2/1PPR4/P4P2/6K b - - 2 30",
    "4kr2/3qpp1p/p7/1pPp4/8/P3R1P1/5P1P/4K1R b - - 1 30",
    "r3kb1r/p3pppp/5n2/8/8/PPPNB3/1Q2KPqP/R2R b kq - 1 22",
    "r6r/ppknqp2/3p3p/2pn2p1/1b2N3/1P1P1PPP/4BP2/1R1QKR w - - 1 23",
    "8/2p2p2/1p3k1p/4n3/1r2P2P/5p2/1PP2P2/1K2R2R w - - 3 34",
    "r6k/2p3pp/1p3n2/p2q4/8/P1P1P1P1/P6P/2K2R1b b - - 1 26",
    "8/3B2pk/7p/5Pp1/4Q3/7P/2p2P2/3RK1R b - - 0 32",
    "4k3/p3r3/2p5/7p/1rPNP3/1P3P2/P5PP/R4RK b - - 2 27",
    "4qkr1/p3r1pp/1p3p2/8/3P1P2/Q6P/PP3K2/4R b - - 16 30",
    "5r2/6bk/6r1/8/p2P3P/P3P1P1/1PQ2P2/3RKR b - - 2 29",
    "r1bkr3/p2n4/2p1p2p/1p1p1n2/3P1Pp1/PBP1P1P1/2P3KP/2R2Q1R w - - 1 28",
    "4r3/3kb1pp/1qpp4/1p2pP2/1P4P1/r1NP1P2/7P/B4RK w - - 1 26",
    "r3kb2/ppp1ppr1/7p/4Nb2/4pP2/PP2qP2/4N2P/2R1K2R w Kq - 5 24",
    "4k3/8/4Pp2/1p1K4/8/1P1P4/4r2r/ b - - 0 48",
    "8/2k5/1p4p1/1P1p3p/3K3P/3BP1P1/5p2/3R3R w - - 2 35",
    "3k3r/p2bb2p/3p4/8/p4Rp1/2P2qP1/4N3/4K b - - 9 35",
    "5k2/6p1/p3q2p/r7/4P3/2p2PP1/4BKP1/4R w - - 4 38",
    "3k4/8/1p6/p7/4P2p/P2b1P2/7b/4K w - - 2 41",
    "8/5k2/8/P1P1P3/1P2QP2/4PKP1/6P1/R w - - 3 40",
    "r3kb1r/1p1n2p1/7p/Pp1R4/5B1P/5NP1/8/RK w k - 0 36",
    "2r2k2/p2Qnp1p/8/1P2r3/P7/1P3B2/3R1PPP/1N3RK b - - 2 35",
    "8/5p2/2p3k1/7p/2BP4/P1P1N2P/1P1K4/ b - - 2 35",
    "3r1rn1/p2pk1p1/4p2p/4R2P/1bp3P1/5N2/1B1PB3/4KR b - - 1 33",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
];

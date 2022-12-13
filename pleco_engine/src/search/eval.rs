//! Evaluation function!
//!
//!
//!
use std::fmt;
use std::mem;

use pleco::core::masks::*;
use pleco::core::mono_traits::*;
use pleco::core::score::*;
use pleco::{BitBoard, Board, File, Piece, PieceType, Player, Rank, SQ};

use pleco::helper::prelude::*;

use tables::material::*;
use tables::pawn_table::{PawnEntry, PawnTable};

const CENTER: BitBoard = BitBoard((FILE_D | FILE_E) & (RANK_4 | RANK_5));
const QUEEN_SIDE: BitBoard = BitBoard(FILE_A | FILE_B | FILE_C | FILE_D);
const CENTER_FILES: BitBoard = BitBoard(FILE_C | FILE_D | FILE_E | FILE_F);
const KING_SIDE: BitBoard = BitBoard(FILE_E | FILE_F | FILE_G | FILE_H);

const KING_FLANK: [BitBoard; FILE_CNT] = [
    QUEEN_SIDE,
    QUEEN_SIDE,
    QUEEN_SIDE,
    CENTER_FILES,
    CENTER_FILES,
    KING_SIDE,
    KING_SIDE,
    KING_SIDE,
];

const KING_ATTACKS_WEIGHT: [i32; PIECE_TYPE_CNT] = [0, 0, 78, 56, 45, 11, 0, 0];

const MOBILITY_BONUS: [[Score; 32]; PIECE_TYPE_CNT] = [
    [Score::ZERO; 32], // No Piece
    [Score::ZERO; 32], // Pawns
    [
        Score(-75, -76),
        Score(-57, -54),
        Score(-9, -28),
        Score(-2, -10),
        Score(6, 5),
        Score(14, 12), // Knights
        Score(22, 26),
        Score(29, 29),
        Score(36, 29),
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
    ],
    [
        Score(-48, -59),
        Score(-20, -23),
        Score(16, -3),
        Score(26, 13),
        Score(38, 24),
        Score(51, 42), // Bishops
        Score(55, 54),
        Score(63, 57),
        Score(63, 65),
        Score(68, 73),
        Score(81, 78),
        Score(81, 86),
        Score(91, 88),
        Score(98, 97),
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
    ],
    [
        Score(-58, -76),
        Score(-27, -18),
        Score(-15, 28),
        Score(-10, 55),
        Score(-5, 69),
        Score(-2, 82), // Rooks
        Score(9, 112),
        Score(16, 118),
        Score(30, 132),
        Score(29, 142),
        Score(32, 155),
        Score(38, 165),
        Score(46, 166),
        Score(48, 169),
        Score(58, 171),
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
    ],
    [
        Score(-39, -36),
        Score(-21, -15),
        Score(3, 8),
        Score(3, 18),
        Score(14, 34),
        Score(22, 54), // Queens
        Score(28, 61),
        Score(41, 73),
        Score(43, 79),
        Score(48, 92),
        Score(56, 94),
        Score(60, 104),
        Score(60, 113),
        Score(66, 120),
        Score(67, 123),
        Score(70, 126),
        Score(71, 133),
        Score(73, 136),
        Score(79, 140),
        Score(88, 143),
        Score(88, 148),
        Score(99, 166),
        Score(102, 170),
        Score(102, 175),
        Score(106, 184),
        Score(109, 191),
        Score(113, 206),
        Score(116, 212),
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
        Score::ZERO,
    ],
    [Score::ZERO; 32], // King
    [Score::ZERO; 32], // All piece
];

const KING_PROTECTOR: [Score; PIECE_TYPE_CNT] = [
    Score(0, 0),
    Score(0, 0),
    Score(3, 5),
    Score(4, 3),
    Score(3, 0),
    Score(1, -1),
    Score(0, 0),
    Score(0, 0),
];

// Outpost[knight/bishop][supported by pawn] contains bonuses for minor
// pieces if they can reach an outpost square, bigger if that square is
// supported by a pawn. If the minor piece occupies an outpost square
// then score is doubled.
const OUTPOST: [[Score; 2]; 2] = [
    [Score(22, 6), Score(36, 12)], // Knight
    [Score(9, 2), Score(15, 5)],   // Bishop
];

const ROOK_ON_FILE: [Score; 2] = [Score(20, 7), Score(45, 20)];

// ThreatByMinor/ByRook[attacked PieceType] contains bonuses according to
// which piece type attacks which one. Attacks on lesser pieces which are
// pawn-defended are not considered.
const THREAT_BY_MINOR: [Score; PIECE_TYPE_CNT] = [
    Score(0, 0),
    Score(0, 0),
    Score(0, 33),
    Score(45, 43),
    Score(46, 47),
    Score(72, 107),
    Score(48, 118),
    Score(0, 0),
];

const THREAT_BY_ROOK: [Score; PIECE_TYPE_CNT] = [
    Score(0, 0),
    Score(0, 0),
    Score(0, 25),
    Score(40, 62),
    Score(40, 59),
    Score(0, 34),
    Score(35, 48),
    Score(0, 0),
];

// ThreatByKing[on one/on many] contains bonuses for king attacks on
// pawns or pieces which are not pawn-defended.
const THREAT_BY_KING: [Score; 2] = [Score(3, 62), Score(9, 138)];

// Passed[mg/eg][Rank] contains midgame and endgame bonuses for passed pawns.
// We don't use a Score because we process the two components independently.
const PASSED: [[Value; RANK_CNT]; 2] = [
    [0, 5, 5, 31, 73, 166, 252, 0],
    [0, 7, 14, 38, 73, 166, 252, 0],
];

// PassedFile[File] contains a bonus according to the file of a passed pawn
const PASSED_FILE: [Score; FILE_CNT] = [
    Score(9, 10),
    Score(2, 10),
    Score(1, -8),
    Score(-20, -12),
    Score(-20, -12),
    Score(1, -8),
    Score(2, 10),
    Score(9, 10),
];

//const PASSED_RANK: [Score; FILE_CNT] = [
//    Score(0, 0), Score(5, 7), Score(5, 13), Score(32, 42),
//    Score(70, 70), Score(172, 170), Score(217, 269), Score(0, 0)
//];

const PASSED_DANGER: [i32; RANK_CNT] = [0, 0, 0, 2, 7, 12, 19, 0];

// Assorted bonuses and penalties used by evaluation
const MINOR_BEHIND_PAWN: Score = Score(16, 0);
const BISHOP_PAWNS: Score = Score(8, 12);
const CONNECTIVITY: Score = Score(3, 1);
const LONG_RANGED_BISHOP: Score = Score(22, 0);
const ROOK_ON_PAWN: Score = Score(8, 24);
const TRAPPED_ROOK: Score = Score(92, 0);
const WEAK_QUEEN: Score = Score(50, 10);
const CLOSE_ENEMIES: Score = Score(7, 0);
const PAWNLESS_FLANK: Score = Score(20, 80);
const THREAT_BY_SAFE_PAWN: Score = Score(192, 175);
const THREAT_BY_RANK: Score = Score(16, 3);
const HANGING: Score = Score(48, 27);
const WEAK_UNOPOSSED_PAWN: Score = Score(5, 25);
const SLIDER_ON_QUEEN: Score = Score(42, 21);
const THREAT_BY_PAWN_PUSH: Score = Score(38, 22);
const THREAT_BY_ATTACK_ON_QUEEN: Score = Score(38, 22);
const HINDER_PASSED_PAWN: Score = Score(7, 0);
const TRAPPED_BISHOP_A1H1: Score = Score(50, 50);

// Penalties for enemy's safe checks
const QUEEN_SAFE_CHECK: i32 = 780;
const ROOK_SAFE_CHECK: i32 = 880;
const BISHOP_SAFE_CHECK: i32 = 435;
const KNIGHT_SAFE_CHECK: i32 = 790;

const LAZY_THRESHOLD: Value = 1500;
const SPACE_THRESHOLD: Value = 12222;

#[repr(u8)]
#[derive(Copy, Clone)]
enum EvalPasses {
    Pawn = 0,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
    Material = 8,
    Imbalance = 9,
    Mobility = 10,
    Threat = 11,
    Passed = 12,
    Space = 13,
    Initiative = 14,
    Total = 15,
}

const EVAL_PASSES_CNT: usize = 16;

struct Tracer {
    a: [[Score; EVAL_PASSES_CNT]; PLAYER_CNT],
    used: bool,
}

struct PassScore {
    pass: EvalPasses,
    score_white: Score,
    score_black: Score,
}

impl fmt::Display for PassScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.pass {
            EvalPasses::Material
            | EvalPasses::Imbalance
            | EvalPasses::Initiative
            | EvalPasses::Total => write!(f, " ----  ---- | ----  ---- ")?,
            _ => write!(f, "{} | {}", self.score_white, self.score_black)?,
        }
        write!(f, " | {}", self.score_white - self.score_black)
    }
}

impl Tracer {
    pub fn new() -> Self {
        Tracer {
            a: unsafe { mem::zeroed() },
            used: true,
        }
    }
    pub fn add_piece(&mut self, piece: PieceType, player: Player, score: Score) {
        self.a[player as usize][piece as usize] = score;
    }

    pub fn add(&mut self, pass: EvalPasses, player: Player, score: Score) {
        self.a[player as usize][pass as usize] = score;
    }

    pub fn add_both(&mut self, pass: EvalPasses, white: Score, black: Score) {
        self.a[0][pass as usize] = white;
        self.a[1][pass as usize] = black;
    }

    pub fn add_one(&mut self, pass: EvalPasses, white: Score) {
        self.a[0][pass as usize] = white;
    }

    pub fn term(&self, pass: EvalPasses) -> PassScore {
        PassScore {
            pass,
            score_white: self.a[0][pass as usize],
            score_black: self.a[1][pass as usize],
        }
    }
}

impl fmt::Display for Tracer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "     Term    |    White    |    Black    |    Total   ")?;
        writeln!(f, "             |   MG    EG  |   MG    EG  |   MG    EG ")?;
        writeln!(f, " ------------+-------------+-------------+------------")?;
        writeln!(f, "    Material | {}", self.term(EvalPasses::Material))?;
        writeln!(f, "   Imbalance | {}", self.term(EvalPasses::Imbalance))?;
        writeln!(f, "  Initiative | {}", self.term(EvalPasses::Initiative))?;
        writeln!(f, "       Pawns | {}", self.term(EvalPasses::Pawn))?;
        writeln!(f, "     Knights | {}", self.term(EvalPasses::Knight))?;
        writeln!(f, "     Bishops | {}", self.term(EvalPasses::Bishop))?;
        writeln!(f, "       Rooks | {}", self.term(EvalPasses::Rook))?;
        writeln!(f, "      Queens | {}", self.term(EvalPasses::Queen))?;
        writeln!(f, "    Mobility | {}", self.term(EvalPasses::Mobility))?;
        writeln!(f, " King safety | {}", self.term(EvalPasses::King))?;
        writeln!(f, "     Threats | {}", self.term(EvalPasses::Threat))?;
        writeln!(f, "      Passed | {}", self.term(EvalPasses::Passed))?;
        writeln!(f, "       Space | {}", self.term(EvalPasses::Space))?;
        writeln!(f, " ------------+-------------+-------------+------------")?;
        writeln!(f, "       Total | {}", self.term(EvalPasses::Total))
    }
}

trait Tracing {
    fn trace(&mut self) -> Option<&mut Tracer>;

    fn new() -> Self;
}

struct NoTrace {}

struct Trace {
    t: Tracer,
}

impl Tracing for NoTrace {
    fn trace(&mut self) -> Option<&mut Tracer> {
        None
    }

    fn new() -> Self {
        NoTrace {}
    }
}

impl Tracing for Trace {
    fn trace(&mut self) -> Option<&mut Tracer> {
        Some(&mut self.t)
    }

    fn new() -> Self {
        Trace { t: Tracer::new() }
    }
}

pub struct Evaluation {}

impl Evaluation {
    pub fn evaluate(board: &Board, pawn_table: &mut PawnTable, material: &mut Material) -> Value {
        let pawn_entry = { pawn_table.probe(&board) };
        let material_entry = { material.probe(&board) };
        let mut no_trace = NoTrace::new();
        let mut eval =
            EvaluationInner::<NoTrace>::new(board, pawn_entry, material_entry, &mut no_trace);
        eval.value()
    }

    pub fn trace(board: &Board) {
        let mut pawn_table = PawnTable::new();
        let mut material = Material::new();
        let pawn_entry = { pawn_table.probe(&board) };
        let material_entry = { material.probe(&board) };
        let mut trace = Trace::new();
        let mut total = {
            let mut eval =
                EvaluationInner::<Trace>::new(board, pawn_entry, material_entry, &mut trace);
            eval.value()
        };
        if board.turn() == Player::Black {
            total = -total;
        }
        print!("{}", trace.t);
        if trace.t.used {
            println!(
                "Total evaluation: {:6.3}  (white side)",
                total as f64 / PAWN_EG as f64
            );
        } else {
            println!(
                "Total evaluation: {:6.3}  (white side) (lazy)",
                total as f64 / PAWN_EG as f64
            );
        }
    }
}

struct EvaluationInner<'a, 'b, T: 'b + Tracing> {
    board: &'a Board,
    pawn_entry: &'a mut PawnEntry,
    material_entry: &'a mut MaterialEntry,
    trace: &'b mut T,
    king_ring: [BitBoard; PLAYER_CNT],
    mobility_area: [BitBoard; PLAYER_CNT],
    mobility: [Score; PLAYER_CNT],
    attacked_by: [[BitBoard; PIECE_TYPE_CNT]; PLAYER_CNT],
    attacked_by2: [BitBoard; PLAYER_CNT],
    king_attackers_count: [u8; PLAYER_CNT],
    king_attackers_weight: [i32; PLAYER_CNT],
    king_adjacent_zone_attacks_count: [i32; PLAYER_CNT],
}

impl<'a, 'b, T: Tracing> EvaluationInner<'a, 'b, T> {
    fn new(
        board: &'a Board,
        pawn_entry: &'a mut PawnEntry,
        material_entry: &'a mut MaterialEntry,
        trace: &'b mut T,
    ) -> Self {
        EvaluationInner {
            board,
            pawn_entry,
            material_entry,
            trace,
            king_ring: [BitBoard(0); PLAYER_CNT],
            mobility_area: [BitBoard(0); PLAYER_CNT],
            mobility: [Score(0, 0); PLAYER_CNT],
            attacked_by: [[BitBoard(0); PIECE_TYPE_CNT]; PLAYER_CNT],
            attacked_by2: [BitBoard(0); PLAYER_CNT],
            king_attackers_count: [0; PLAYER_CNT],
            king_attackers_weight: [0; PLAYER_CNT],
            king_adjacent_zone_attacks_count: [0; PLAYER_CNT],
        }
    }

    fn white_value(&mut self) -> Value {
        let mut score = self.pawn_entry.pawns_score(Player::White)
            - self.pawn_entry.pawns_score(Player::Black)
            + self.material_entry.score()
            + self.board.psq();

        let mut v: i32 = (score.0 + score.1) / 2;
        if v.abs() > LAZY_THRESHOLD {
            if let Some(trace) = self.trace.trace() {
                trace.used = false;
            }
            return v;
        }

        self.initialize::<WhiteType>();
        self.initialize::<BlackType>();

        score += self.evaluate_pieces::<WhiteType, KnightType>()
            - self.evaluate_pieces::<BlackType, KnightType>();
        score += self.evaluate_pieces::<WhiteType, BishopType>()
            - self.evaluate_pieces::<BlackType, BishopType>();
        score += self.evaluate_pieces::<WhiteType, RookType>()
            - self.evaluate_pieces::<BlackType, RookType>();
        score += self.evaluate_pieces::<WhiteType, QueenType>()
            - self.evaluate_pieces::<BlackType, QueenType>();

        score += self.mobility[Player::White as usize] - self.mobility[Player::Black as usize];

        score += self.evaluate_king::<WhiteType>() - self.evaluate_king::<BlackType>();
        score += self.evaluate_threats::<WhiteType>() - self.evaluate_threats::<BlackType>();
        score +=
            self.evaluate_passed_pawns::<WhiteType>() - self.evaluate_passed_pawns::<BlackType>();
        score += self.evaluate_space::<WhiteType>() - self.evaluate_space::<BlackType>();

        score += self.evaluate_initiative(score.eg());

        let phase = self.material_entry.phase as i32;
        let sf = self.scale_factor(score.eg());

        v = score.mg() * phase
            + score.eg() * (PHASE_MID_GAME as i32 - phase) * sf as i32 / SCALE_FACTOR_NORMAL as i32;

        v /= PHASE_MID_GAME as i32;

        if let Some(trace) = self.trace.trace() {
            trace.add_one(EvalPasses::Material, self.board.psq());
            trace.add_one(EvalPasses::Imbalance, self.material_entry.score());
            trace.add_both(
                EvalPasses::Pawn,
                self.pawn_entry.pawns_score(Player::White),
                self.pawn_entry.pawns_score(Player::Black),
            );
            trace.add_both(
                EvalPasses::Mobility,
                self.mobility[Player::White as usize],
                self.mobility[Player::Black as usize],
            );
            trace.add_one(EvalPasses::Total, score);
        }

        #[cfg(debug_assertions)]
        {
            if self.trace.trace().is_none() && (v <= -32001 || v >= 32001) {
                println!("\n Unusable score!");
                println!("fen: {} ", self.board.fen());
                Evaluation::trace(&self.board);
                println!();
                panic!();
            }
        }
        v
    }

    fn value(&mut self) -> Value {
        if self.board.turn() == Player::White {
            self.white_value()
        } else {
            -self.white_value()
        }
    }

    fn initialize<P: PlayerTrait>(&mut self) {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let ksq_us: SQ = self.board.king_sq(us);
        let low_ranks: BitBoard = if us == Player::White {
            BitBoard::RANK_2 | BitBoard::RANK_3
        } else {
            BitBoard::RANK_6 | BitBoard::RANK_7
        };

        // Find our pawns on the first two ranks, and those which are blocked
        let mut b: BitBoard = self.board.piece_bb(us, PieceType::P)
            & P::shift_down(self.board.occupied() | low_ranks);

        // Squares occupied by those pawns, by our king, or controlled by enemy pawns
        // are excluded from the mobility area.
        self.mobility_area[us as usize] =
            !(b | self.board.piece_bb(us, PieceType::K) | self.pawn_entry.pawn_attacks(them));

        b = king_moves(ksq_us);
        self.attacked_by[us as usize][PieceType::K as usize] = b;
        self.attacked_by[us as usize][PieceType::P as usize] = self.pawn_entry.pawn_attacks(us);

        self.attacked_by[us as usize][PieceType::All as usize] =
            b | self.attacked_by[us as usize][PieceType::P as usize];
        self.attacked_by2[us as usize] = b & self.attacked_by[us as usize][PieceType::P as usize];

        if self.board.non_pawn_material(them) >= ROOK_MG + KNIGHT_MG {
            self.king_ring[us as usize] = b;
            if us.relative_rank_of_sq(ksq_us) == Rank::R1 {
                self.king_ring[us as usize] |= P::shift_up(b);
            }

            if ksq_us.file() == File::H {
                self.king_ring[us as usize] |= P::shift_left(b);
            } else if ksq_us.file() == File::A {
                self.king_ring[us as usize] |= P::shift_right(b);
            }

            self.king_attackers_count[them as usize] =
                (b & self.pawn_entry.pawn_attacks(them)).count_bits();
            self.king_adjacent_zone_attacks_count[them as usize] = 0;
            self.king_attackers_weight[them as usize] = 0;
        } else {
            self.king_ring[us as usize] = BitBoard(0);
            self.king_attackers_count[them as usize] = 0;
        }
    }

    fn evaluate_pieces<P: PlayerTrait, S: PieceTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let piece: PieceType = S::piece_type();
        let ksq_us: SQ = self.board.king_sq(us);
        let outpost_ranks: BitBoard = if us == Player::White {
            BitBoard::RANK_4 | BitBoard::RANK_5 | BitBoard::RANK_6
        } else {
            BitBoard::RANK_5 | BitBoard::RANK_4 | BitBoard::RANK_3
        };

        let mut ps1 = self.board.piece_bb(P::player(), piece);
        let mut score = Score::ZERO;
        let mut b: BitBoard;
        let mut bb: BitBoard;

        while let Some((s, bits)) = ps1.pop_some_lsb_and_bit() {
            b = if piece == PieceType::B {
                let o: BitBoard =
                    self.board.occupied() ^ self.board.piece_bb_both_players(PieceType::Q);
                bishop_moves(o, s)
            } else if piece == PieceType::R {
                let o: BitBoard = self.board.occupied()
                    ^ self.board.piece_bb_both_players(PieceType::Q)
                    ^ self.board.piece_bb(us, PieceType::R);
                rook_moves(o, s)
            } else {
                self.board.attacks_from(piece, s, us)
            };

            if (self.board.all_pinned_pieces(us) & bits).is_not_empty() {
                b &= line_bb(ksq_us, s);
            }

            self.attacked_by2[us as usize] |=
                b & self.attacked_by[us as usize][PieceType::All as usize];
            self.attacked_by[us as usize][piece as usize] |= b;
            self.attacked_by[us as usize][PieceType::All as usize] |= b;

            if (b & self.king_ring[them as usize]).is_not_empty() {
                self.king_attackers_count[us as usize] += 1;
                self.king_attackers_weight[us as usize] += KING_ATTACKS_WEIGHT[piece as usize];
                self.king_adjacent_zone_attacks_count[us as usize] +=
                    (b & self.attacked_by[them as usize][PieceType::K as usize]).count_bits()
                        as i32;
            }

            let mob: u8 = (b & self.mobility_area[us as usize]).count_bits();

            self.mobility[us as usize] += MOBILITY_BONUS[piece as usize][mob as usize];

            // Penalty if the piece is far from the king
            score -= KING_PROTECTOR[piece as usize] * distance_of_sqs(s, ksq_us);

            if piece == PieceType::B || piece == PieceType::N {
                bb = outpost_ranks & !self.pawn_entry.pawn_attacks_span(them);
                if (bb & bits).is_not_empty() {
                    score += OUTPOST[(piece == PieceType::B) as usize][(self.attacked_by
                        [us as usize][PieceType::P as usize]
                        & bits)
                        .is_not_empty()
                        as usize]
                        * 2;
                } else {
                    bb &= b & !self.board.get_occupied_player(us);
                    if bb.is_not_empty() {
                        score += OUTPOST[(piece == PieceType::B) as usize][(self.attacked_by
                            [us as usize][PieceType::P as usize]
                            & bb)
                            .is_not_empty()
                            as usize];
                    }
                }

                // bonus when behind a pawn
                if us.relative_rank_of_sq(s) < Rank::R5
                    && (self.board.piece_bb_both_players(PieceType::P) & P::up(s).to_bb())
                        .is_not_empty()
                {
                    score += MINOR_BEHIND_PAWN;
                }

                if piece == PieceType::B {
                    // Penalty according to number of pawns on the same color square as the bishop
                    score -= BISHOP_PAWNS * self.pawn_entry.pawns_on_same_color_squares(us, s);

                    // Bonus for bishop on a long diagonal which can "see" both center squares
                    if (CENTER & (bishop_moves(self.board.piece_bb_both_players(PieceType::P), s))
                        | bits)
                        .more_than_one()
                    {
                        score += LONG_RANGED_BISHOP;
                    }
                }
            } else if piece == PieceType::R {
                // Bonus for aligning with enemy pawns on the same rank/file
                if us.relative_rank_of_sq(s) >= Rank::R5 {
                    score += ROOK_ON_PAWN
                        * (self.board.piece_bb(them, PieceType::P) & rook_moves(BitBoard(0), s))
                            .count_bits();
                }

                // Bonus when on an open or semi-open file
                if self.pawn_entry.semiopen_file(us, s.file()) {
                    score += ROOK_ON_FILE[self.pawn_entry.semiopen_file(them, s.file()) as usize];
                } else if mob <= 3 {
                    // Penalty when trapped by the king, even more if the king cannot castle
                    let k_file = ksq_us.file();
                    if (k_file < File::E) == (s.file() < k_file) {
                        score -= (TRAPPED_ROOK - Score(mob as i32 * 22, 0))
                            * (1 + (self.board.player_can_castle(us).bits() == 0) as u8);
                    }
                }
            } else if piece == PieceType::Q {
                let mut pinners: BitBoard = unsafe { mem::MaybeUninit::uninit().assume_init() };
                let pieces = self.board.piece_two_bb(PieceType::B, PieceType::R, them);
                self.board.slider_blockers(pieces, s, &mut pinners);
                if pinners.is_not_empty() {
                    score -= WEAK_QUEEN
                }
            }
        }

        if let Some(trace) = self.trace.trace() {
            trace.add_piece(piece, us, score);
        }
        score
    }

    fn evaluate_king<P: PlayerTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let ksq_us: SQ = self.board.king_sq(us);

        let camp: BitBoard = if us == Player::White {
            BitBoard::ALL ^ BitBoard::RANK_6 ^ BitBoard::RANK_7 ^ BitBoard::RANK_8
        } else {
            BitBoard::ALL ^ BitBoard::RANK_1 ^ BitBoard::RANK_2 ^ BitBoard::RANK_3
        };

        let weak: BitBoard;
        let b: BitBoard;
        let mut b1: BitBoard;
        let mut b2: BitBoard;
        let mut safe_b: BitBoard;
        let mut unsafe_checks: BitBoard;
        let pinned: BitBoard;

        // King shelter and enemy pawns storm
        let mut score = self.pawn_entry.king_safety::<P>(self.board, ksq_us);

        // Main king safety evaluation
        if self.king_attackers_count[them as usize] as i32
            > (1 - self.board.count_piece(them, PieceType::Q) as i32)
        {
            let mut king_danger: i32 = 0;
            unsafe_checks = BitBoard(0);

            // Attacked squares defended at most once by our queen or king
            weak = self.attacked_by[them as usize][PieceType::All as usize]
                & !self.attacked_by2[us as usize]
                & (self.attacked_by[us as usize][PieceType::K as usize]
                    | self.attacked_by[us as usize][PieceType::Q as usize]
                    | !self.attacked_by[us as usize][PieceType::All as usize]);

            // Analyse the safe enemy's checks which are possible on next move
            safe_b = !self.board.get_occupied_player(them);
            safe_b &= !self.attacked_by[us as usize][PieceType::All as usize]
                | (weak & self.attacked_by2[them as usize]);

            let us_queen: BitBoard = self.board.piece_bb(us, PieceType::Q);
            b1 = rook_moves(self.board.occupied() ^ us_queen, ksq_us);
            b2 = bishop_moves(self.board.occupied() ^ us_queen, ksq_us);

            // Enemy queen safe checks
            if ((b1 | b2)
                & self.attacked_by[them as usize][PieceType::Q as usize]
                & safe_b
                & !self.attacked_by[us as usize][PieceType::Q as usize])
                .is_not_empty()
            {
                king_danger += QUEEN_SAFE_CHECK;
            }

            b1 &= self.attacked_by[them as usize][PieceType::R as usize];
            b2 &= self.attacked_by[them as usize][PieceType::B as usize];

            // Enemy rook checks
            if (b1 & safe_b).is_not_empty() {
                king_danger += ROOK_SAFE_CHECK;
            } else {
                unsafe_checks |= b1;
            }

            // Enemy bishops checks
            if (b2 & safe_b).is_not_empty() {
                king_danger += BISHOP_SAFE_CHECK;
            } else {
                unsafe_checks |= b2;
            }

            // Enemy knights checks
            b = knight_moves(ksq_us) & self.attacked_by[them as usize][PieceType::N as usize];
            if (b & safe_b).is_not_empty() {
                king_danger += KNIGHT_SAFE_CHECK;
            } else {
                unsafe_checks |= b;
            }

            // Unsafe or occupied checking squares will also be considered, as long as
            // the square is in the attacker's mobility area.
            unsafe_checks &= self.mobility_area[them as usize];

            pinned = self.board.all_pinned_pieces(us) & self.board.get_occupied_player(us);

            king_danger += self.king_attackers_count[them as usize] as i32
                * self.king_attackers_weight[them as usize];
            king_danger += 102 * self.king_adjacent_zone_attacks_count[them as usize];
            king_danger += 191 * (self.king_ring[us as usize] & weak).count_bits() as i32;
            king_danger += 848 * (pinned | unsafe_checks).count_bits() as i32;
            king_danger -= 848 * (self.board.count_piece(them, PieceType::Q) != 0) as i32;
            king_danger -= 9 * score.mg() as i32 / 8;
            king_danger += 40;

            if king_danger > 0 {
                let mobility_danger =
                    (self.mobility[them as usize] - self.mobility[us as usize]).mg();
                king_danger = (king_danger + mobility_danger).max(0);
                let mg: Value = (king_danger * king_danger) / 4096;
                let eg: Value = king_danger / 16;
                score -= Score(mg, eg);
            }
        }

        let kf: File = ksq_us.file();

        // Penalty when our king is on a pawnless flank
        if (self.board.piece_bb_both_players(PieceType::P) & KING_FLANK[kf as usize]).is_empty() {
            score -= PAWNLESS_FLANK;
        }

        // Find the squares that opponent attacks in our king flank, and the squares
        // which are attacked twice in that flank but not defended by our pawns.
        b1 = self.attacked_by[them as usize][PieceType::All as usize]
            & SQ(kf as u8).file_bb()
            & camp;
        b2 = b1
            & self.attacked_by2[them as usize]
            & !self.attacked_by[us as usize][PieceType::P as usize];

        score -= CLOSE_ENEMIES * (b1.count_bits() + b2.count_bits());

        if let Some(trace) = self.trace.trace() {
            trace.add_piece(PieceType::K, us, score);
        }

        score
    }

    fn evaluate_threats<P: PlayerTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let t_rank_3_bb = if us == Player::White {
            BitBoard::RANK_3
        } else {
            BitBoard::RANK_6
        };

        let mut b: BitBoard;
        let weak: BitBoard;
        let defended: BitBoard;
        let non_pawn_enemies: BitBoard;
        let strongly_protected: BitBoard;
        let mut safe_threats: BitBoard;
        let mut score: Score = Score::ZERO;

        // Non-pawn enemies attacked by a pawn
        non_pawn_enemies =
            self.board.piece_bb(them, PieceType::P) ^ self.board.get_occupied_player(them);
        weak = non_pawn_enemies & self.attacked_by[us as usize][PieceType::P as usize];

        if weak.is_not_empty() {
            b = self.board.piece_bb(us, PieceType::P)
                & (!self.attacked_by[them as usize][PieceType::All as usize]
                    | self.attacked_by[them as usize][PieceType::P as usize]);
            safe_threats = (P::shift_up_right(b) | P::shift_up_left(b)) & weak;
            score += THREAT_BY_SAFE_PAWN * safe_threats.count_bits();
        }

        // Squares strongly protected by the opponent, either because they attack the
        // square with a pawn, or because they attack the square twice and we don't.
        strongly_protected = self.attacked_by[them as usize][PieceType::P as usize]
            | (self.attacked_by2[them as usize] & !self.attacked_by2[us as usize]);

        // Non-pawn enemies, strongly protected
        defended = (self.board.get_occupied_player(them) ^ self.board.piece_bb(them, PieceType::P))
            & strongly_protected;

        // Add a bonus according to the kind of attacking pieces
        if (defended | weak).is_not_empty() {
            b = (defended | weak)
                & (self.attacked_by[us as usize][PieceType::N as usize]
                    | self.attacked_by[us as usize][PieceType::B as usize]);

            while let Some(s) = b.pop_some_lsb() {
                let piece = self.board.piece_at_sq(s).type_of();
                score += THREAT_BY_MINOR[piece as usize];
                if piece != PieceType::P {
                    score += THREAT_BY_RANK * them.relative_rank_of_sq(s) as u8;
                }
            }

            b = (self.board.piece_bb(them, PieceType::Q) | weak)
                & self.attacked_by[us as usize][PieceType::R as usize];
            while let Some(s) = b.pop_some_lsb() {
                let piece = self.board.piece_at_sq(s).type_of();
                score += THREAT_BY_ROOK[piece as usize];
                if piece != PieceType::P {
                    score += THREAT_BY_RANK * them.relative_rank_of_sq(s) as u8;
                }
            }

            score += HANGING
                * (weak & !self.attacked_by[them as usize][PieceType::All as usize]).count_bits();

            b = weak & self.attacked_by[us as usize][PieceType::K as usize];
            if b.is_not_empty() {
                score += THREAT_BY_KING[b.more_than_one() as usize];
            }
        }

        // Bonus for opponent unopposed weak pawns
        if self
            .board
            .piece_two_bb(PieceType::R, PieceType::Q, us)
            .is_not_empty()
        {
            score += WEAK_UNOPOSSED_PAWN * self.pawn_entry.weak_unopposed(them);
        }

        // Find squares where our pawns can push on the next move
        b = P::shift_up(self.board.piece_bb(us, PieceType::P)) & !self.board.occupied();
        b |= P::shift_up(b & t_rank_3_bb) & !self.board.occupied();

        // Find squares where our pawns can push on the next move
        b &= !self.attacked_by[them as usize][PieceType::P as usize]
            & (self.attacked_by[us as usize][PieceType::All as usize]
                | !self.attacked_by[them as usize][PieceType::All as usize]);

        // Add a bonus for each new pawn threats from those squares
        b = (P::shift_up_left(b) | P::shift_up_right(b))
            & self.board.get_occupied_player(them)
            & !self.attacked_by[us as usize][PieceType::P as usize];

        score += THREAT_BY_PAWN_PUSH * b.count_bits();

        if self.board.count_piece(them, PieceType::Q) == 1 {
            let mut opp_quens = self.board.piece_bb(them, PieceType::Q);
            while let Some(s) = opp_quens.pop_some_lsb() {
                safe_threats = self.mobility_area[us as usize] & !strongly_protected;

                let occ_all = self.board.occupied();
                b = (self.attacked_by[us as usize][PieceType::B as usize]
                    & bishop_moves(occ_all, s))
                    | (self.attacked_by[us as usize][PieceType::R as usize]
                        & rook_moves(occ_all, s));

                score += SLIDER_ON_QUEEN
                    * (b * safe_threats & self.attacked_by2[us as usize]).count_bits();
            }
        }

        // Connectivity: ensure that knights, bishops, rooks, and queens are protected
        b = (self.board.get_occupied_player(us)
            ^ self.board.piece_two_bb(PieceType::P, PieceType::K, us))
            & self.attacked_by[us as usize][PieceType::All as usize];

        score += CONNECTIVITY * b.count_bits();

        if let Some(trace) = self.trace.trace() {
            trace.add(EvalPasses::Threat, us, score);
        }

        score
    }

    fn evaluate_passed_pawns<P: PlayerTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();

        let us_ksq = self.board.king_sq(us);
        let them_ksq = self.board.king_sq(them);

        let mut b: BitBoard;
        let mut bb: BitBoard;
        let mut squares_to_queen: BitBoard;
        let mut defended_squares: BitBoard;
        let mut unsafe_squares: BitBoard;

        let mut score: Score = Score::ZERO;

        b = self.pawn_entry.passed_pawns(us);

        while let Some((s, bits)) = b.pop_some_lsb_and_bit() {
            bb = forward_file_bb(us, s)
                & (self.attacked_by[them as usize][PieceType::All as usize]
                    | self.board.get_occupied_player(them));
            score -= HINDER_PASSED_PAWN * bb.count_bits();

            let r: Rank = us.relative_rank_of_sq(s);
            let w = PASSED_DANGER[r as usize];

            let mut mbonus: Value = PASSED[0][r as usize];
            let mut ebonus: Value = PASSED[1][r as usize];

            if w != 0 {
                let block_sq: SQ = P::up(s);

                ebonus += (king_proximity(block_sq, them_ksq) * 5
                    - king_proximity(block_sq, us_ksq) as i32 * 2)
                    * w;

                if r != Rank::R7 {
                    ebonus -= self.king_distance(us, P::up(block_sq)) as i32 * w;
                }

                if self.board.piece_at_sq(block_sq) == Piece::None {
                    // If there is a rook or queen attacking/defending the pawn from behind,
                    // consider all the squaresToQueen. Otherwise consider only the squares
                    // in the pawn's path attacked or occupied by the enemy.
                    defended_squares = forward_file_bb(us, s);
                    unsafe_squares = defended_squares;
                    squares_to_queen = defended_squares;

                    bb = self
                        .board
                        .piece_two_bb_both_players(PieceType::R, PieceType::Q)
                        & rook_moves(self.board.occupied(), s)
                        & forward_file_bb(them, s);

                    if (self.board.get_occupied_player(us) & bb).is_empty() {
                        defended_squares &= self.attacked_by[us as usize][PieceType::All as usize];
                    }

                    if (self.board.get_occupied_player(them) & bb).is_empty() {
                        unsafe_squares &= self.attacked_by[them as usize][PieceType::All as usize]
                            | self.board.get_occupied_player(them);
                    }

                    // If there aren't any enemy attacks, assign a big bonus. Otherwise
                    // assign a smaller bonus if the block square isn't attacked.
                    let mut k: i32 = if unsafe_squares.is_empty() {
                        20
                    } else if (unsafe_squares & block_sq.to_bb()).is_empty() {
                        9
                    } else {
                        0
                    };

                    // If the path to the queen is fully defended, assign a big bonus.
                    // Otherwise assign a smaller bonus if the block square is defended.
                    if defended_squares == squares_to_queen {
                        k += 6;
                    } else if (defended_squares & block_sq.to_bb()).is_not_empty() {
                        k += 4;
                    }

                    mbonus += k * w;
                    ebonus += k * w;
                } else if (self.board.get_occupied_player(us) & bits).is_not_empty() {
                    mbonus += w + r as i32 * 2;
                    ebonus += w + r as i32 * 2;
                }
            }

            // Scale down bonus for candidate passers which need more than one
            // pawn push to become passed or have a pawn in front of them.
            if !self.board.pawn_passed(us, P::up(s))
                || (self.board.piece_bb_both_players(PieceType::P) & forward_file_bb(us, s))
                    .is_not_empty()
            {
                mbonus /= 2;
                ebonus /= 2;
            }

            score += Score(mbonus, ebonus) + PASSED_FILE[s.file() as usize];
        }

        if let Some(trace) = self.trace.trace() {
            trace.add(EvalPasses::Passed, us, score);
        }

        score
    }
    // king_distance() returns an estimate of the distance that the king
    // of the given color has to run to reach square s.
    fn king_distance(&self, player: Player, sq: SQ) -> u8 {
        distance_of_sqs(self.board.king_sq(player), sq).min(5)
    }

    // evaluate_space() computes the space evaluation for a given side. The
    // space evaluation is a simple bonus based on the number of safe squares
    // available for minor pieces on the central four files on ranks 2--4. Safe
    // squares one, two or three squares behind a friendly pawn are counted
    // twice. Finally, the space bonus is multiplied by a weight. The aim is to
    // improve play on game opening.
    fn evaluate_space<P: PlayerTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();

        let space_mask = if us == Player::White {
            CENTER_FILES & (BitBoard::RANK_2 | BitBoard::RANK_3 | BitBoard::RANK_4)
        } else {
            CENTER_FILES & (BitBoard::RANK_7 | BitBoard::RANK_6 | BitBoard::RANK_5)
        };

        if self.board.non_pawn_material(Player::White) + self.board.non_pawn_material(Player::Black)
            < SPACE_THRESHOLD
        {
            return Score::ZERO;
        }

        let safe: BitBoard = space_mask
            & !self.board.piece_bb(us, PieceType::P)
            & !self.attacked_by[them as usize][PieceType::P as usize]
            & (self.attacked_by[us as usize][PieceType::All as usize]
                | !self.attacked_by[them as usize][PieceType::All as usize]);

        let mut behind: BitBoard = self.board.piece_bb(us, PieceType::P);
        behind |= P::shift_down(behind);
        behind |= P::shift_down(P::shift_down(behind));

        let bonus: i32 = (safe).count_bits() as i32 + (behind & safe).count_bits() as i32;
        let weight: i32 =
            self.board.count_pieces_player(us) as i32 - 2 * self.pawn_entry.open_files() as i32;

        let score = Score(bonus * weight * weight / 16, 0);

        if let Some(trace) = self.trace.trace() {
            trace.add(EvalPasses::Space, us, score);
        }
        score
    }

    // evaluate_initiative() computes the initiative correction value for the
    // position, i.e., second order bonus/malus based on the known attacking/defending
    // status of the players.
    fn evaluate_initiative(&mut self, eg: Value) -> Score {
        let w_ksq = self.board.king_sq(Player::White);
        let b_ksq = self.board.king_sq(Player::Black);
        let king_distance: i32 =
            w_ksq.file().distance(b_ksq.file()) as i32 - w_ksq.rank().distance(b_ksq.rank()) as i32;
        let both_flanks: bool = (self.board.piece_bb_both_players(PieceType::P) & QUEEN_SIDE)
            .is_not_empty()
            && (self.board.piece_bb_both_players(PieceType::P) & KING_SIDE).is_not_empty();
        let pawn_count: u8 = self.board.count_piece(Player::White, PieceType::P)
            + self.board.count_piece(Player::Black, PieceType::P);

        let npm = self.board.non_pawn_material(Player::White)
            + self.board.non_pawn_material(Player::Black);

        // Compute the initiative bonus for the attacking side
        let complexity: i32 = 8 * self.pawn_entry.asymmetry() as i32
            + 8 * king_distance
            + 12 * pawn_count as i32
            + 16 * both_flanks as i32
            + 48 * (npm == 0) as i32
            - 136;

        // Now apply the bonus: note that we find the attacking side by extracting
        // the sign of the endgame value, and that we carefully cap the bonus so
        // that the endgame score will never change sign after the bonus.
        let v: i32 = ((eg > 0) as i32 - (eg < 0) as i32) * complexity.max(-eg.abs());

        if let Some(trace) = self.trace.trace() {
            trace.add_one(EvalPasses::Initiative, Score(0, v));
        }

        Score(0, v)
    }

    fn scale_factor(&self, eg: i32) -> u8 {
        let strong_side = if eg > 0 { Player::White } else { Player::Black };

        let mut sf = self.material_entry.scale_factor(strong_side);

        // If we don't already have an unusual scale factor, check for certain
        // types of endgames, and use a lower scale for those.
        if sf == SCALE_FACTOR_NORMAL || sf == SCALE_FACTOR_ONEPAWN {
            if self.board.opposite_bishops() {
                // Endgame with opposite-colored bishops and no other pieces is almost a draw
                if self.board.non_pawn_material(Player::White) == BISHOP_MG
                    && self.board.non_pawn_material(Player::Black) == BISHOP_MG
                {
                    sf = 31;
                } else {
                    sf = 46;
                }
            }
        }
        // Endings where weaker side can place his king in front of the enemy's
        // pawns are drawish.
        else if eg.abs() < BISHOP_EG
            && self.board.count_piece(strong_side, PieceType::P) <= 2
            && !self
                .board
                .pawn_passed(!strong_side, self.board.king_sq(!strong_side))
        {
            sf = 37 + 7 * self.board.count_piece(strong_side, PieceType::P);
        }
        sf
    }
}

fn king_proximity(king_sq: SQ, sq: SQ) -> i32 {
    distance_of_sqs(king_sq, sq).min(5) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    //    #[test]
    //    fn eval_stuff() {
    //        init_statics();
    //        let board = Board::start_pos();
    //        let mut score = Score::ZERO;
    //        let mut bb = board.get_occupied();
    //        while let Some(sq) = bb.pop_some_lsb() {
    //            let player = board.player_at_sq(sq).unwrap();
    //            let piece = board.piece_at_sq(sq).unwrap();
    //            let ps = psq(piece, player, sq);
    //            println!("Player: {}, Piece: {}, sq: {}, score mg: {}, eg: {}",player,piece,sq.to_string(),ps.mg(),ps.eg());
    //            score+=ps;
    //        }
    //        println!("mg: {}, eg: {}",score.mg(),score.mg());
    //
    //    }

    #[test]
    fn bad_board() {
        let board =
            Board::from_fen("rnbqk1nr/pppp1ppp/8/4p3/1b2P3/P7/1PPP1PPP/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        Evaluation::trace(&board);
    }

    #[test]
    fn trace_eval() {
        let mut board = Board::start_pos();
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("e2e3");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("e7e5");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("d1g4");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("d7d6");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("g4c8");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("d8c8");
        board.pretty_print();
        Evaluation::trace(&board);
        println!("\n>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n");
        board.apply_uci_move("a2a4");
        board.pretty_print();
        Evaluation::trace(&board);
    }
}

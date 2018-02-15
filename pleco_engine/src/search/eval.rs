
use std::mem;
#[allow(unused_imports)]
use pleco::{Board,BitBoard,SQ,Rank,File,Player,PieceType};
#[allow(unused_imports)]
use pleco::core::mono_traits::*;
use pleco::core::score::*;
use pleco::core::masks::*;

use pleco::helper::prelude::*;

use tables::pawn_table::{PawnEntry, PawnTable};
use tables::material::*;

const CENTER: BitBoard = BitBoard((FILE_D | FILE_E) & (RANK_4 | RANK_5));
const QUEEN_SIDE: BitBoard = BitBoard(FILE_A | FILE_B | FILE_C | FILE_D);
const CENTER_FILES: BitBoard = BitBoard(FILE_C | FILE_D | FILE_E | FILE_F);
const KING_SIDE: BitBoard = BitBoard(FILE_E | FILE_F | FILE_G | FILE_H);

const KING_FLANK: [BitBoard; FILE_CNT] = [QUEEN_SIDE, QUEEN_SIDE, QUEEN_SIDE, CENTER_FILES, CENTER_FILES, KING_SIDE, KING_SIDE, KING_SIDE];

const KING_ATTACKS_WEIGHT: [i32; PIECE_TYPE_CNT] = [0, 78, 56, 45, 11, 0];


const MOBILITY_BONUS: [[Score; 32]; PIECE_TYPE_CNT] = [
[   Score::ZERO; 32], // Pawns
[   Score(-75,-76), Score(-57,-54), Score( -9,-28), Score( -2,-10), Score(  6,  5), Score( 14, 12), // Knights
    Score( 22, 26), Score( 29, 29), Score( 36, 29), Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,          Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO
],
[   Score(-48,-59), Score(-20,-23), Score( 16, -3), Score( 26, 13), Score( 38, 24), Score( 51, 42), // Bishops
    Score( 55, 54), Score( 63, 57), Score( 63, 65), Score( 68, 73), Score( 81, 78), Score( 81, 86),
    Score( 91, 88), Score( 98, 97), Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO
],
[   Score(-58,-76), Score(-27,-18), Score(-15, 28), Score(-10, 55), Score( -5, 69), Score( -2, 82), // Rooks
    Score(  9,112), Score( 16,118), Score( 30,132), Score( 29,142), Score( 32,155), Score( 38,165),
    Score( 46,166), Score( 48,169), Score( 58,171), Score::ZERO,          Score::ZERO,          Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO,           Score::ZERO,          Score::ZERO,          Score::ZERO,           Score::ZERO,
    Score::ZERO,          Score::ZERO
],
[   Score(-39,-36), Score(-21,-15), Score(  3,  8), Score(  3, 18), Score( 14, 34), Score( 22, 54), // Queens
    Score( 28, 61), Score( 41, 73), Score( 43, 79), Score( 48, 92), Score( 56, 94), Score( 60,104),
    Score( 60,113), Score( 66,120), Score( 67,123), Score( 70,126), Score( 71,133), Score( 73,136),
    Score( 79,140), Score( 88,143), Score( 88,148), Score( 99,166), Score(102,170), Score(102,175),
    Score(106,184), Score(109,191), Score(113,206), Score(116,212), Score::ZERO,          Score::ZERO,
    Score::ZERO,          Score::ZERO
],
[Score::ZERO; 32]
];

const KING_PROTECTOR: [Score; PIECE_TYPE_CNT] = [Score(0,0), Score(-3, -5), Score(-4, -3), Score(-3, 0), Score(-1, 1), Score(0,0) ];

// Outpost[knight/bishop][supported by pawn] contains bonuses for minor
// pieces if they can reach an outpost square, bigger if that square is
// supported by a pawn. If the minor piece occupies an outpost square
// then score is doubled.
const OUTPOST: [[Score; 2]; 2] = [
[ Score(22, 6), Score(36,12) ], // Knight
[ Score( 9, 2), Score(15, 5) ]  // Bishop
];


const ROOK_ON_FILE: [Score; 2] = [Score(20, 7), Score(45, 20)];

// ThreatByMinor/ByRook[attacked PieceType] contains bonuses according to
// which piece type attacks which one. Attacks on lesser pieces which are
// pawn-defended are not considered.
const THREAT_BY_MINOR: [Score; PIECE_TYPE_CNT] = [
    Score(0, 0), Score(0, 33), Score(45, 43), Score(46, 47), Score(72, 107), Score(48, 118)
];

const THREAT_BY_ROOK: [Score; PIECE_TYPE_CNT] = [
    Score(0, 0), Score(0, 25), Score(40, 62), Score(40, 59), Score(0, 34), Score(35, 48)
];

// ThreatByKing[on one/on many] contains bonuses for king attacks on
// pawns or pieces which are not pawn-defended.
const THREAT_BY_KING: [Score; 2] = [Score(3, 62), Score(9, 138) ];

// Passed[mg/eg][Rank] contains midgame and endgame bonuses for passed pawns.
// We don't use a Score because we process the two components independently.
const PASSED: [[Value; RANK_CNT]; 2] = [
    [ 0, 5,  5, 31, 73, 166, 252, 0 ],
    [ 0, 7, 14, 38, 73, 166, 252, 0 ]
];

// PassedFile[File] contains a bonus according to the file of a passed pawn
const PASSED_FILE: [Score; FILE_CNT] = [
Score(  9, 10), Score( 2, 10), Score( 1, -8), Score(-20,-12),
Score(-20,-12), Score( 1, -8), Score( 2, 10), Score(  9, 10)
];

const RANK_FACTOR: [i32; RANK_CNT] = [ 0, 0, 0, 2, 6, 11, 16, 0];

// Assorted bonuses and penalties used by evaluation
const MINOR_BEHIND_PAWN: Score = Score( 16,  0);
const BISHOP_PAWNS          : Score = Score(  8, 12);
const LONG_RANGED_BISHOP     : Score = Score( 22,  0);
const ROOK_ON_PAWN           : Score = Score(  8, 24);
const TRAPPED_ROOK          : Score = Score( 92,  0);
const WEAK_QUEEN            : Score = Score( 50, 10);
const CLOSE_ENEMIES         : Score = Score(  7,  0);
const PAWNLESS_FLANK        : Score = Score( 20, 80);
const THREAT_BY_SAFE_PAWN     : Score = Score(192,175);
const THREAT_BY_RANK         : Score = Score( 16,  3);
const HANGING              : Score = Score( 48, 27);
const WEAK_UNOPOSSED_PAWN    : Score = Score(  5, 25);
const THREAT_BY_PAWN_PUSH     : Score = Score( 38, 22);
const THREAT_BY_ATTACK_ON_QUEEN : Score = Score( 38, 22);
const HINDER_PASSED_PAWN     : Score = Score(  7,  0);
const TRAPPED_BISHOP_A1H1    : Score = Score( 50, 50);


// Penalties for enemy's safe checks
const QUEEN_SAFE_CHECK: i32  = 780;
const ROOK_SAFE_CHECK: i32   = 880;
const BISHOP_SAFE_CHECK: i32 = 435;
const KNIGHT_SAFE_CHECK: i32 = 790;


const LAZY_THRESHOLD: Value = 1500;
const SPACE_THRESHOLD: Value = 12222;

pub struct Evaluation<'a> {
    board: &'a Board,
    pawn_entry: &'a mut PawnEntry,
    material_entry: &'a mut MaterialEntry,
    king_ring: [BitBoard; PLAYER_CNT],
    mobility_area: [BitBoard; PLAYER_CNT],
    mobility: [Score; PLAYER_CNT],
    attacked_by: [[BitBoard; PIECE_TYPE_CNT];PLAYER_CNT],
    attacked_by_all: [BitBoard; PLAYER_CNT],
    attacked_by_queen_diagonal: [BitBoard; PLAYER_CNT],
    attacked_by2: [BitBoard; PLAYER_CNT],
    king_attackers_count: [u8; PLAYER_CNT],
    king_attackers_weight: [i32; PLAYER_CNT],
    king_adjacent_zone_attacks_count: [i32; PLAYER_CNT],
}

impl <'a> Evaluation <'a> {
    pub fn evaluate(board: &Board, pawn_table: &mut PawnTable, material: &mut Material) -> Value {
        #[allow(unused_variables)]

        let pawn_entry = { pawn_table.probe(&board) };
        let material_entry = { material.probe(&board) };

        let mut eval = Evaluation {
            board,
            pawn_entry,
            material_entry,
            king_ring: [BitBoard(0); PLAYER_CNT],
            mobility_area: [BitBoard(0); PLAYER_CNT],
            mobility: [Score(0,0); PLAYER_CNT],
            attacked_by: [[BitBoard(0); PIECE_TYPE_CNT];PLAYER_CNT],
            attacked_by_all: [BitBoard(0); PLAYER_CNT],
            attacked_by_queen_diagonal: [BitBoard(0); PLAYER_CNT],
            attacked_by2: [BitBoard(0) ;PLAYER_CNT],
            king_attackers_count: [0; PLAYER_CNT],
            king_attackers_weight: [0; PLAYER_CNT],
            king_adjacent_zone_attacks_count: [0; PLAYER_CNT],
        };

        eval.value()
    }

    fn value(&mut self) -> Value {
        let mut score = self.pawn_entry.pawns_score() + self.material_entry.score();
        let mut v = (score.0 + score.1) / 2;
        if v.abs() > LAZY_THRESHOLD {
            if self.board.turn() == Player::White {return v;}
            else {return -v;}
        }

        self.initialize::<WhiteType>();
        self.initialize::<BlackType>();

        score += self.evaluate_pieces::<WhiteType,KnightType>() - self.evaluate_pieces::<BlackType,KnightType>();
        score += self.evaluate_pieces::<WhiteType,BishopType>() - self.evaluate_pieces::<BlackType,BishopType>();
        score += self.evaluate_pieces::<WhiteType,RookType  >() - self.evaluate_pieces::<BlackType,RookType  >();
        score += self.evaluate_pieces::<WhiteType,QueenType >() - self.evaluate_pieces::<BlackType,QueenType >();

        score += self.mobility[Player::White as usize] - self.mobility[Player::Black as usize];

        score += self.evaluate_king::<WhiteType>() - self.evaluate_king::<BlackType>();

        score += self.evaluate_threats::<WhiteType>() - self.evaluate_threats::<BlackType>();

        score += self.evaluate_passed_pawns::<WhiteType>() - self.evaluate_passed_pawns::<BlackType>();
        let phase = self.material_entry.phase as i32;

        v =   score.mg() * phase
            + score.eg() * (PHASE_MID_GAME as i32 - phase);

        v /= PHASE_MID_GAME as i32;

        if self.board.turn() == Player::White {
            v
        } else {
            -v
        }
    }

    fn initialize<P: PlayerTrait>(&mut self) {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let ksq_us: SQ = self.board.king_sq(us);
        let low_ranks: BitBoard =
            if us == Player::White {BitBoard::RANK_2 | BitBoard::RANK_3} else {BitBoard::RANK_6 | BitBoard::RANK_7};

        // Find our pawns on the first two ranks, and those which are blocked
        let mut b: BitBoard = self.board.piece_bb(us, PieceType::P)
            & P::shift_down(self.board.get_occupied() | low_ranks);

        self.mobility_area[us as usize] = !(b | self.board.piece_bb(us, PieceType::K)
                | self.pawn_entry.pawn_attacks(us));

        b = king_moves(ksq_us);
        self.attacked_by[us as usize][PieceType::K as usize] = b;
        self.attacked_by[us as usize][PieceType::P as usize] = self.pawn_entry.pawn_attacks(us);

        self.attacked_by2[us as usize] = b & self.attacked_by[us as usize][PieceType::P as usize];
        self.attacked_by_all[us as usize] = b | self.attacked_by[us as usize][PieceType::P as usize];

        if self.board.non_pawn_material(them) >= ROOK_MG + KNIGHT_MG {
            self.king_ring[us as usize] = b;
            if us.relative_rank_of_sq(ksq_us) == Rank::R1 {
                self.king_ring[us as usize] |= P::shift_up(b);
            }

            self.king_attackers_count[them as usize] = (b & self.pawn_entry.pawn_attacks(them)).count_bits();
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
        let outpost_ranks: BitBoard =
            if us == Player::White {BitBoard::RANK_4 | BitBoard::RANK_5 | BitBoard::RANK_6}
                                else {BitBoard::RANK_5 | BitBoard::RANK_4 | BitBoard::RANK_3};

        let mut ps1 = self.board.piece_bb(P::player(), piece);
        let mut score = Score::ZERO;
        let mut b: BitBoard;
        let mut bb: BitBoard;

        while let Some((s, bits)) = ps1.pop_some_lsb_and_bit() {
            b = if piece == PieceType::B {
                let o: BitBoard = self.board.get_occupied() ^ self.board.piece_bb_both_players(PieceType::Q);
                bishop_moves(o,s)
            } else if piece == PieceType::R {
                let o: BitBoard = self.board.get_occupied() ^ self.board.piece_bb_both_players(PieceType::Q);
                rook_moves(o,s)
            } else {
                self.board.attacks_from(piece, s, us)
            };

            if (self.board.pinned_pieces(us) & bits).is_not_empty() {
                b &= line_bb(ksq_us,s);
            }

            self.attacked_by2[us as usize] |= self.attacked_by_all[us as usize] & b;
            self.attacked_by[us as usize][piece as usize] |= b;
            self.attacked_by_all[us as usize] |= self.attacked_by[us as usize][piece as usize];

            if piece == PieceType::Q {
                self.attacked_by_queen_diagonal[us as usize] |= b & bishop_moves(BitBoard(0), s);
            }

            if (b & self.king_ring[them as usize]).is_not_empty() {
                self.king_attackers_count[us as usize] += 1;
                self.king_attackers_weight[us as usize] += KING_ATTACKS_WEIGHT[piece as usize];
                self.king_adjacent_zone_attacks_count[us as usize] += (b & self.attacked_by[them as usize][PieceType::K as usize]).count_bits() as i32;
            }

            let mob: u8 = (b & self.mobility_area[us as usize]).count_bits();

            self.mobility[us as usize] += MOBILITY_BONUS[piece as usize][mob as usize];

            score += KING_PROTECTOR[piece as usize] * distance_of_sqs(s, ksq_us);

            if piece == PieceType::B || piece == PieceType::R {
                bb = outpost_ranks & !self.pawn_entry.pawn_attacks_span(them);
                if (bb & bits).is_not_empty() {
                    score += OUTPOST[(piece == PieceType::B) as usize][(self.attacked_by[us as usize][PieceType::P as usize] & bits).is_not_empty() as usize] * 2;
                } else {
                    bb &= b & !self.board.get_occupied_player(us);
                    if bb.is_not_empty() {
                        score += OUTPOST[(piece == PieceType::B) as usize][(self.attacked_by[us as usize][PieceType::P as usize] & bb).is_not_empty() as usize];
                    }
                }

                if us.relative_rank_of_sq(s) < Rank::R5 &&
                    (self.board.piece_bb_both_players(PieceType::P) & P::shift_up(bits)).is_not_empty() {
                    score += MINOR_BEHIND_PAWN;
                }

            } else if piece == PieceType::R {
                // Bonus for aligning with enemy pawns on the same rank/file
                if us.relative_rank_of_sq(s) >= Rank::R5 {
                    score += ROOK_ON_PAWN * (self.board.piece_bb(them, PieceType::P) * rook_moves(BitBoard(0), s)).count_bits();
                }

                // Bonus when on an open or semi-open file
                if self.pawn_entry.semiopen_file(us, s.file()) {
                    score += ROOK_ON_FILE[self.pawn_entry.semiopen_file(them, s.file()) as usize];
                } else if mob <= 3 {
                    // Penalty when trapped by the king, even more if the king cannot castle
                    let k_file = ksq_us.file();
                    if !((k_file < File::F) && (s.file() < k_file))
                        && !self.pawn_entry.semiopen_side(us, k_file, s.file() < k_file) {
                        score -= (TRAPPED_ROOK - Score(mob as i32 * 22, 0)) * (1 + (self.board.player_can_castle(us).bits() == 0) as u8);
                    }
                }

            } else if piece == PieceType::Q {
                let mut pinners: BitBoard = unsafe {mem::uninitialized()};
                let pieces = self.board.piece_two_bb(PieceType::B, PieceType::R, them);
                self.board.slider_blockers(pieces, s, &mut pinners);
                if pinners.is_not_empty() {
                    score -= WEAK_QUEEN
                }
            }
        }
        score
    }

    fn evaluate_king<P: PlayerTrait>(&mut self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let ksq_us: SQ = self.board.king_sq(us);

        let camp: BitBoard = if us == Player::White { BitBoard::ALL ^ BitBoard::RANK_6 ^ BitBoard::RANK_7 ^ BitBoard::RANK_8}
            else { BitBoard::ALL ^ BitBoard::RANK_1 ^ BitBoard::RANK_2 ^ BitBoard::RANK_3 };

        let weak: BitBoard;
        let mut b: BitBoard;
        let mut b1: BitBoard;
        let mut b2: BitBoard;
        let mut safe_b: BitBoard;
        let mut unsafe_checks: BitBoard = BitBoard(0);

        // King shelter and enemy pawns storm
        let mut score = self.pawn_entry.king_safety::<P>(self.board, ksq_us);

        // Main king safety evaluation
        if self.king_attackers_count[them as usize] as i32 > (1 - self.board.count_piece(them, PieceType::Q) as i32) {
            // Attacked squares defended at most once by our queen or king
            weak = self.attacked_by_all[them as usize]
                    & !self.attacked_by2[us as usize]
                    & (self.attacked_by[us as usize][PieceType::K as usize]
                        | self.attacked_by[us as usize][PieceType::Q as usize]
                        | !self.attacked_by_all[us as usize]);

            let mut king_danger: i32 = 0;

            // Analyse the safe enemy's checks which are possible on next move
            safe_b =  !self.board.get_occupied_player(them);
            safe_b &= !self.attacked_by_all[us as usize] | (weak * self.attacked_by2[them as usize]);

            let us_queen: BitBoard = self.board.piece_bb(us, PieceType::Q);
            b1 = rook_moves(self.board.get_occupied() ^ us_queen, ksq_us);
            b2 = bishop_moves(self.board.get_occupied() ^ us_queen, ksq_us);

            // Enemy queen safe checks
            if ((b1 | b2) & self.attacked_by[them as usize][PieceType::Q as usize] & safe_b
                & !self.attacked_by[us as usize][PieceType::Q as usize]).is_not_empty() {
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

            king_danger +=        self.king_attackers_count[them as usize] as i32 * self.king_attackers_weight[them as usize];
            king_danger += 102 *  self.king_adjacent_zone_attacks_count[them as usize];
            king_danger += 191 * (self.king_ring[us as usize] & weak).count_bits() as i32;
            king_danger += 848 * (self.board.pinned_pieces(us) | unsafe_checks).count_bits() as i32;
            king_danger -= 848 * (self.board.count_piece(them, PieceType::Q) != 0) as i32;
            king_danger -=   9 * score.mg() as i32 / 8;
            king_danger +=  40;

            if king_danger > 0 {
                let mobility_danger = (self.mobility[them as usize] - self.mobility[us as usize]).mg() as i32;
                king_danger = (king_danger + mobility_danger).max(0);
                let mg: Value = (king_danger * king_danger) / 4096;
                let eg: Value = king_danger / 16;
                score -= Score(mg, eg);
            }
        }

        let kf: File = ksq_us.file();

        b = self.attacked_by_all[them as usize] & KING_FLANK[kf as usize] & camp;

        b = (b & self.attacked_by2[them as usize] & !self.attacked_by[us as usize][PieceType::P as usize]) |
            if us == Player::White {b << 4} else {b >> 4};

        score -= CLOSE_ENEMIES * b.count_bits();

        // Penalty when our king is on a pawnless flank
        if (self.board.piece_bb_both_players(PieceType::P) & KING_FLANK[kf as usize]).is_empty() {
            score -= PAWNLESS_FLANK;
        }

        score
    }

    fn evaluate_threats<P: PlayerTrait>(&self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();
        let t_rank_3_bb = if us == Player::White {BitBoard::RANK_3} else {BitBoard::RANK_6};

        let mut b: BitBoard;
        let mut weak: BitBoard;
        let defended: BitBoard;
        let strongly_protected: BitBoard;
        let mut safe_threats: BitBoard;

        let mut score: Score = Score::ZERO;

        // Non-pawn enemies attacked by a pawn
        weak = (self.board.get_occupied_player(them) ^ self.board.piece_bb( them, PieceType::P))
                & self.attacked_by[us as usize][PieceType::P as usize];

        if weak.is_not_empty() {
            b = self.board.piece_bb(us, PieceType::P)
                & (!self.attacked_by_all[them as usize] | self.attacked_by_all[us as usize]);
            safe_threats = (P::shift_up_right(b) | P::shift_up_left(b)) & weak;

            score += THREAT_BY_SAFE_PAWN * safe_threats.count_bits();
        }

        // Squares strongly protected by the opponent, either because they attack the
        // square with a pawn, or because they attack the square twice and we don't.
        strongly_protected = self.attacked_by[them as usize][PieceType::P as usize]
                            | (self.attacked_by2[them as usize] & ! self.attacked_by2[us as usize]);

        // Non-pawn enemies, strongly protected
        defended = (self.board.get_occupied_player(them) ^ self.board.piece_bb( them, PieceType::P)) & strongly_protected;

        // Enemies not strongly protected and under our attack
        weak = self.board.get_occupied_player(them) & !strongly_protected & self.attacked_by_all[us as usize];

        // Add a bonus according to the kind of attacking pieces
        if (defended | weak).is_not_empty() {
            b = (defended | weak) &  (self.attacked_by[us as usize][PieceType::N as usize]
                                    | self.attacked_by[us as usize][PieceType::B as usize]);

            while let Some(s) = b.pop_some_lsb() {
                let piece = self.board.piece_at_sq(s).unwrap();
                score += THREAT_BY_MINOR[piece as usize];
                if piece != PieceType::P {
                    score += THREAT_BY_RANK * them.relative_rank_of_sq(s) as u8;
                }
            }

            b = (self.board.piece_bb(them, PieceType::Q) | weak) & self.attacked_by[us as usize][PieceType::R as usize];
            while let Some(s) = b.pop_some_lsb() {
                let piece = self.board.piece_at_sq(s).unwrap();
                score += THREAT_BY_ROOK[piece as usize];
                if piece != PieceType::P {
                    score += THREAT_BY_RANK * them.relative_rank_of_sq(s) as u8;
                }
            }

            score += HANGING * (weak & !self.attacked_by_all[them as usize]).count_bits();

            b = weak & self.attacked_by[us as usize][PieceType::K as usize];
            if b.is_not_empty() {
                score += THREAT_BY_KING[b.more_than_one() as usize];
            }
        }

        // Bonus for opponent unopposed weak pawns
        if self.board.piece_two_bb(PieceType::R, PieceType::Q, us).is_not_empty() {
            score += WEAK_UNOPOSSED_PAWN * self.pawn_entry.weak_unopposed(them);
        }

        // Find squares where our pawns can push on the next move
        b  = P::shift_up(self.board.piece_bb(them, PieceType::P)) & ! self.board.get_occupied();
        b |= P::shift_up(b & t_rank_3_bb) & ! self.board.get_occupied();

        // Add a bonus for each new pawn threats from those squares
        b = (P::shift_up_left(b) | P::shift_up_right(b))
            & self.board.get_occupied_player(them)
            & !self.attacked_by[us as usize][PieceType::P as usize];

        score += THREAT_BY_PAWN_PUSH * b.count_bits();

        // Add a bonus for safe slider attack threats on opponent queen
        safe_threats = !self.board.get_occupied_player(us)
            & !self.attacked_by2[us as usize]
            & !self.attacked_by2[them as usize];

        b =  (self.attacked_by[us as usize][PieceType::B as usize]
                    & self.attacked_by_queen_diagonal[them as usize])
            | (self.attacked_by[us as usize][PieceType::R as usize]
                & self.attacked_by[us as usize][PieceType::B as usize]
                & !self.attacked_by_queen_diagonal[them as usize]);

        score += THREAT_BY_ATTACK_ON_QUEEN * (b & safe_threats).count_bits();
        score
    }

    fn evaluate_passed_pawns<P: PlayerTrait>(&self) -> Score {
        let us: Player = P::player();
        let them: Player = P::opp_player();

        let mut b: BitBoard;
        let mut bb: BitBoard;
        let mut squares_to_queen: BitBoard;
        let mut defended_squares: BitBoard;
        let mut unsafe_squares: BitBoard;

        let mut score: Score = Score::ZERO;

        b = self.pawn_entry.passed_pawns(us);

        while let Some((s,bits)) = b.pop_some_lsb_and_bit() {
            bb = forward_file_bb(us, s) & (self.attacked_by_all[them as usize] | self.board.get_occupied_player(them));
            score -= HINDER_PASSED_PAWN * bb.count_bits();

            let r: Rank = us.relative_rank_of_sq(s);
            let rr: i32 = RANK_FACTOR[r as usize];

            let mut mbonus: Value = PASSED[0][r as usize];
            let mut ebonus: Value = PASSED[1][r as usize];

            if rr > 0 {
                let block_sq: SQ = P::up(s);

                ebonus += (self.king_distance(them, block_sq) as i32 * 5 - self.king_distance(us, block_sq) as i32 * 2) * rr;

                if r != Rank::R7 {
                    ebonus -= self.king_distance(us, P::up(block_sq)) as i32 * rr;
                }

                if self.board.piece_at_sq(block_sq).is_none() {
                    // If there is a rook or queen attacking/defending the pawn from behind,
                    // consider all the squaresToQueen. Otherwise consider only the squares
                    // in the pawn's path attacked or occupied by the enemy.
                    defended_squares = forward_file_bb(us, s);
                    unsafe_squares = forward_file_bb(us, s);
                    squares_to_queen = forward_file_bb(us, s);

                    bb = self.board.piece_two_bb_both_players(PieceType::R, PieceType::Q)
                        & rook_moves(self.board.get_occupied(),s)  & forward_file_bb(them, s);

                    if (self.board.get_occupied_player(us) & bb).is_empty() {
                        defended_squares &= self.attacked_by_all[us as usize];
                    }

                    if (self.board.get_occupied_player(them) & bb).is_empty() {
                        unsafe_squares &= self.attacked_by_all[them as usize] | self.board.get_occupied_player(them);
                    }

                    // If there aren't any enemy attacks, assign a big bonus. Otherwise
                    // assign a smaller bonus if the block square isn't attacked.
                    let mut k: i32 = if unsafe_squares.is_empty() {18} else if (unsafe_squares & bits).is_empty() {8} else {0};

                    // If the path to the queen is fully defended, assign a big bonus.
                    // Otherwise assign a smaller bonus if the block square is defended.
                    if defended_squares == squares_to_queen {
                        k += 6;
                    } else if (defended_squares & bits).is_not_empty() {
                        k += 4;
                    }

                    mbonus += k * rr;
                    ebonus += k * rr;
                } else if (self.board.get_occupied_player(us) & bits).is_not_empty() {
                    mbonus += rr + r as i32 * 2;
                    ebonus += rr + r as i32 * 2;
                }
            }

            // Scale down bonus for candidate passers which need more than one
            // pawn push to become passed or have a pawn in front of them.
            if !self.board.pawn_passed(us, P::up(s))
                || (self.board.piece_bb_both_players(PieceType::P) & forward_file_bb(us, s)).is_not_empty() {
                mbonus /= 2;
                ebonus /= 2;
            }

            score += Score(mbonus, ebonus) + PASSED_FILE[s.file() as usize];
        }

        score
    }
    // king_distance() returns an estimate of the distance that the king
    // of the given color has to run to reach square s.
    fn king_distance(&self, player: Player, sq: SQ) -> u8 {
        distance_of_sqs(self.board.king_sq(player),sq).min(5)
    }
}
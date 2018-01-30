

use {Player,File,SQ,BitBoard,Board,Piece,Rank};
use super::super::core::masks::{PLAYER_CNT,RANK_CNT};
use super::super::core::score::*;
use super::super::core::mono_traits::*;
use super::super::board::castle_rights::Castling;
use super::super::core::masks::FILE_DISPLAYS;
use core::CastleType;

use super::TableBase;

use std::mem::transmute;
// isolated pawn penalty
const ISOLATED: Score = Score(13, 18);

// backwards pawn penalty
const BACKWARDS: Score = Score(24, 12);

// doubled pawn penalty
const DOUBLED: Score = Score(18, 28);

// Lever bonus by rank
const LEVER: [Score; RANK_CNT] = [
    Score(0,0),
    Score(0,0),
    Score(0,0),
    Score(0,0),
    Score(17,16),
    Score(33,32),
    Score(0,0),
    Score(0,0),
];

const MAX_SAFETY_BONUS: Value = Value(258);


// Weakness of our pawn shelter in front of the king by [isKingFile][distance from edge][rank].
// RANK_1 = 0 is used for files where we have no pawns or our pawn is behind our king.
const SHELTER_WEAKNESS: [[[Value; RANK_CNT]; 4]; 2] = [
        [[ Value( 0), Value( 97), Value(17), Value( 9), Value(44), Value( 84), Value( 87), Value( 99) ], // Not On King file
        [ Value( 0), Value(106), Value( 6), Value(33), Value(86), Value( 87), Value(104), Value(112) ],
        [ Value( 0), Value(101), Value( 2), Value(65), Value(98), Value( 58), Value( 89), Value(115) ],
        [ Value( 0), Value( 73), Value( 7), Value(54), Value(73), Value( 84), Value( 83), Value(111) ] ],
        [ [ Value( 0), Value(104), Value(20), Value( 6), Value(27), Value( 86), Value( 93), Value( 82) ], // On King file
        [ Value( 0), Value(123), Value( 9), Value(34), Value(96), Value(112), Value( 88), Value( 75) ],
        [ Value( 0), Value(120), Value(25), Value(65), Value(91), Value( 66), Value( 78), Value(117) ],
        [ Value( 0), Value( 81), Value( 2), Value(47), Value(63), Value( 94), Value( 93), Value(104) ] ]
];

// Danger of enemy pawns moving toward our king by [type][distance from edge][rank].
// For the unopposed and unblocked cases, RANK_1 = 0 is used when opponent has
// no pawn on the given file, or their pawn is behind our king.
const STORM_DANGER: [[[Value; 5]; 4]; 4] = [
    [ [ Value( 0),  Value(-290), Value(-274), Value(57), Value(41) ],  // BlockedByKing
    [ Value( 0),  Value(  60), Value( 144), Value(39), Value(13) ],
    [ Value( 0),  Value(  65), Value( 141), Value(41), Value(34) ],
    [ Value( 0),  Value(  53), Value( 127), Value(56), Value(14) ] ],
    [ [ Value( 4),  Value(  73), Value( 132), Value(46), Value(31) ],  // Unopposed
    [ Value( 1),  Value(  64), Value( 143), Value(26), Value(13) ],
    [ Value( 1),  Value(  47), Value( 110), Value(44), Value(24) ],
    [ Value( 0),  Value(  72), Value( 127), Value(50), Value(31) ] ],
    [ [ Value( 0),  Value(   0), Value(  79), Value(23), Value( 1) ],  // BlockedByPawn
    [ Value( 0),  Value(   0), Value( 148), Value(27), Value( 2) ],
    [ Value( 0),  Value(   0), Value( 161), Value(16), Value( 1) ],
    [ Value( 0),  Value(   0), Value( 171), Value(22), Value(15) ] ],
    [ [Value(22),  Value(  45), Value( 104), Value(62), Value( 6) ],  // Unblocked
    [ Value(31),  Value(  30), Value(  99), Value(39), Value(19) ],
    [ Value(23),  Value(  29), Value(  96), Value(41), Value(15) ],
    [ Value(21),  Value(  23), Value( 116), Value(41), Value(15) ] ]
];

lazy_static!{
    static ref CONNECTED: [[[[Score; 2]; 2] ;3]; RANK_CNT] = {
        let seed: [i32; 8] = [0, 13, 24, 18, 76, 100, 175, 330];
        let mut a: [[[[Score; 2]; 2] ;3]; 8] = [[[[Score(0,0); 2]; 2] ;3]; 8];
        for opposed in 0..2 {
            for phalanx in 0..2 {
                for support in 0..3 {
                    for r in 1..7 {
                        let mut v: i32 = 17 * support;
                        v += (seed[r] + (phalanx * ((seed[r as usize +1] - seed[r as usize]) / 2))) >> opposed;
                        let eg: i16 = (v * (r as i32 - 2) / 4) as i16;
                        a[r as usize][support as usize][phalanx as usize][opposed as usize] = Score(v as i16, eg);
                    }
                }
            }
        }
        a
    };
}

fn init_connected() -> [[[[Score; 2]; 2] ;3]; RANK_CNT] {
    let seed: [i32; 8] = [0, 13, 24, 18, 76, 100, 175, 330];
    let mut a: [[[[Score; 2]; 2] ;3]; 8] = [[[[Score(0,0); 2]; 2] ;3]; 8];
    for opposed in 0..2 {
        for phalanx in 0..2 {
            for support in 0..3 {
                for r in 1..7 {
                    let mut v: i32 = 17 * support;
                    v += (seed[r] + (phalanx * ((seed[r as usize +1] - seed[r as usize]) / 2))) >> opposed;
                    let eg: i16 = (v * (r as i32 - 2) / 4) as i16;
                    a[r as usize][support as usize][phalanx as usize][opposed as usize] = Score(v as i16, eg);
                }
            }
        }
    }
    a
}

pub struct PawnTable {
    table: TableBase<Entry>,
}

impl PawnTable {
    pub fn new(size: usize) -> Self {
        PawnTable {
            table: TableBase::new(size).unwrap()
        }
    }

    pub fn get(&mut self, key: u64) -> &mut Entry {
        self.table.get_mut(key)
    }


    pub fn clear(&mut self) {
        let size = self.table.size();
        self.table.resize(size);
    }

    pub fn probe(&mut self, board: &Board) -> &mut Entry {
        let key: u64 = board.pawn_key();
        let entry = self.get(key);

        if entry.key == key {
            return entry;
        }

        entry.key = key;
        entry.score = entry.evaluate::<WhiteType>(board) - entry.evaluate::<BlackType>(board);
        entry.asymmetry = (entry.semiopen_files[Player::White as usize] ^ entry.semiopen_files[Player::Black as usize]).count_ones() as i16;
        entry.open_files = (entry.semiopen_files[Player::White as usize] ^ entry.semiopen_files[Player::Black as usize]).count_ones() as u8;
        entry
    }
}

pub struct Entry {
    key: u64,
    score: Score,
    passed_pawns: [BitBoard; PLAYER_CNT],
    pawn_attacks: [BitBoard; PLAYER_CNT],
    pawn_attacks_span: [BitBoard; PLAYER_CNT],
    king_squares: [SQ; PLAYER_CNT],
    king_safety_score: [Score; PLAYER_CNT],
    weak_unopposed: [i16; PLAYER_CNT],
    castling_rights: [Castling; PLAYER_CNT],
    semiopen_files: [u8; PLAYER_CNT],
    // per
    pawns_on_squares: [[u8; PLAYER_CNT]; PLAYER_CNT], // [color][light/dark squares]
    asymmetry: i16,
    open_files: u8
}

impl Entry {
    pub fn pawns_score(&self) -> Score {
        self.score
    }

    pub fn pawn_attacks(&self, player: Player) -> BitBoard {
        self.pawn_attacks[player as usize]
    }

    pub fn passed_pawns(&self, player: Player) -> BitBoard {
        self.passed_pawns[player as usize]
    }

    pub fn pawn_attacks_span(&self, player: Player) -> BitBoard {
        self.pawn_attacks_span[player as usize]
    }

    pub fn weak_unopposed(&self, player: Player) -> i16 {
        self.weak_unopposed[player as usize]
    }

    pub fn asymmetry(&self) -> i16 {
        self.asymmetry
    }

    pub fn open_files(&self) -> u8 {
        self.open_files
    }

    pub fn semiopen_file(&self, player: Player, file: File) -> bool {
        self.semiopen_files[player as usize] & (1 << file as u8) != 0
    }

    pub fn semiopen_side(&self, player: Player, file: File, left_side: bool) -> bool {
        let side_mask: u8 = if left_side {
            file.left_side_mask()
        } else {
            file.right_side_mask()
        };
        self.semiopen_files[player as usize] & side_mask != 0
    }

    // returns count
    pub fn pawns_on_same_color_squares(&self, player: Player, sq: SQ) -> u8 {
        self.pawns_on_squares[player as usize][sq.square_color_index()]
    }

    pub fn king_safety<P: PlayerTrait>(&mut self, board: &Board, ksq: SQ) -> Score {
        if self.king_squares[P::player_idx()] == ksq
            && self.castling_rights[P::player_idx()] == board.player_can_castle(P::player()) {
            self.king_safety_score[P::player_idx()]
        } else {
            self.king_safety_score[P::player_idx()] = self.do_king_safety::<P>(board, ksq);
            self.king_safety_score[P::player_idx()]
        }
    }

    pub fn do_king_safety<P: PlayerTrait>(&mut self, board: &Board, ksq: SQ) -> Score {
        self.king_squares[P::player_idx()] = ksq;
        self.castling_rights[P::player_idx()] = board.player_can_castle(P::player());
        let mut min_king_distance = 0;

        let pawns: BitBoard = board.piece_bb(P::player(),Piece::P);
        if !pawns.is_empty() {
            while (board.magic_helper.ring_distance(ksq, min_king_distance as u8) & pawns).is_empty() {
                min_king_distance += 1;
            }
        }

        let mut bonus: Value = self.shelter_storm::<P>(board,ksq);

        if board.can_castle(P::player(),CastleType::KingSide) {
            bonus = bonus.max( self.shelter_storm::<P>(board, P::player().relative_square(SQ::G1)));
        }

        if board.can_castle(P::player(),CastleType::QueenSide) {
            bonus = bonus.max(self.shelter_storm::<P>(board, P::player().relative_square(SQ::C1)));
        }

        Score::new(bonus, Value(-16 * min_king_distance))
    }


    pub fn shelter_storm<P: PlayerTrait>(&self, board: &Board, ksq: SQ) -> Value {
        let mut b: BitBoard = board.piece_bb_both_players(Piece::P)
            & (board.magic_helper.forward_rank_bb(P::player(), ksq.rank()) | ksq.rank_bb());

        let our_pawns: BitBoard = b & board.get_occupied_player(P::player());
        let their_pawns: BitBoard = b & board.get_occupied_player(P::opp_player());
        let mut safety: Value = MAX_SAFETY_BONUS;
        let center: File = (File::B).max(File::G.min(ksq.file()));

        for file in ((center as u8) - 1)..((center as u8) + 2) {
            b = our_pawns & SQ(file).file_bb();
            let rk_us: Rank = if b.is_empty() {
                Rank::R1
            } else {
                P::player().relative_rank_of_sq(b.backmost_sq(P::player()))
            };

            b = their_pawns & SQ(file).file_bb();
            let rk_them: Rank = if b.is_empty() {
                Rank::R1
            } else {
                P::player().relative_rank_of_sq(b.frontmost_sq(P::opp_player()))
            };
            let d: File = unsafe { (transmute::<u8,File>(file)).min(!transmute::<u8,File>(file)) };

            let r = if file == ksq.file() as u8 {
                1
            } else {
                0
            };
            let storm_danger_idx: usize = if file == ksq.file() as u8 && P::player().relative_rank_of_sq(ksq) as u8 + 1 == rk_them as u8 {
                0   // Blocked By King
            } else if rk_us == Rank::R1 {
                1  // Unopossed
            } else if rk_them as u8 == rk_us as u8 + 1 {
                2  // Blocked by Pawn
            } else {
                3  // Unblocked
            };
            if d >= File::E {
                println!("file: {}, num: {}",FILE_DISPLAYS[file as usize], file);
                println!("flip: {}", FILE_DISPLAYS[d as usize]);
            }
            safety -= SHELTER_WEAKNESS[r as usize][d as usize][rk_us as usize];
            if rk_them <= Rank::R5 {
                safety -= STORM_DANGER[storm_danger_idx][d as usize][rk_them as usize];
            }
        }
        safety
    }

    pub fn evaluate<P: PlayerTrait>(&mut self, board: &Board) -> Score {
        let mut b: BitBoard;
        let mut neighbours: BitBoard;
        let mut stoppers: BitBoard;
        let mut doubled: BitBoard;
        let mut supported: BitBoard;
        let mut phalanx: BitBoard;
        let mut lever: BitBoard;
        let mut lever_push: BitBoard;
        let mut opposed: bool;
        let mut backward: bool;

        let mut score: Score = Score::ZERO;
        let our_pawns: BitBoard = board.piece_bb(P::player(), Piece::P);
        let their_pawns: BitBoard = board.piece_bb(P::opp_player(), Piece::P);

        let mut p1: BitBoard = our_pawns;

        self.passed_pawns[P::player() as usize] =  BitBoard(0);
        self.pawn_attacks_span[P::player() as usize] = BitBoard(0);
        self.weak_unopposed[P::player() as usize] = 0;
        self.semiopen_files[P::player() as usize] = 0xFF;
        self.king_squares[P::player() as usize] = SQ::NO_SQ;
        self.pawn_attacks[P::player() as usize] = P::shift_up_left(our_pawns) | P::shift_up_right(our_pawns);

        let pawns_on_dark: u8 = (our_pawns & BitBoard::DARK_SQUARES).count_bits();
        self.pawns_on_squares[P::player() as usize][Player::Black as usize] = pawns_on_dark;
        if pawns_on_dark > board.count_piece(P::player(),Piece::P) {
            println!("Error: pawns on dark: {}, total: {}",pawns_on_dark, board.count_piece(P::player(),Piece::P));
        }
        self.pawns_on_squares[P::player() as usize][Player::White as usize] = board.count_piece(P::player(),Piece::P) - pawns_on_dark;

        while let Some(s) = p1.pop_some_lsb() {
            assert_eq!(board.piece_at_sq(s).unwrap(),Piece::P);

            let f: File = s.file();

            self.semiopen_files[P::player() as usize] &= !(1 << f as u8);
            self.pawn_attacks[P::player() as usize] |= board.magic_helper.pawn_attacks_span(P::player(), s);

            opposed = (their_pawns & board.magic_helper.forward_file_bb(P::player(),s)).is_not_empty();
            stoppers = their_pawns & board.magic_helper.passed_pawn_mask(P::player(),s);
            lever = their_pawns & board.magic_helper.pawn_attacks_from(s, P::player());
            lever_push = their_pawns & board.magic_helper.pawn_attacks_from(P::up(s), P::player());
            doubled = our_pawns & (P::down(s)).to_bb();
            neighbours = our_pawns & board.magic_helper.adjacent_file(f);
            phalanx = neighbours & s.rank_bb();
            supported = neighbours & (P::up(s)).rank_bb();

            // A pawn is backward when it is behind all pawns of the same color on the
            // adjacent files and cannot be safely advanced.
            if neighbours.is_empty() || !lever.is_empty() || P::player().relative_rank_of_sq(s) >= Rank::R5 {
                backward = false;
            } else {
                // Find the backmost rank with neighbours or stoppers
                b = (neighbours | stoppers).backmost_sq(P::player()).rank_bb();

                // The pawn is backward when it cannot safely progress to that rank:
                // either there is a stopper in the way on this rank, or there is a
                // stopper on adjacent file which controls the way to that rank.
                backward = ((b | P::shift_up(b & board.magic_helper.adjacent_file(f))) & stoppers).is_not_empty();

                assert!(!(backward && (board.magic_helper.forward_rank_bb(P::opp_player(), P::up(s).rank()) & neighbours).is_not_empty()));
            }
            // Passed pawns will be properly scored in evaluation because we need
            // full attack info to evaluate them. Include also not passed pawns
            // which could become passed after one or two pawn pushes when are
            // not attacked more times than defended.
            if    (stoppers ^ lever ^ lever_push).is_empty()
                && (our_pawns & board.magic_helper.forward_file_bb(P::player(),s)).is_empty()
                && supported.count_bits() >= lever.count_bits()
                && phalanx.count_bits()   >= lever_push.count_bits() {
                self.passed_pawns[P::player() as usize] |= s.to_bb();
            } else if stoppers == P::up(s).to_bb() && P::player().relative_rank_of_sq(s) >= Rank::R5 {
                b = P::shift_up(supported) & !their_pawns;
                while let Some(b_sq) = b.pop_some_lsb() {
                    if !(their_pawns & board.magic_helper.pawn_attacks_from(b_sq, P::player())).more_than_one() {
                        self.passed_pawns[P::player() as usize] |= s.to_bb();
                    }
                }
            }

            if supported.is_not_empty() | supported.is_not_empty() {
                score += CONNECTED[P::player().relative_rank_of_sq(s) as usize][supported.count_bits() as usize][phalanx.is_not_empty() as usize][opposed as usize];
            } else if neighbours.is_empty() {
                score -= ISOLATED;
                self.weak_unopposed[P::player() as usize] += (!opposed) as i16;
            } else if backward {
                score -= BACKWARDS;
                self.weak_unopposed[P::player() as usize] += (!opposed) as i16;
            }

            if doubled.is_not_empty() && supported.is_empty() {
                score -= DOUBLED;
            }

            if lever.is_not_empty() {
                score += LEVER[P::player().relative_rank_of_sq(s) as usize];
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Board;

    #[test]
    fn pawn_eval() {
        let mut t: PawnTable = PawnTable::new(1 << 7);
        let boards: Vec<Board> = Board::random().pseudo_random(2222212).many(15);
        let mut score: i64 = 0;
        boards.iter().for_each(|b| {
            score += t.probe(b).pawns_score().0 as i64;
        });
    }

}
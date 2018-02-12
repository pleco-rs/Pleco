

#[allow(unused_imports)]
use pleco::{Board,BitBoard,SQ,Rank,File,Player,PieceType};
#[allow(unused_imports)]
use pleco::core::mono_traits::*;
use pleco::core::score::*;
use pleco::core::masks::*;

use tables::pawn_table::{PawnEntry, PawnTable};
use tables::material::{MaterialEntry,Material};

const LAZY_THRESHOLD: Value = 1500;

pub struct Evaluation<'a> {
    board: &'a Board,
    pawn_entry: &'a mut PawnEntry,
    material_entry: &'a mut MaterialEntry,
    king_ring: [BitBoard; PLAYER_CNT],
    mobility_area: [BitBoard; PLAYER_CNT],
    mobility: [Score; PLAYER_CNT],
    attacked_by: [[Score; PIECE_TYPE_CNT];PLAYER_CNT],
    attacked_by2: [Score;PLAYER_CNT],
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
            attacked_by: [[Score(0,0); PIECE_TYPE_CNT];PLAYER_CNT],
            attacked_by2: [Score(0,0) ;PLAYER_CNT],
            king_attackers_count: [0; PLAYER_CNT],
            king_attackers_weight: [0; PLAYER_CNT],
            king_adjacent_zone_attacks_count: [0; PLAYER_CNT],
        };

        eval.value()
    }

    fn value(&mut self) -> Value {
        let score = self.pawn_entry.pawns_score() + self.material_entry.score();
        let v = (score.0 + score.1) / 2;
        if v.abs() > LAZY_THRESHOLD {
            if self.board.turn() == Player::White {return v;}
            else {return -v;}
        }

        return v;
    }

//    fn initialize<P: PlayerTrait>(&mut self) {
//        let low_ranks: BitBoard = if P::player() == Player::White {Ra | RANK_3} else {RANK_6 | RANK_8};
//
//        // Find our pawns on the first two ranks, and those which are blocked
//        let mut b: BitBoard = self.board.piece_bb(P::player(), PieceType::P)
//            & P::shift_down(self.board.get_occupied() | low_ranks);
//
//        self.mobility_area[P::player() as usize] = !(b | self.board.piece_bb(P::player(), PieceType::K)
//                | self.pawn_entry.pawn_attacks(P::player()));
//
//
//    }
}
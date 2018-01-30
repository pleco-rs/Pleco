
use pleco::{Board,BitBoard,SQ,Rank,File,Player};
use pleco::core::mono_traits::*;
use pleco::core::score::*;

use tables::pawn_table::{Entry,PawnTable};

pub struct Evaluation<'a> {
    board: &'a Board,
    pawn_entry: &'a mut Entry,
}

impl <'a> Evaluation <'a> {
    pub fn evaluate(board: &Board, pawn_table: &mut PawnTable) -> i16 {
        let score: Score = Score(0,0);

        let pawn_entry = { pawn_table.probe(&board) };
        let mut eval = Evaluation {
            board,
            pawn_entry: pawn_entry
        };


        unimplemented!()
    }

}
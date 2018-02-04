
#[allow(unused_imports)]
use pleco::{Board,BitBoard,SQ,Rank,File,Player};
#[allow(unused_imports)]
use pleco::core::mono_traits::*;
use pleco::core::score::*;

use tables::pawn_table::{PawnEntry, PawnTable};

pub struct Evaluation<'a> {
    board: &'a Board,
    pawn_entry: &'a mut PawnEntry,
}

impl <'a> Evaluation <'a> {
    pub fn evaluate(board: &Board, pawn_table: &mut PawnTable) -> i16 {
        #[allow(unused_variables)]
        let score: Score = Score(0,0);

        let pawn_entry = { pawn_table.probe(&board) };

        let mut _eval = Evaluation {
            board,
            pawn_entry: pawn_entry
        };


        unimplemented!()
    }

}
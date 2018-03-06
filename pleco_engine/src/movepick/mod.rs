mod pick;

#[allow(unused_imports)]
use pleco::{BitMove,Board,ScoringMove,ScoringMoveList};
use pleco::board::movegen::{Legality,PseudoLegal,MoveGen};
use pleco::core::mono_traits::*;

use self::pick::*;

// TODO: use Generators once stabilized.

pub trait MovePickerType: Sized {
    fn next(&mut self, board: &Board, skip_quiets: bool) -> BitMove;
}


pub struct MovePicker<MPT: MovePickerType> {
    picker: MPT,
    board: *const Board,
    moves: ScoringMoveList
}

impl MovePicker<MainSearchPicker> {
    pub fn main_search(board: &Board, depth: u16, mut ttm: BitMove,
                       killers: &[BitMove; 2], counter_move: BitMove) -> Self {
        assert!(!board.in_check());

        if ttm == BitMove::null() || !board.pseudo_legal_move(ttm) {
            ttm = BitMove::null();
        }
        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = unsafe {moves.as_mut_ptr()};
        let picker = MainSearchPicker::new(depth, ttm, killers[0], killers[1], counter_move, first);
        MovePicker::new(*&board, picker, moves)
    }
}

impl <MPT: MovePickerType> MovePicker<MPT> {
    fn new(board: *const Board, picker: MPT, moves: ScoringMoveList) -> Self {
        MovePicker {
            picker,
            board,
            moves
        }
    }
}

struct MainSearchPicker {
    pick: PickMain,
    depth: u16,
    ttm: BitMove,
    killer1: BitMove,
    killer2: BitMove,
    counter_move: BitMove,
    begin_ptr: *mut ScoringMove,
    end_ptr: *mut ScoringMove
}

impl MainSearchPicker {
    pub fn new(depth: u16, ttm: BitMove, killer1: BitMove, killer2: BitMove, counter_move: BitMove, mvs: *mut ScoringMove) -> Self {
        let pick = if ttm == BitMove::null() {PickMain::CapturesInit} else {PickMain::MainSearch};
        MainSearchPicker {
            pick,
            depth,
            ttm,
            killer1,
            killer2,
            counter_move,
            begin_ptr: mvs,
            end_ptr: mvs
        }
    }

}

impl MovePickerType for MainSearchPicker {
    fn next(&mut self, board: &Board, skip_quiets: bool) -> BitMove {
        match self.pick {
            PickMain::MainSearch => {
                self.pick.incr();
                return self.ttm;
            },
            PickMain::CapturesInit => {
                unsafe {
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,CapturesGenType, ScoringMoveList>
                        (board, self.begin_ptr);
                }
                self.next(board, skip_quiets)
            },
//            PickMain::GoodCaptures => {
//
//            },
//            PickMain::KillerOne => {},
//            PickMain::KillerTwo => {},
//            PickMain::CounterMove => {},
//            PickMain::QuietInit => {},
//            PickMain::QuietMoves => {},
            _ => BitMove::null()
        }
    }
}
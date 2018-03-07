mod pick;

use std::ptr;
#[allow(unused_imports)]
use pleco::{BitMove,Board,ScoringMove,ScoringMoveList};
use pleco::board::movegen::{PseudoLegal,MoveGen};
use pleco::helper::prelude::piece_value;
use pleco::core::mono_traits::*;

use self::pick::*;

// TODO: use Generators once stabilized.

pub struct MovePicker {
    pick: Pick,
    board: *const Board,
    moves: ScoringMoveList,
    depth: u16,
    ttm: BitMove,
    killers: [BitMove; 2],
    cm: BitMove,

    cur_ptr: *mut ScoringMove,
    end_ptr: *mut ScoringMove,
    end_bad_captures: *mut ScoringMove,
}

impl MovePicker {
    pub fn main_search(board: &Board, depth: u16, mut ttm: BitMove,
                       killers: &[BitMove; 2], counter_move: BitMove) -> Self {
        assert!(!board.in_check());

        if ttm == BitMove::null() || !board.pseudo_legal_move(ttm) {
            ttm = BitMove::null();
        }

        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = moves.as_mut_ptr();

        MovePicker {
            pick: Pick::MainSearch,
            board: &* board,
            moves,
            depth,
            ttm,
            killers: (*killers).clone(),
            cm: counter_move,
            cur_ptr: first,
            end_ptr: first,
            end_bad_captures: first,
        }
    }

    fn score_captures(&mut self, board: &Board) {
        let mut ptr = self.cur_ptr;
        unsafe {
            while ptr < self.end_ptr {
                let mov: BitMove = (*ptr).bit_move;
                let piece_moved = board.moved_piece(mov);
                let piece_cap = board.captured_piece(mov).unwrap();
                (*ptr).score = piece_value(piece_cap,false) as i16
                    - piece_value(piece_moved,false) as i16;
                ptr = ptr.add(1);
            }
        }
    }

    fn pick_best(&self, begin: *mut ScoringMove, end: *mut ScoringMove) -> ScoringMove {
        unsafe {
            let mut best_score = begin;
            let mut cur = begin.add(1);
            while cur < end {
                if (*cur).score > (*best_score).score {
                    best_score = cur;
                }
                cur = cur.add(1);
            }
            ptr::swap(begin, best_score);
            *begin
        }
    }


    pub fn next(&mut self, board: &Board, skip_quiets: bool) -> BitMove {
        let mut mov: ScoringMove = ScoringMove::null();
        match self.pick {
            Pick::MainSearch => {
                self.pick.incr();
                return self.ttm;
            },
            Pick::CapturesInit => {
                unsafe {
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,CapturesGenType, ScoringMoveList>
                        (board, self.cur_ptr);
                }
                self.score_captures(board);
                self.pick.incr();
                self.next(board, skip_quiets)
            },
            Pick::GoodCaptures => {
                while self.cur_ptr < self.end_ptr {
                    mov = self.pick_best(self.cur_ptr, self.end_ptr);
                    unsafe {self.cur_ptr = self.cur_ptr.add(1);}
                    if mov.bit_move != self.ttm && mov.score > -20 {
                        return mov.bit_move;
                    }
                    unsafe {
                        *self.end_bad_captures = mov;
                        self.end_bad_captures = self.end_bad_captures.add(1);
                    }
                }
                self.pick.incr();
                self.next(board, skip_quiets)
            },
            Pick::KillerOne | Pick::KillerTwo => {
                mov.bit_move = self.killers[self.pick as usize - Pick::KillerOne as usize];
                self.pick.incr();
                if mov.bit_move != BitMove::null()
                    && mov.bit_move != self.ttm
                    && board.pseudo_legal_move(mov.bit_move)
                    && !mov.bit_move.is_capture() {
                    return mov.bit_move;
                }
                self.next(board, skip_quiets)
            },
            Pick::CounterMove => {
                self.pick.incr();
                if self.cm != BitMove::null()
                    && self.cm != self.ttm
                    && self.cm != self.killers[0]
                    && self.cm != self.killers[1]
                    && board.pseudo_legal_move(self.cm)
                    && !self.cm.is_capture() {
                    return self.cm;
                }
                self.next(board, skip_quiets)
            },
            Pick::QuietInit => {
                unsafe {
                    self.cur_ptr = self.end_bad_captures;
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,CapturesGenType, ScoringMoveList>(
                        board, self.cur_ptr);
                    // TODO: Need to score the captures
                }
                return BitMove::null();
            }
            _ => BitMove::null()
        }
    }
}

fn partial_insertion_sort(begin: *mut ScoringMove, end: *mut ScoringMove, limit: i32) {
    unsafe {
        let mut sorted_end: *mut ScoringMove = begin;
        let mut p: *mut ScoringMove = begin.add(1);
        while p < end {
            if (*p).score as i32 >= limit {
                let tmp: ScoringMove = *p;
                sorted_end = sorted_end.add(1);
                *p = *sorted_end;
                let mut q = sorted_end;
                while q != begin && *(q.sub(1)) < tmp {
                    *q = *(q.sub(1));
                    q = q.sub(1);
                }
                *q = tmp;
            }
            p = p.add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    use std::i16::{MAX,MIN};

    #[test]
    fn mp_partial_insertion_sort() {
        let mut moves = ScoringMoveList::default();
        moves.push_score(BitMove::null(), 34);
        moves.push_score(BitMove::null(), 20);
        moves.push_score(BitMove::null(), -5);
        moves.push_score(BitMove::null(), 50);
        moves.push_score(BitMove::null(), 3);
        moves.push_score(BitMove::null(), 0);
        moves.push_score(BitMove::null(), 9);
        moves.push_score(BitMove::null(), -1);
        moves.push_score(BitMove::null(), 2);

        let len = moves.len();
        let begin = moves.get_mut(0).unwrap() as *mut ScoringMove;
        let end = unsafe {
            begin.add(len)
        };

        partial_insertion_sort(begin, end, 4);
    }

    #[test]
    fn mp_partial_insertion_sort_rand() {
        for _x in 0..10 {
            partial_insertion_t();
        }
    }


    fn partial_insertion_t() {
        let min = 10;
        let max = 200;
        let num = (rand::random::<u16>() % (max - min)) + min;

        let mut moves = ScoringMoveList::default();

        for _x in 0..num {
            let rand_score = rand::random::<i16>();
            moves.push_score(BitMove::null(),rand_score);
        }

        let limit: i16 = rand::random::<i16>()
            .max(MIN + 10)
            .min(MAX - 10);

        let len = moves.len();
        let begin = moves.get_mut(0).unwrap() as *mut ScoringMove;
        let end = unsafe {
            begin.add(len)
        };

        partial_insertion_sort(begin, end, limit as i32);

        let mut unsorted_idx = 0;

        while unsorted_idx < len {
            if moves[unsorted_idx].score < limit {
                break;
            }
            unsorted_idx += 1;
        }

        for x in 0..unsorted_idx {
            assert!(moves[x].score >= limit);
        }

        for x in unsorted_idx..len {
            assert!(moves[x].score < limit);
        }
    }
}

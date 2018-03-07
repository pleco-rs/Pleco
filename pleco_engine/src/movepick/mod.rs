mod pick;

use std::ptr;
use std::mem;

#[allow(unused_imports)]
use pleco::{BitMove,Board,ScoringMove,ScoringMoveList,SQ};
use pleco::board::movegen::{PseudoLegal,MoveGen};
use pleco::helper::prelude::piece_value;
use pleco::core::mono_traits::*;

use self::pick::*;

// TODO: use Generators once stabilized.

pub struct MovePicker {
    pick: Pick,
    board: *const Board,
    moves: ScoringMoveList,
    depth: i16,
    ttm: BitMove,
    killers: [BitMove; 2],
    cm: BitMove,
    recapture_sq: SQ,
    threshold: i32,
    cur_ptr: *mut ScoringMove,
    end_ptr: *mut ScoringMove,
    end_bad_captures: *mut ScoringMove,
}

impl MovePicker {
    /// MovePicker constructor for the main search
    pub fn main_search(board: &Board, depth: i16, mut ttm: BitMove,
                       killers: &[BitMove; 2], counter_move: BitMove) -> Self {
        assert!(depth > 0);
        let mut pick = if board.in_check() {Pick::EvasionSearch} else {Pick::MainSearch};

        if ttm == BitMove::null() || !board.pseudo_legal_move(ttm) {
            ttm = BitMove::null();
            pick.incr();
        }

        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = moves.as_mut_ptr();

        MovePicker {
            pick,
            board: &* board,
            moves,
            depth,
            ttm,
            killers: (*killers).clone(),
            cm: counter_move,
            recapture_sq: unsafe {mem::uninitialized()},
            threshold: unsafe {mem::uninitialized()},
            cur_ptr: first,
            end_ptr: first,
            end_bad_captures: first,
        }
    }

    /// MovePicker constructor for quiescence search
    pub fn qsearch(board: &Board, depth: i16, ttm: BitMove, recapture_sq: SQ) -> Self {
        assert!(depth <= 0);
        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = moves.as_mut_ptr();
        let mut mp_qs = MovePicker {
            pick: unsafe { mem::uninitialized() },
            board: &*board,
            moves,
            depth,
            ttm,
            killers: unsafe { mem::uninitialized() },
            cm: unsafe { mem::uninitialized() },
            recapture_sq: unsafe { mem::uninitialized() },
            threshold: unsafe {mem::uninitialized()},
            cur_ptr: first,
            end_ptr: first,
            end_bad_captures: first,
        };

        if board.in_check() {
            mp_qs.pick = Pick::EvasionSearch;
        } else if depth > -5 {
            mp_qs.pick = Pick::QSearch;
        } else {
            mp_qs.pick = Pick::QSearchRecaptures;
            mp_qs.recapture_sq = recapture_sq;
            return mp_qs;
        }

        if ttm == BitMove::null() || !board.pseudo_legal_move(ttm) {
            mp_qs.ttm = BitMove::null();
            mp_qs.pick.incr()
        }

        mp_qs
    }

    /// MovePicker constructor for ProbCut: we generate captures with SEE higher
    /// than or equal to the given threshold.
    pub fn probcut_search(board: &Board, threshold: i32, mut ttm: BitMove, recapture_sq: SQ) -> Self {
        assert!(!board.in_check());
        let mut moves = ScoringMoveList::default();
        let first: *mut ScoringMove = moves.as_mut_ptr();

        let mut pick = Pick::ProbCutSearch;

        ttm = if ttm != BitMove::null()
            && board.pseudo_legal_move(ttm)
            && board.is_capture(ttm) {
            ttm
        } else {
            pick.incr();
            BitMove::null()
        };

        MovePicker {
            pick,
            board: &*board,
            moves,
            depth: unsafe { mem::uninitialized() },
            ttm,
            killers: unsafe { mem::uninitialized() },
            cm: unsafe { mem::uninitialized() },
            recapture_sq: recapture_sq,
            threshold,
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

    fn score_evasions(&mut self, board: &Board) {
        let mut ptr = self.cur_ptr;
        unsafe {
            while ptr < self.end_ptr {
                let mov: BitMove = (*ptr).bit_move;
                if board.is_capture(mov) {
                    let piece_moved = board.moved_piece(mov);
                    let piece_cap = board.captured_piece(mov).unwrap();
                    (*ptr).score = piece_value(piece_cap,false) as i16
                        - piece_value(piece_moved,false) as i16;
                }
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
            Pick::MainSearch | Pick::EvasionSearch | Pick::QSearch | Pick::ProbCutSearch => {
                self.pick.incr();
                return self.ttm;
            },
            Pick::CapturesInit | Pick::ProbCutCapturesInit | Pick::QSearchInit | Pick::QSearchRecaptures => {
                unsafe {
                    self.end_bad_captures = self.moves.as_mut_ptr();
                    self.cur_ptr = self.end_bad_captures;
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,CapturesGenType, ScoringMoveList>
                        (board, self.cur_ptr);
                }
                self.score_captures(board);
                self.pick.incr();
                return self.next(board, skip_quiets);
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
                return self.next(board, skip_quiets);
            },
            Pick::KillerOne | Pick::KillerTwo => {
                mov.bit_move = self.killers[self.pick as usize - Pick::KillerOne as usize];
                self.pick.incr();
                if mov.bit_move != BitMove::null()
                    && mov.bit_move != self.ttm
                    && board.pseudo_legal_move(mov.bit_move)
                    && !board.is_capture(mov.bit_move) {
                    return mov.bit_move;
                }
                return self.next(board, skip_quiets);
            },
            Pick::CounterMove => {
                self.pick.incr();
                if self.cm != BitMove::null()
                    && self.cm != self.ttm
                    && self.cm != self.killers[0]
                    && self.cm != self.killers[1]
                    && board.pseudo_legal_move(self.cm)
                    && !board.is_capture(self.cm) {
                    return self.cm;
                }
                return self.next(board, skip_quiets);
            },
            Pick::QuietInit => {
                unsafe {
                    self.cur_ptr = self.end_bad_captures;
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,QuietsGenType, ScoringMoveList>(
                        board, self.cur_ptr);
                    // TODO: Need to score the captures
                }
                self.pick.incr();
                return self.next(board, skip_quiets);
            },
            Pick::QuietMoves => {
                if !skip_quiets {
                    while self.cur_ptr < self.end_ptr {
                        unsafe {
                            mov = *self.cur_ptr;
                            self.cur_ptr = self.cur_ptr.add(1);
                            if mov.bit_move != self.ttm
                                && mov.bit_move != self.killers[0]
                                && mov.bit_move != self.killers[1]
                                && mov.bit_move != self.cm {
                                return mov.bit_move;
                            }
                        }
                    }
                }
                self.pick.incr();
                self.cur_ptr = self.moves.as_mut_ptr();
                return self.next(board, skip_quiets);
            },
            Pick::BadCaptures => {
                if self.cur_ptr < self.end_bad_captures {
                    unsafe  {
                        mov = *self.cur_ptr;
                        self.cur_ptr = self.cur_ptr.add(1);
                    }
                    return mov.bit_move
                }
            },
            Pick::EvasionsInit => {
                unsafe {
                    self.cur_ptr = self.moves.as_mut_ptr();
                    self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,EvasionsGenType,ScoringMoveList>
                        (board, self.cur_ptr);
                }
                self.score_evasions(board);
                self.pick.incr();
                return self.next(board, skip_quiets);
            },
            Pick::AllEvasions => {
                while self.cur_ptr < self.end_ptr {
                    mov = self.pick_best(self.cur_ptr, self.end_ptr);
                    unsafe {self.cur_ptr = self.cur_ptr.add(1);}
                    if mov.bit_move != self.ttm {
                        return mov.bit_move;
                    }
                }
            },
            Pick::ProbCutCaptures => {
                while self.cur_ptr < self.end_ptr {
                    mov = self.pick_best(self.cur_ptr, self.end_ptr);
                    unsafe {self.cur_ptr = self.cur_ptr.add(1);}
                    if mov.bit_move != self.ttm {
                        return mov.bit_move;
                    }
                }
            },
            Pick::QCaptures => {
                while self.cur_ptr < self.end_ptr {
                    mov = self.pick_best(self.cur_ptr, self.end_ptr);
                    unsafe {self.cur_ptr = self.cur_ptr.add(1);}
                    if mov.bit_move != self.ttm {
                        return mov.bit_move;
                    }
                }

                if self.depth > -1 {
                    unsafe {
                        self.cur_ptr = self.moves.as_mut_ptr();
                        self.end_ptr = MoveGen::extend_from_ptr::<PseudoLegal,QuietChecksGenType,ScoringMoveList>
                            (board, self.cur_ptr);
                    }
                    self.pick.incr();
                    return self.next(board, skip_quiets);
                }
            },
            Pick::QChecks => {
                while self.cur_ptr < self.end_ptr {
                    unsafe {
                        mov = *self.cur_ptr;
                        self.cur_ptr = self.cur_ptr.add(1);
                    }
                    if mov.bit_move != self.ttm {
                        return mov.bit_move;
                    }
                }

            },
            Pick::QRecaptures => {
                while self.cur_ptr < self.end_ptr {
                    unsafe {
                        mov = *self.cur_ptr;
                        self.cur_ptr = self.cur_ptr.add(1);
                        if mov.bit_move.get_dest() == self.recapture_sq {
                            return mov.bit_move;
                        }
                    }
                }
            }
        }
        BitMove::null()
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
    use pleco::MoveList;
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

    #[test]
    fn movepick_startpos_blank() {
        movepick_main_search(Board::default(), BitMove::null(), &[BitMove::null(); 2],
                             BitMove::null(), 5);
    }

    #[test]
    fn movepick_startpos_rand_op() {
        let b = Board::default();
        for _x in 0..25 {
            movepick_rand_one(b.clone());
        }
    }

    #[test]
    fn movepick_rand_mainsearch() {
        for _x in 0..20 {
            let mut b = Board::random().one();
            while b.checkmate() {
                b = Board::random().one();
            }
            movepick_rand_one(b);
//            println!("pass movepick rand! {}",_x);
        }
    }

    #[test]
    fn movepick_incorrect_move1() {
    //    MovePicker Returned an incorrect move: 39 at index 0
    //    Incorrect Move: d1f3,
    //    depth: 4232, fen: rnb1k2r/pppp1ppp/7n/2b1P3/4P3/2P5/PP3PPP/RN1QKBNR w KQkq - 1 6
    //    in check?: false
    //    ttm: a5e2b bits: 54048
    //    killer1: d1f3 bits: 5443
    //    killer1: c2f8q bits: 65354
    //    counter: d6h2q bits: 62443', pleco_engine\src\movepick\mod.rs:523:17

        let b = Board::from_fen("rnb1k2r/pppp1ppp/7n/2b1P3/4P3/2P5/PP3PPP/RN1QKBNR w KQkq - 1 6").unwrap();
        let ttm = BitMove::new(54048);
        let killers = [BitMove::new(5443), BitMove::new(65354)];
        let depth = 4232;
        let cm = BitMove::new(62443);
        movepick_main_search(b, ttm, &killers, cm, depth);
    }

    
    #[test]
    fn movepick_incorrect_move2() {
    //    MovePicker Returned an incorrect move: e2c3 at index 0, bits: 29836
    //    Real Length: 30, MovePicker Length: 31,
    //
    //    depth: 15927, fen: r4r2/1n1k1pp1/p1p4p/5n1P/1PpPB3/6PR/P3NP2/3RK3 w - - 4 28
    //    in check?: false
    //    ttm: f4h6 bits: 31709
    //    killer1: c7f7 bits: 3442
    //    killer1: e2c3 bits: 29836
    //    counter: e1f2 bits: 836', pleco_engine\src\movepick\mod.rs:541:17

        let b = Board::from_fen("r4r2/1n1k1pp1/p1p4p/5n1P/1PpPB3/6PR/P3NP2/3RK3 w - - 4 28").unwrap();
        let ttm = BitMove::new(31709);
        let killers = [BitMove::new(3442), BitMove::new(29836)];
        let depth = 15927;
        let cm = BitMove::new(836);
        movepick_main_search(b, ttm, &killers, cm, depth);
    }

    fn movepick_rand_one(b: Board) {
        let ttm = BitMove::new(rand::random());
        let cm = BitMove::new(rand::random());
        let killers = [BitMove::new(rand::random()),BitMove::new(rand::random())];
        let depth = rand::random::<i16>().abs().max(1).min(127);
        movepick_main_search(b, ttm, &killers, cm, depth);
    }



    fn movepick_main_search(b: Board, ttm: BitMove, killers: &[BitMove; 2], cm: BitMove, depth: i16) {
        let real_moves = b.generate_pseudolegal_moves();
        let mut mp = MovePicker::main_search(&b, depth, ttm, &killers, cm);

        let mut moves_mp = MoveList::default();
        let mut mp_next = mp.next(&b, false);

        while mp_next != BitMove::null() {
            moves_mp.push(mp_next);
            mp_next = mp.next(&b, false);
        }

        // Check to see if the MovePicker gives all the right moves
        for (i, mov) in real_moves.iter().enumerate() {
            if !moves_mp.contains(mov) {
                panic!("\nMovePicker is missing this move: {} at index {}, bits: {}\
                \n Real Length: {}, MovePicker Length: {},
                \n depth: {}, fen: {}\
                \n in check?: {}\
                \n ttm: {} bits: {} \
                \n killer1: {} bits: {}\
                \n killer2: {} bits: {}\
                \n counter: {} bits: {}",
                       mov, i, mov.get_raw(),
                       real_moves.len(), moves_mp.len(),
                       depth, b.fen(),
                       b.in_check(),
                       ttm, ttm.get_raw(),
                       killers[0], killers[0].get_raw(),
                       killers[1], killers[1].get_raw(),
                       cm, cm.get_raw());
            }
        }

        for (i, mov) in moves_mp.iter().enumerate() {
            if !real_moves.contains(mov) {
                panic!("\nMovePicker Returned an incorrect move: {} at index {}, bits: {}\
                \n Real Length: {}, MovePicker Length: {},
                \n depth: {}, fen: {}\
                \n in check?: {}\
                \n ttm: {} bits: {} \
                \n killer1: {} bits: {}\
                \n killer2: {} bits: {}\
                \n counter: {} bits: {}",
                       mov, i, mov.get_raw(),
                       real_moves.len(), moves_mp.len(),
                       depth, b.fen(),
                       b.in_check(),
                       ttm, ttm.get_raw(),
                       killers[0], killers[0].get_raw(),
                       killers[1], killers[1].get_raw(),
                       cm, cm.get_raw());
            }
        }

        if moves_mp.len() != real_moves.len() {
            panic!("\nMovePicker did not return the correct number of moves: {}, Actual number: {}\
                \n depth: {}, fen: {}\
                \n in check?: {}\
                \n ttm: {} bits: {} \
                \n killer1: {} bits: {}\
                \n killer2: {} bits: {}\
                \n counter: {} bits: {}",
                   moves_mp.len(), real_moves.len(),
                   depth, b.fen(),
                   b.in_check(),
                   ttm, ttm.get_raw(),
                   killers[0], killers[0].get_raw(),
                   killers[1], killers[1].get_raw(),
                   cm, cm.get_raw());
        }
    }
}

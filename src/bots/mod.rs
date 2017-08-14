pub mod bot_random;
pub mod bot_minimax;
pub mod bot_parallel_minimax;
pub mod bot_alphabeta;
pub mod bot_jamboree;
pub mod bot_advanced;
pub mod bot_expert;

use piece_move::BitMove;

pub struct BestMove {
    best_move: Option<BitMove>,
    score: i16,
}

impl BestMove {
    pub fn new(score: i16) -> Self {
        BestMove {
            best_move: None,
            score: score,
        }
    }

    pub fn negate(mut self) -> Self {
        self.score = self.score.wrapping_neg();
        self
    }
}

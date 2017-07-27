use board::*;
use timer::*;
use piece_move::*;
use engine::Searcher;
use bots::eval::*;
use test;
use test::Bencher;
use timer;


const MAX_PLY: u16 = 3;

pub struct SimpleBot {
    board: Board,
    timer: Timer,
}

pub struct BestMove {
    best_move: Option<BitMove>,
    score: i16,
}

impl BestMove {
    pub fn new(score: i16) -> Self {
        BestMove{
            best_move: None,
            score: score
        }
    }

    pub fn negate(&mut self) {
        self.score.wrapping_neg();
    }
}


// depth: depth from given
// half_moves: total moves

impl Searcher for SimpleBot {

    fn name() -> &'static str {
        "Simple Searcher"
    }

    fn best_move(board: Board, timer: Timer) -> BitMove {
        SimpleBot::best_move_depth(board, timer, MAX_PLY)
    }

    fn best_move_depth(board: Board, timer: Timer, max_depth: u16) -> BitMove {
        let mut b = SimpleBot {board: board, timer: timer};
        minimax(&mut b, max_depth).best_move.unwrap()
    }

}

fn minimax(bot: &mut SimpleBot, max_depth: u16) -> BestMove {
//    println!("depth = {}", bot.board.depth());
    if bot.board.depth() == max_depth {

       return eval_board(bot);
    }

    let moves = bot.board.generate_moves();
    if moves.len() == 0 {
        if bot.board.in_check() {
            return BestMove::new(NEG_INFINITY - (bot.board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }
    let mut best_value: i16 = NEG_INFINITY;
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        bot.board.apply_move(mov);
        let mut returned_move: BestMove = minimax(bot, max_depth);
        returned_move.negate();
        bot.board.undo_move();
        if returned_move.score > best_value {
            best_value = returned_move.score;
            best_move = Some(mov);
        }
    }
    BestMove{best_move: best_move, score: best_value}

}

fn eval_board(bot: &mut SimpleBot) -> BestMove {
    BestMove::new(Eval::eval(&bot.board))
}


#[bench]
fn bench_bot_ply_4__simple_bot(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 2;
        (0..iter).fold(0, |a: u64, c| {
            let mov = SimpleBot::best_move_depth(b.shallow_clone(),timer::Timer::new(20),4);
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}


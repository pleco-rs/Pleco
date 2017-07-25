use board::*;
use timer::*;
use piece_move::*;
use engine::Searcher;
use eval::*;


const MAX_PLY: u16 = 5;

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
        self.score.wrapping_mul(-1);
    }
}


// depth: depth from given
// half_moves: total moves

impl Searcher for SimpleBot {

    fn name() -> &'static str {
        "Simple Searcher"
    }

    fn best_move(board: Board, timer: Timer) -> BitMove {
        let mut b = SimpleBot {board: board, timer: timer};
        minimax(&mut b).best_move.unwrap()
    }

}

fn minimax(bot: &mut SimpleBot) -> BestMove {
//    println!("depth = {}", bot.board.depth());
    if bot.board.depth() == MAX_PLY {

       return eval_board(bot);
    }

    let moves = bot.board.generate_moves();
//    println!("length = {}", moves.len());
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
        // && mov.stringify().to_owned().eq("d7d6")
//        if bot.board.depth() == 3 && mov.stringify().to_owned().eq("d7d6")
//            && bot.board.zobrist() == 15536998096656736002 {
//            println!("Applying move: {}. Depth is: {}",mov,bot.board.depth());
//            println!("Move raw {}",mov.get_raw());
//
//            println!("Board looks like this beforehand:");
//            bot.board.fancy_print();
//            println!();
//            println!();
//            println!("Applying Move");
//            bot.board.apply_move(mov);
//            println!("Move applied");
//        } else if bot.board.depth() == 2 && mov.stringify().to_owned().eq("d1a4")
//            && bot.board.zobrist() == 17959625815994702195 {
//            println!("Applying move: {}. Depth is: {}",mov,bot.board.depth());
//            println!("Move raw {}",mov.get_raw());
//            println!("Board looks like this beforehand:");
//            bot.board.fancy_print();
//            println!();
//            println!();
//            println!("Applying Move");
//            bot.board.apply_move(mov);
//            println!("Move applied");
//        } else {
//            bot.board.apply_move(mov);
//        }

        println!("Applying move: {}. Depth is: {}",mov,bot.board.depth());
        println!("Move raw {}",mov.get_raw());
        println!("Board looks like this beforehand:");
        bot.board.fancy_print();
        println!();
        println!();
        println!("Applying Move");
            bot.board.apply_move(mov);

        let mut returned_move: BestMove = minimax(bot);
        returned_move.negate();
        bot.board.undo_move();
//        println!("undoing");
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


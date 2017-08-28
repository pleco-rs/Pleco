extern crate pleco;
extern crate rand;

use pleco::templates::print_bitboard;
use pleco::engine::*;
use pleco::timer::Timer;

use pleco::{board,piece_move,templates,timer};

use pleco::bots::basic::bot_minimax::SimpleBot;
use pleco::bots::basic::bot_random::RandomBot;
use pleco::bots::basic::bot_parallel_minimax::ParallelSearcher;
use pleco::bots::basic::bot_alphabeta::AlphaBetaBot;
use pleco::bots::basic::bot_jamboree::JamboreeSearcher;

use pleco::bots::threaded_searcher::ThreadSearcher;
use pleco::bots::bot_expert::ExpertBot;
use pleco::bots::bot_iterative_parallel_mvv_lva::IterativeSearcher;


fn main() {

}





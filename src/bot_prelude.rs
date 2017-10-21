//! Easy importing of all available bots.

pub use bots::basic::bot_random::RandomBot;
pub use bots::basic::bot_minimax::SimpleBot;
pub use bots::basic::bot_parallel_minimax::ParallelSearcher;
pub use bots::basic::bot_alphabeta::AlphaBetaBot;
pub use bots::basic::bot_jamboree::JamboreeSearcher;

pub use bots::bot_iterative_parallel_mvv_lva::IterativeSearcher;
pub use bots::lazy_smp::LazySMPSearcher;
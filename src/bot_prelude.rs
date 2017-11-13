//! Easy importing of all available bots.

pub use bots::RandomBot;
pub use bots::MiniMaxSearcher;
pub use bots::ParallelMiniMaxSearcher;
pub use bots::AlphaBetaSearcher;
pub use bots::JamboreeSearcher;
pub use bots::IterativeSearcher;

pub use pleco_searcher::lazy_smp::PlecoSearcher;

pub use engine::Searcher;
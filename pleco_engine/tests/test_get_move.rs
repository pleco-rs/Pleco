extern crate pleco;
extern crate pleco_engine;

use pleco::Board;
use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;

pub fn get_move(fen: String, depth: u16) -> String {
    let mut limit = PreLimits::blank();
    limit.depth = Some(depth);
    let board = Board::from_fen(fen.as_str()).unwrap();
    let mut s = PlecoSearcher::init(false);

    s.search(&board, &limit);
    let bit_move = s.await_move();

    bit_move.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = get_move(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            10,
        );
        assert_eq!(result, "e2e4");
    }
}

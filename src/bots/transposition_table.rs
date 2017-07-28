use piece_move::*;

// https://docs.rs/chashmap/2.2.0/chashmap/struct.CHashMap.html
// https://docs.rs/chashmap/2.2.0/chashmap/
// chashmap

pub type Key = u64;

pub struct TranspositionTable {

}

pub struct Value {
    best_move: BitMove, // What was the best move found here?
    score: u16, // What was the Evlauation of this Node?
    depth: u8, // How deep was this Score Found?
    node_type: NodeType,


}

pub enum NodeType {
    UpperBound,
    LowerBound,
    Exact
}


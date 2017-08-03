use piece_move::*;
use chashmap::CHashMap;

// https://docs.rs/chashmap/2.2.0/chashmap/struct.CHashMap.html
// https://docs.rs/chashmap/2.2.0/chashmap/
// chashmap

pub type Key = u64;

pub type TranspositionTable = CHashMap<Key, Value>;

pub fn init_tt() -> TranspositionTable {
    TranspositionTable::with_capacity(10000)
}

// 2 bytes + 2 bytes + 1 Byte + 1 byte = ?6 Bytes
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



extern crate chrono;

use super::THREAD_STACK_SIZE;
use pleco::core::piece_move::BitMove;
use pleco::core::masks::PLAYER_CNT;

use std::mem;

pub trait PVNode {
    fn is_pv() -> bool;
}

pub struct PV {}
pub struct NonPV {}

impl PVNode for PV {
    fn is_pv() -> bool {
        true
    }
}

impl PVNode for NonPV {
    fn is_pv() -> bool {
        false
    }
}


pub struct ThreadStack {
    pub pos_eval: i32,
}

impl ThreadStack {
    pub fn new() -> Self {
        ThreadStack {
            pos_eval: 0
        }
    }
}

pub fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}

pub enum LimitsType {
    Time,
    Infinite,
    Perft,
    Mate,
    MoveTime,
    Depth,
    Nodes,
}

pub struct PreLimits {
    max_depth: u16,
    max_nodes: u64,
    time_msec: [i32; PLAYER_CNT],
    inc_msec: [i32; PLAYER_CNT],
    moves_to_go: u32,
    nodes: Option<u32>,
    mate: u32,
    move_time: u32,
    infinite: bool,
    search_moves: Vec<BitMove>,
}

pub struct Limits {

}

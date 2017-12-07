
extern crate chrono;

use super::THREAD_STACK_SIZE;
use pleco::core::piece_move::BitMove;
use pleco::core::masks::PLAYER_CNT;
use std::time;

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

#[derive(Clone)]
pub enum LimitsType {
    Time(UCITimer), // use time limits
    MoveTime(u64), // search for exactly x msec
    Mate(u16), // Search for a mate in x moves
    Depth(u16), // Search only x plys
    Nodes(u64), // Search only x nodes
    Infinite, // infinite
    Ponder   // ponder mode
}

#[derive(Clone)]
pub struct UCITimer {
    pub time_msec: [i32; PLAYER_CNT], // time each player has remaining
    pub inc_msec: [i32; PLAYER_CNT], // increments for each palyer after each turn
    pub moves_to_go: u32, // Moves to go until next time control sent
}

impl UCITimer {
    pub fn blank() -> Self {
        UCITimer {
            time_msec: [0;PLAYER_CNT],
            inc_msec: [0;PLAYER_CNT],
            moves_to_go: 0
        }
    }
}

#[derive(Clone)]
pub struct PreLimits {
    pub time: Option<UCITimer>,
    pub move_time: Option<u64>,
    pub nodes: Option<u64>,
    pub depth: Option<u16>,
    pub mate: Option<u16>,
    pub infinite: bool,
    pub ponder: bool,
    pub search_moves: Vec<BitMove>,
}

impl PreLimits {
    pub fn blank() -> Self {
        PreLimits {
            time: None,
            move_time: None,
            nodes: None,
            depth: None,
            mate: None,
            infinite: false,
            ponder: false,
            search_moves: Vec::new()
        }
    }
    pub fn create(self) -> Limits {
        let mut limits = Limits {
            search_moves: self.search_moves.clone(),
            limits_type: LimitsType::Infinite,
            start: time::Instant::now()
        };

        limits.limits_type = if self.ponder {
            LimitsType::Ponder
        } else if let Some(m_time) = self.move_time {
            LimitsType::MoveTime(m_time)
        } else if let Some(mate) = self.mate {
            LimitsType::Mate(mate)
        } else if let Some(depth) = self.depth {
            LimitsType::Depth(depth)
        } else if let Some(nodes) = self.nodes {
            LimitsType::Nodes(nodes)
        } else if self.infinite  {
            LimitsType::Infinite
        } else if let Some(timer) = self.time {
            LimitsType::Time(timer)
        } else {
            LimitsType::Infinite
        };
        limits
    }
}

#[derive(Clone)]
pub struct Limits {
    pub search_moves: Vec<BitMove>,
    pub limits_type: LimitsType,
    pub start: time::Instant
}

use chrono;
use pleco::core::masks::PLAYER_CNT;
use std::time;

#[derive(Clone)]
pub enum LimitsType {
    Time(UCITimer), // use time limits
    MoveTime(u64),  // search for exactly x msec
    Mate(u16),      // Search for a mate in x moves
    Depth(u16),     // Search only x plys
    Nodes(u64),     // Search only x nodes
    Infinite,       // infinite
    Ponder,         // ponder mode
}

impl LimitsType {
    pub fn is_depth(&self) -> bool {
        match *self {
            LimitsType::Depth(_x) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct UCITimer {
    pub time_msec: [i64; PLAYER_CNT], // time each player has remaining
    pub inc_msec: [i64; PLAYER_CNT],  // increments for each palyer after each turn
    pub moves_to_go: u32,             // Moves to go until next time control sent
}

impl UCITimer {
    pub fn blank() -> Self {
        UCITimer {
            time_msec: [0; PLAYER_CNT],
            inc_msec: [0; PLAYER_CNT],
            moves_to_go: 0,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.time_msec[0] == 0
            && self.time_msec[1] == 0
            && self.inc_msec[0] == 0
            && self.inc_msec[1] == 0
    }

    pub fn display(&self) {
        println!(
            "time: [{}, {}], inc: [{}, {}], moves to go: {}",
            self.time_msec[0],
            self.time_msec[1],
            self.inc_msec[0],
            self.inc_msec[1],
            self.moves_to_go
        );
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
    pub search_moves: Vec<String>,
}

impl PreLimits {
    pub fn print(&self) {
        if let Some(ref time) = self.time {
            println!(
                "time_msec: W = {}, B = {}",
                time.time_msec[0], time.time_msec[1]
            );
            println!(
                "inc_msec: W = {}, B = {}",
                time.inc_msec[0], time.inc_msec[1]
            );
            println!("movestogo: {}", time.moves_to_go);
        }
        if let Some(move_time) = self.move_time {
            println!("move_time: {}", move_time)
        }
        if let Some(nodes) = self.nodes {
            println!("nodes: {}", nodes)
        }
        if let Some(depth) = self.depth {
            println!("depth: {}", depth)
        }
        if let Some(mate) = self.mate {
            println!("move_time: {}", mate)
        }
        println!("infinite: {}", self.infinite);
        println!("ponder: {}", self.ponder);
        if self.search_moves.len() > 1 {
            print!("search_moves:");
            self.search_moves.iter().for_each(|p| print!(" {}", p));
            println!();
        }
    }
    pub fn blank() -> Self {
        PreLimits {
            time: None,
            move_time: None,
            nodes: None,
            depth: None,
            mate: None,
            infinite: false,
            ponder: false,
            search_moves: Vec::new(),
        }
    }

    pub fn create(self) -> Limits {
        let mut limits = Limits {
            search_moves: self.search_moves.clone(),
            limits_type: LimitsType::Infinite,
            start: time::Instant::now(),
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
        } else if self.infinite {
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
    pub search_moves: Vec<String>,
    pub limits_type: LimitsType,
    pub start: time::Instant,
}

impl Limits {
    pub fn use_time_management(&self) -> Option<UCITimer> {
        match self.limits_type {
            LimitsType::Time(ref timer) => Some(timer.clone()),
            _ => None,
        }
    }

    pub fn blank() -> Self {
        Limits {
            search_moves: Vec::new(),
            limits_type: LimitsType::Infinite,
            start: time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> i64 {
        chrono::Duration::from_std(self.start.elapsed())
            .unwrap()
            .num_milliseconds()
    }

    pub fn use_movetime(&self) -> Option<u64> {
        match self.limits_type {
            LimitsType::MoveTime(time) => Some(time),
            _ => None,
        }
    }
}

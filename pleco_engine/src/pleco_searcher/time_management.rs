use chrono;

use pleco::Player;
use super::uci_timer::UCITimer;

use std::cell::UnsafeCell;
use std::time::Instant;
use std::cmp::{min,max};

#[derive(PartialEq)]
enum TimeCalc {
    Ideal,
    Max
}

impl TimeCalc {
    pub fn ratio(&self, moves_to_go: bool) -> f64 {
        if moves_to_go {
            match *self {
                TimeCalc::Ideal => 1.0,
                TimeCalc::Max => 0.07
            }
        } else {
            match *self {
                TimeCalc::Ideal => 0.017,
                TimeCalc::Max => 3.0
            }
        }
    }
}

pub struct TimeManager {
    ideal_time: UnsafeCell<i64>,
    maximum_time: UnsafeCell<i64>,
    start: UnsafeCell<Instant>
}

unsafe impl Sync for TimeManager {}

impl TimeManager {
    pub fn blank() -> TimeManager {
        TimeManager {
            ideal_time: UnsafeCell::new(0),
            maximum_time: UnsafeCell::new(0),
            start: UnsafeCell::new(Instant::now())
        }
    }

    pub fn init(&self, start: Instant, timer: &UCITimer, turn: Player, ply: u16) {
        let move_num = (ply as u32 + 1) / 2;
        let move_overhead: i64 = 1;
        let ideal_time = TimeManager::remaining(timer.time_msec[turn as usize],
                                                timer.time_msec[turn as usize],
                                                move_overhead,
                                                timer.moves_to_go,
                                                move_num,
                                                false,
                                                TimeCalc::Ideal);

        let max_time = TimeManager::remaining(timer.time_msec[turn as usize],
                                                timer.time_msec[turn as usize],
                                                move_overhead,
                                                timer.moves_to_go,
                                                move_num,
                                                false,
                                                TimeCalc::Max);
        unsafe {
            let self_start = self.start.get();
            let self_ideal = self.ideal_time.get();
            let self_max = self.maximum_time.get();
            *self_start = start;
            *self_ideal = ideal_time;
            *self_max = max_time;
        }
    }

    pub fn elapsed(&self) -> i64 {
        unsafe {
            let start = &*self.start.get();
            chrono::Duration::from_std(start.elapsed())
                .unwrap()
                .num_milliseconds()
        }
    }

    fn remaining(my_time: i64, my_inc: i64, move_overhead: i64, movestogo: u32, move_num: u32, ponder: bool, time_type: TimeCalc) -> i64 {
        if my_time <= 0 {
            return 0;
        }
        let inc: f64 = my_inc as f64 * (55.0 as f64).max(120.0 - 0.12 * f64::from((move_num - 25) * (move_num - 25)));

        let ratio: f64 = if movestogo != 0 {
            let mut pre_ratio: f64 = time_type.ratio(true) / f64::from(min(50, movestogo));
            if move_num <= 40 {
                pre_ratio *= 1.1 - 0.001 * ((move_num - 20) * (move_num - 20)) as f64;
            } else {
                pre_ratio *= 1.5;
            }

            if movestogo > 1 {
                pre_ratio = pre_ratio.min(0.75);
            }

            pre_ratio * (1.0 + inc / (my_time as f64 * 8.5))
        } else {
            let k: f64 = 1.0 + 20.0 * move_num as f64 / (500.0 + move_num as f64);
            time_type.ratio(false) * (k + inc / my_time as f64)
        };

        let time: i64 = min(1, ratio as i64) * max(0, my_time - move_overhead);

        if time_type == TimeCalc::Ideal && ponder {
            5 * time / 4
        } else {
            time
        }
    }

    pub fn maximum_time(&self) -> i64 {
        unsafe {
            *self.maximum_time.get()
        }
    }

    pub fn ideal_time(&self) -> i64 {
        unsafe {
            *self.ideal_time.get()
        }
    }


}
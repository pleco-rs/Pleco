//! Time Management calculations for the searcher.

use chrono;

use super::uci_timer::UCITimer;
use pleco::Player;

use std::cell::UnsafeCell;
use std::f64;
use std::time::Instant;

const MOVE_HORIZON: i64 = 50;
const MAX_RATIO: f64 = 6.32;
const STEAL_RATIO: f64 = 0.34;

// TODO: These should be made into UCIOptions
const MIN_THINKING_TIME: i64 = 20;
const MOVE_OVERHEAD: i64 = 100;

// Lower values means places less importance on the current move
const SLOW_MOVER: i64 = 22;

#[derive(PartialEq)]
enum TimeCalc {
    Ideal,
    Max,
}

impl TimeCalc {
    #[inline(always)]
    pub fn t_max_ratio(&self) -> f64 {
        match *self {
            TimeCalc::Ideal => 1.0,
            TimeCalc::Max => MAX_RATIO,
        }
    }

    #[inline(always)]
    pub fn t_steal_ratio(&self) -> f64 {
        match *self {
            TimeCalc::Ideal => 0.0,
            TimeCalc::Max => STEAL_RATIO,
        }
    }
}

pub struct TimeManager {
    ideal_time: UnsafeCell<i64>,
    maximum_time: UnsafeCell<i64>,
    start: UnsafeCell<Instant>,
}

unsafe impl Sync for TimeManager {}

impl TimeManager {
    pub fn uninitialized() -> TimeManager {
        TimeManager {
            ideal_time: UnsafeCell::new(0),
            maximum_time: UnsafeCell::new(0),
            start: UnsafeCell::new(Instant::now()),
        }
    }

    pub fn start_timer(&self, start: Instant) {
        unsafe {
            let self_start = self.start.get();
            *self_start = start;
        }
    }

    pub fn init(&self, start: Instant, timer: &UCITimer, turn: Player, ply: u16) {
        let moves_to_go: i64 = timer.moves_to_go as i64;
        let my_time: i64 = (timer.time_msec[turn as usize]) as i64;
        let my_inc: i64 = (timer.inc_msec[turn as usize]) as i64;

        let mut ideal_time = (timer.time_msec[turn as usize]).max(MIN_THINKING_TIME);
        let mut max_time = ideal_time;

        let max_mtg: i64 = if moves_to_go == 0 {
            MOVE_HORIZON
        } else {
            moves_to_go.min(MOVE_HORIZON)
        };

        // We calculate optimum time usage for different hypothetical "moves to go"-values
        // and choose the minimum of calculated search time values. Usually the greatest
        // hypMTG gives the minimum values.
        for hyp_mtg in 1..=max_mtg {
            let mut hyp_my_time: i64 =
                my_time + my_inc * (hyp_mtg - 1) - MOVE_OVERHEAD * (2 + hyp_mtg.min(40));
            hyp_my_time = hyp_my_time.max(0);

            let t1: i64 = MIN_THINKING_TIME
                + TimeManager::remaining(
                    hyp_my_time,
                    hyp_mtg,
                    ply as i64,
                    SLOW_MOVER,
                    TimeCalc::Ideal,
                );
            let t2: i64 = MIN_THINKING_TIME
                + TimeManager::remaining(
                    hyp_my_time,
                    hyp_mtg,
                    ply as i64,
                    SLOW_MOVER - 5,
                    TimeCalc::Max,
                );

            ideal_time = t1.min(ideal_time);
            max_time = t2.min(max_time);
        }

        unsafe {
            let self_start = self.start.get();
            let self_ideal = self.ideal_time.get();
            let self_max = self.maximum_time.get();
            *self_start = start;
            *self_ideal = ideal_time;
            *self_max = max_time;
        }
    }

    pub fn start(&self) -> Instant {
        unsafe { *self.start.get() }
    }

    pub fn elapsed(&self) -> i64 {
        let start = self.start();
        chrono::Duration::from_std(start.elapsed())
            .unwrap()
            .num_milliseconds()
    }

    fn move_importance(ply: i64) -> f64 {
        const X_SCALE: f64 = 6.85;
        const X_SHIFT: f64 = 64.5;
        const SKEW: f64 = 0.171;

        let exp: f64 = ((ply as f64 - X_SHIFT) / X_SCALE).exp();
        let base: f64 = 1.0 + exp;
        base.powf(-SKEW) + f64::MIN_POSITIVE
    }

    fn remaining(
        my_time: i64,
        movestogo: i64,
        move_num: i64,
        slow_mover: i64,
        time_type: TimeCalc,
    ) -> i64 {
        let slow_move_f: f64 = slow_mover as f64;
        let t_max_ratio: f64 = time_type.t_max_ratio();
        let t_steal_ratio: f64 = time_type.t_steal_ratio();

        let move_importance: f64 = (TimeManager::move_importance(move_num) * slow_move_f) / 100.0;
        let mut other_moves_importance: f64 = 0.0;

        for i in 1..movestogo {
            other_moves_importance += TimeManager::move_importance(move_num + 2 * i);
        }

        let ratio1: f64 = (t_max_ratio * move_importance)
            / (t_max_ratio * move_importance + other_moves_importance);
        let ratio2: f64 = (move_importance + t_steal_ratio * other_moves_importance)
            / (move_importance + other_moves_importance);

        (my_time as f64 * ratio1.min(ratio2)) as i64
    }

    #[inline(always)]
    pub fn maximum_time(&self) -> i64 {
        unsafe { *self.maximum_time.get() }
    }

    #[inline(always)]
    pub fn ideal_time(&self) -> i64 {
        unsafe { *self.ideal_time.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_man() {
        let timer = UCITimer {
            time_msec: [120000, 0],
            inc_msec: [6000, 0],
            moves_to_go: 20,
        };
        let ply: u16 = 0;
        let time_man = TimeManager::uninitialized();
        time_man.init(Instant::now(), &timer, Player::White, ply);
        let max = time_man.maximum_time();
        let ideal = time_man.ideal_time();
        println!("ideal: {} max: {}", ideal, max);
    }
}

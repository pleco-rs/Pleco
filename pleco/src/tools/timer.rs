//! Timer for keeping track of the time for both sides of the a chess game.
use std::time::Instant;

// Structure to keep track of time for two players.
#[derive(Clone, Copy)]
pub struct Timer {
    start: Instant,   // when the current timer was created
    milli_seconds_remaining: [i64; 2],
    turn: Turn, // turn of the current clock
    inc: [i64; 2], // Amount to incremeant per player if there is time left
}

#[derive(Clone, Copy)]
enum Turn {
    One = 0,
    Two = 1,
}

impl Timer {
    /// Creates a new `Timer` with a set number of milliseconds left for each player.
    pub fn new_no_inc(milliseconds: i64) -> Self {
        Timer::new(milliseconds, milliseconds, 0, 0)
    }

    /// Creates a new `Timer`.
    pub fn new(player_one_msec: i64, player_two_msec: i64, player_one_inc: i64, player_two_inc: i64) -> Self {
        Timer {
            start: Instant::now(),
            milli_seconds_remaining: [player_one_msec, player_two_msec],
            turn: Turn::One,
            inc: [player_one_inc, player_two_inc]
        }
    }

    /// Returns the number of milliseconds remaining while running for the current turn.
    pub fn msec_remaining(&self) -> i64 {
        let diff = self.start.elapsed();
        self.milli_seconds_remaining[self.turn as usize] - diff.as_secs() as i64
    }

    /// Returns the time increment for the current turn's player.
    pub fn current_time_inc(&self) -> i64 {
        match self.turn {
            Turn::One => self.inc[0],
            Turn::Two => self.inc[1],
        }
    }

    /// Returns the time increment for the current turn's opposing player.
    pub fn opponent_time_inc(&self) -> i64 {
        match self.turn {
            Turn::One => self.inc[1],
            Turn::Two => self.inc[0],
        }
    }

    /// Returns the number of milliseconds remaining for the opponent.
    pub fn opp_time_remaining(&self) -> i64 {
        self.milli_seconds_remaining[other_turn(self.turn) as usize]
    }

    /// Starts the timer.
    pub fn start_time(&mut self) {
        self.start = Instant::now();
    }

    /// Stops the timer.
    pub fn stop_time(&mut self) {
        let diff = self.start.elapsed();
        self.milli_seconds_remaining[self.turn as usize] -= diff.as_secs() as i64;
        if !self.out_of_time() {
            self.milli_seconds_remaining[self.turn as usize] += self.inc[self.turn as usize];
        }
    }

    /// Switches the turn.
    pub fn switch_turn(&mut self) {
        self.turn = other_turn(self.turn);
    }

    /// Returns if the current player is out of time.
    pub fn out_of_time(&self) -> bool {
        self.milli_seconds_remaining[self.turn as usize] <= 0
    }
}

fn other_turn(turn: Turn) -> Turn {
    match turn {
        Turn::One => Turn::Two,
        Turn::Two => Turn::One,
    }
}

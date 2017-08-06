use chrono::{DateTime,Utc};


// Structure to keep track of time for two players


pub struct Timer {
    start: i64,   // when the current timer was created
    total_duration: i64, // unchanging, seconds each
    seconds_remaining: [i64; 2],
    turn: Turn, // turn of the current clock
}

#[derive(Clone,Copy)]
enum Turn {
    One = 0,
    Two = 1,
}

impl Timer {
    pub fn new(minutes: i64) -> Self {
        let secs = minutes * 60;
        Timer {
            start: 0,
            total_duration: secs,
            seconds_remaining: [secs, secs],
            turn: Turn::One
        }
    }

    pub fn time_remaining(&self) -> i64 {
        let utc: DateTime<Utc> = Utc::now();
        let end = utc.timestamp();
        let diff = end - self.start;
        self.seconds_remaining[self.turn as usize] - diff
    }

    pub fn opp_time_remaining(&self) -> i64 {
        self.seconds_remaining[other_turn(self.turn) as usize]
    }

    pub fn start_time(&mut self) {
        let utc: DateTime<Utc> = Utc::now();
        self.start = utc.timestamp();
    }

    pub fn stop_time(&mut self) {
        let utc: DateTime<Utc> = Utc::now();
        let end = utc.timestamp();
        let diff = end - self.start;
        self.seconds_remaining[self.turn as usize] -= diff;
    }

    pub fn switch_turn(&mut self) {
        self.turn = other_turn(self.turn);
    }

    pub fn out_of_time(&self) -> bool {
        self.seconds_remaining[self.turn as usize] <= 0
    }
}

fn other_turn(turn: Turn) -> Turn {
    match turn {
        Turn::One => Turn::Two,
        Turn::Two => Turn::One
    }
}
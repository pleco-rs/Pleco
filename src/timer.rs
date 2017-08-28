use std::time::Instant;


// Structure to keep track of time for two players
#[derive(Clone, Copy)]
pub struct Timer {
    start: Instant,   // when the current timer was created
    total_duration: i64, // unchanging, seconds each
    seconds_remaining: [i64; 2],
    turn: Turn, // turn of the current clock
    inc: [i64; 2], // Amount to incremeant per player if there is time left
}

#[derive(Clone, Copy)]
enum Turn {
    One = 0,
    Two = 1,
}

impl Timer {
    pub fn new_no_inc(minutes: i64) -> Self {
        Timer::new(minutes, 0, 0)
    }

    pub fn new(minutes: i64, player_one_inc: i64, player_two_inc: i64) -> Self {
        let secs = minutes * 60;
        Timer {
            start: Instant::now(),
            total_duration: secs,
            seconds_remaining: [secs, secs],
            turn: Turn::One,
            inc: [player_one_inc, player_two_inc]
        }
    }

    pub fn time_remaining(&self) -> i64 {
        let diff = self.start.elapsed();
        self.seconds_remaining[self.turn as usize] - diff.as_secs() as i64
    }

    pub fn current_time_inc(&self) -> i64 {
        match self.turn {
            Turn::One => self.inc[0],
            Turn::Two => self.inc[1],
        }
    }

    pub fn opponent_time_inc(&self) -> i64 {
        match self.turn {
            Turn::One => self.inc[1],
            Turn::Two => self.inc[0],
        }
    }

    pub fn opp_time_remaining(&self) -> i64 {
        self.seconds_remaining[other_turn(self.turn) as usize]
    }

    pub fn start_time(&mut self) {
        self.start = Instant::now();
    }

    pub fn stop_time(&mut self) {
        let diff = self.start.elapsed();
        self.seconds_remaining[self.turn as usize] -= diff.as_secs() as i64;
        if !self.out_of_time() {
            self.seconds_remaining[self.turn as usize] += self.inc[self.turn as usize];
        }
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
        Turn::Two => Turn::One,
    }
}

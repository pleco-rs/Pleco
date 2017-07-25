use chrono::DateTime;



pub struct Timer {
    start: i64,
    end: i64
}

impl Timer {
    pub fn new(minutes: u32) -> Self {
        Timer {
            start: 0,
            end: minutes as i64
        }
    }
}

pub mod search;
pub mod threadpool;
pub mod misc;
pub mod options;

use engine::UCILimit;
use tools::tt::TT;
use Board;

use self::options::{UciOption,AllOptions};


const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(256);
}

static mut LIMIT: UCILimit = UCILimit::Infinite;



pub struct _PlecoSearcher {
    board: Option<Board>,
    options: AllOptions
}



impl _PlecoSearcher {

    pub fn clear_tt(&mut self) {
        unsafe {TT_TABLE.clear() };
    }

    pub fn apply_option(&mut self, name: &str) {
        unimplemented!()
    }
}


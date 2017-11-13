pub mod lazy_smp;
pub mod search;
pub mod threadpool;
pub mod misc;

use engine::UCILimit;
use tools::tt::TT;

use self::threadpool::ThreadPool;

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(256);
}

static mut LIMIT: UCILimit = UCILimit::Infinite;

pub struct _PlecoSearcher {
    thread_pool: ThreadPool
}

impl _PlecoSearcher {

    pub fn tt_size(mb: usize) {
        unimplemented!()
    }

    pub fn clear_tt() {
        unimplemented!()
    }

}
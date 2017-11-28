use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU16,Ordering};
use std::thread::{JoinHandle,self};
use std::sync::mpsc::{channel,Receiver,Sender};

use std::{mem,time};

use pleco::board::*;
use pleco::core::*;
use pleco::board::eval::*;
use pleco::core::piece_move::BitMove;
use pleco::tools::tt::*;
use pleco::engine::*;

use super::thread_search::ThreadSearcher;
use super::misc::*;
use super::{TT_TABLE,THREAD_STACK_SIZE,MAX_PLY};

pub struct ThreadGo {
    limit: UCILimit,
    board: Board,
    // Options: ?
}

pub enum SendData {
    BestMove(RootMove)
}

pub type RootMoves = Arc<RwLock<Vec<RootMove>>>;
pub type AllRootMoves = Arc<RwLock<Vec<RootMoves>>>;

pub struct ThreadPool {
    // This is the position information we send to each thread upon
    // starting. Contains stuff like the Board, and the Limit to search
    // to.
    pos_state: Arc<RwLock<Option<ThreadGo>>>,

    // This is all rootmoves for all treads.
    all_moves: AllRootMoves,

    // Join handle for the main thread.
    main_thread: Option<JoinHandle<()>>,

    // The mainthread will send us information through this! Such as
    // the best move available.
    receiver: Receiver<SendData>,

    // CondVar that the mainthread blocks on. We will notif the main thread
    // to awaken through this.
    main_thread_go: Arc<(Mutex<bool>,Condvar)>,

    // Vector of all non-main threads
    threads: Vec<JoinHandle<()>>,

    // Tells all threads to go. This is mostly used by the MainThread, we
    // don't really touch this at all.
    all_thread_go: Arc<(Mutex<bool>,Condvar)>,

    // For each thread (including the mainthread), is it finished?
    thread_finished: Vec<Arc<AtomicBool>>,

    // Tells all threads to stop and return the ebstmove found
    stop: Arc<AtomicBool>,

    // Tells the threads to drop.
    drop: Arc<AtomicBool>,
}

// Okay, this all looks like madness, but there is some reason to it all.
// Basically, Threadpool manages spawning and despawning threads, as well
// as passing state to / from those threads, telling them to stop, go, drop,
// and lastly determining the "best move" from all the threads.

// While we spawn all the other threads, We mostly communicate with the
// MainThread to do anything useful. We let the mainthread handle anything fun.
// The goal of the ThreadPool is to be NON BLOCKING, unless we want to await a
// result.

impl ThreadPool {
    pub fn setup(num_threads: usize, use_stdout: bool) -> Self {
        // Turn back, ye wary traveler!
        // The road ahead is perilous and unreadable!
        // Arrrrrrrggggg!!!!
        // (Or should I say Aaaaaaaarrrcccc!!!?)
        // (Get it?)
        // (Cause this code is littered with Arc's.
        // (Ha. ha.)

        let pos_state: Arc<RwLock<Option<ThreadGo>>> = Arc::new(RwLock::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let drop = Arc::new(AtomicBool::new(false));
        let use_stdout = Arc::new(AtomicBool::new(use_stdout));
        let main_thread_go = Arc::new((Mutex::new(false), Condvar::new()));
        let all_thread_go = Arc::new((Mutex::new(false), Condvar::new()));
        let (tx, rx) = channel();

        let mut threads = Vec::new();
        let mut all_threads_finished = Vec::new();

        let mut all_moves = Vec::with_capacity(num_threads);
        let mut depth_completed = Vec::with_capacity(num_threads);

        let main_thread_moves = Arc::new(RwLock::new(Vec::new()));
        all_moves.push(Arc::clone(&main_thread_moves));

        let main_thread_depth = Arc::new(AtomicU16::new(0));
        depth_completed.push(Arc::clone(&main_thread_depth));

        for x in 1..num_threads {
            let builder = thread::Builder::new();
            let thread_moves = Arc::new(RwLock::new(Vec::new()));
            let thread_depth_completed = Arc::new(AtomicU16::new(0));
            let thread_fin = Arc::new(AtomicBool::new(true));
            all_moves.push(Arc::clone(&thread_moves));
            depth_completed.push(Arc::clone(&thread_depth_completed));
            let new_thread =
                Thread::new(thread_moves,
                            thread_depth_completed,
                            x,
                            Arc::clone(&use_stdout),
                            Arc::clone(&stop),
                            Arc::clone(&thread_fin),
                            Arc::clone(&drop),
                            Arc::clone(&pos_state),
                            Arc::clone(&all_thread_go)
            );
            let join_handle = builder.spawn(move || {
                let mut current_thread = new_thread;
                current_thread.idle_loop()
            }).unwrap();
            all_threads_finished.push(thread_fin);
            threads.push(join_handle);
        }

        let all_root_moves = Arc::new(RwLock::new(all_moves));
        let main_thread_fin = Arc::new(AtomicBool::new(true));
        let main_thread_inner = Thread::new
            (main_thread_moves,
             Arc::clone(&main_thread_depth),
             0,
             Arc::clone(&use_stdout),
             Arc::clone(&stop),
             Arc::clone(&main_thread_fin),
             Arc::clone(&drop),
             Arc::clone(&pos_state),
             Arc::clone(&all_thread_go));

        let builder = thread::Builder::new();
        let main_thread = MainThread {
            all_moves: Arc::clone(&all_root_moves),
            all_finished: all_threads_finished.clone(),
            all_depths: depth_completed,
            main_thread_go: Arc::clone(&main_thread_go),
            sender: tx,
            thread: main_thread_inner };

        all_threads_finished.push(main_thread_fin);

        let join_handle = builder.spawn(move || {
            let mut main_thread = main_thread;
            main_thread.main_idle_loop()
        }).unwrap();

        ThreadPool {
            pos_state,
            all_moves: all_root_moves,
            main_thread: Some(join_handle),
            receiver: rx,
            main_thread_go,
            threads,
            all_thread_go,
            thread_finished: all_threads_finished,
            stop,
            drop
        }
    }

    pub fn uci_search(&mut self, board: &Board, limits: &UCILimit) {
        {
            let mut thread_go = self.pos_state.write().unwrap();
            *thread_go = Some(ThreadGo {
                board: board.shallow_clone(),
                limit: limits.clone()
            });
        }
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.main_thread_go));
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }
    }

    pub fn search(&mut self, board: &Board, limits: &UCILimit) -> BitMove {
        self.uci_search(&board, &limits);
        let data = self.receiver.recv().unwrap();
        match data {
            SendData::BestMove(t) => t.bit_move
        }
    }

    pub fn get_move(&self) -> BitMove {
        let data = self.receiver.recv().unwrap();
        match data {
            SendData::BestMove(t) => t.bit_move
        }
    }

    pub fn stop_searching(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub struct MainThread {
    all_moves: AllRootMoves,
    all_finished: Vec<Arc<AtomicBool>>,
    all_depths: Vec<Arc<AtomicU16>>,
    main_thread_go: Arc<(Mutex<bool>,Condvar)>,
    sender: Sender<SendData>,
    thread: Thread,
}

impl MainThread {
    pub fn main_idle_loop(&mut self) {
        while(!self.thread.drop()) {
            {
                let &(ref lock, ref cvar) = &*(Arc::clone(&self.main_thread_go));
                let mut started = lock.lock().unwrap();
                while !*started {
                    started = cvar.wait(started).unwrap();
                }
                if self.thread.drop() {
                    return;
                }
            }
            self.go();
        }
    }

    pub fn set_all_root_moves(&mut self, board: &Board) {
        let base_moves: Vec<RootMove> = board.generate_moves()
            .iter()
            .map(|b| RootMove::new(*b))
            .collect();
        let all_moves_lock = self.all_moves.write().unwrap();
        let all_moves = &*all_moves_lock;

        for lock in all_moves.iter() {
            let mut moves_lock = lock.write().unwrap();
            (*moves_lock) = base_moves.clone();
        }
    }

    pub fn set_all_depth_counts(&mut self) {
        self.all_depths.iter_mut().for_each(|d| d.store(0, Ordering::Relaxed));
    }

    pub fn lock_threads(&mut self) {
        let &(ref lock, ref _cvar) = &*(Arc::clone(&self.thread.cond));
        let mut started = lock.lock().unwrap();
        *started = false;
    }

    pub fn start_threads(&mut self) {
        let &(ref lock, ref cvar) = &*(Arc::clone(&self.thread.cond));
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_all();
    }

    pub fn thread_best_move(&mut self, thread: usize) -> RootMove {
        let all_thread_moves: &Vec<RootMoves> = &*self.all_moves.read().unwrap();
        let thread_moves: Arc<RwLock<Vec<RootMove>>> = Arc::clone(&all_thread_moves.get(thread).unwrap());
        let lock = thread_moves.read().unwrap();
        let moves: &Vec<RootMove> = (*lock).as_ref();
        moves.get(0).unwrap().clone()
    }

    pub fn lock_self(&mut self) {
        let &(ref lock, ref cvar) = &*(Arc::clone(&self.main_thread_go));
        let mut started = lock.lock().unwrap();
        *started = false;
    }

    pub fn wait_for_finish(&self) {
        for finished in self.all_finished.iter() {
            while(!finished.load(Ordering::Relaxed)) {}
        }
    }

    pub fn wait_for_start(&self) {
        for started in self.all_finished.iter() {
            while(started.load(Ordering::Relaxed)) {}
        }
    }

    pub fn num_threads(&self) -> usize {
        self.all_moves.read().unwrap().len()
    }

    pub fn go(&mut self) {
        self.thread.finished.store(false, Ordering::Relaxed);
        self.thread.stop.store(true, Ordering::Relaxed);
        self.set_all_depth_counts();
        let board = self.thread.retrieve_board().unwrap();
        self.set_all_root_moves(&board);

        // turn stop searching off
        self.thread.stop.store(false, Ordering::Relaxed);
        // wakeup all threads
        self.start_threads();

        let limit = self.thread.retrieve_limit().unwrap();
        self.wait_for_start();
        self.lock_threads();

        // start searching
        self.thread.start_searching(board, limit);
        self.thread.finished.store(true, Ordering::Relaxed);
        self.thread.stop.store(true, Ordering::Relaxed);
        self.wait_for_finish();

        // find best move
        let mut best_root_move: RootMove = self.thread_best_move(0);
        let mut depth_reached: i32 = self.all_depths[0].load(Ordering::Relaxed) as i32;
        if self.thread.use_stdout.load(Ordering::Relaxed) {
            println!("id: 0, value: {}, prev_value: {}, depth: {}, depth_comp: {}, mov: {}", best_root_move.score, best_root_move.prev_score, best_root_move.depth_reached, depth_reached, best_root_move.bit_move);
        }


        for x in 1..self.num_threads() {
            let thread_move = self.thread_best_move(x);
            let thread_depth = self.all_depths[x].load(Ordering::Relaxed);
            let depth_diff = thread_depth as i32 - depth_reached;
            let value_diff = thread_move.score - best_root_move.score;


            if self.thread.use_stdout.load(Ordering::Relaxed) {
                println!("id: {}, value: {}, prev_value: {}, depth: {}, depth_comp: {}, mov: {}",x, thread_move.score, best_root_move.prev_score, thread_move.depth_reached,thread_depth, thread_move.bit_move);
            }
            // If it has a bigger value and greater or equal depth
            if value_diff > 0 && depth_diff >= 0 {
                best_root_move = thread_move;
                depth_reached = thread_depth as i32;
            }
        }

        self.sender.send(SendData::BestMove(best_root_move)).unwrap();

        if self.thread.use_stdout.load(Ordering::Relaxed) {
            println!("bestmove {}", best_root_move.bit_move);
        }

        // return to idle loop
        self.lock_self();
    }
}

pub struct Thread {
    pub root_moves: RootMoves,
    pub depth_completed: Arc<AtomicU16>,
    pub id: usize,
    pub tt: &'static TT,
    pub use_stdout: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub finished: Arc<AtomicBool>,
    pub drop: Arc<AtomicBool>,
    pub pos_state: Arc<RwLock<Option<ThreadGo>>>,
    pub cond: Arc<(Mutex<bool>,Condvar)>,
    pub thread_stack: [ThreadStack; THREAD_STACK_SIZE],
}

impl Thread {
    pub fn new(root_moves: Arc<RwLock<Vec<RootMove>>>,
               depth_completed: Arc<AtomicU16>,
               id: usize,
               use_stdout: Arc<AtomicBool>,
               stop: Arc<AtomicBool>,
               finished: Arc<AtomicBool>,
               drop: Arc<AtomicBool>,
               pos_state: Arc<RwLock<Option<ThreadGo>>>,
               cond: Arc<(Mutex<bool>,Condvar)>, ) -> Self {
        Thread {
            root_moves,
            depth_completed,
            id,
            tt: &TT_TABLE,
            use_stdout,
            stop,
            finished,
            drop,
            pos_state,
            cond,
            thread_stack:
            init_thread_stack()
        }
    }

    pub fn drop(&self) -> bool {
        self.drop.load(Ordering::Relaxed)
    }

    pub fn retrieve_board(&self) -> Option<Board> {
        let s: &Option<ThreadGo> = &*(self.pos_state.read().unwrap());
        let board = s.as_ref().map(|ref tg| (*tg).board.shallow_clone());
        board
    }

    pub fn retrieve_limit(&self) -> Option<UCILimit> {
        let s: &Option<ThreadGo> = &*(self.pos_state.read().unwrap());
        let board = s.as_ref().map(|ref tg| (*tg).limit.clone());
        board
    }

    pub fn idle_loop(&mut self) {
        while !self.drop(){
            {
                let &(ref lock, ref cvar) = &*(Arc::clone(&self.cond));
                let mut started = lock.lock().unwrap();
                while !*started {
                    started = cvar.wait(started).unwrap();
                }
                if self.drop() {
                    return;
                }
            }
            self.go();
        }
    }

    fn start_searching(&mut self, board: Board, limit: UCILimit) {
        let mut thread_search = ThreadSearcher {
            thread: self,
            limit: limit,
            board: board
        };
        thread_search.search_root();
    }

    pub fn go(&mut self) {
        self.finished.store(false, Ordering::Relaxed);
        let board = self.retrieve_board().unwrap();
        let limit = self.retrieve_limit().unwrap();
        self.start_searching(board, limit);
        self.finished.store(true, Ordering::Relaxed);
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Store that we are dropping
        self.drop.store(true, Ordering::Relaxed);
        thread::sleep(time::Duration::new(0,100));
        self.stop.store(true, Ordering::Relaxed);

        // Notify the main thread to wakeup and stop
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.main_thread_go));
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }

        // Notify the other threads to wakeup and stop
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.all_thread_go));
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }

        // Join all the handles
        while !self.threads.is_empty() {
            let thread_handle = self.threads.pop().unwrap();
            thread_handle.join().unwrap();
        }
        self.main_thread.take().unwrap().join().unwrap();
    }
}

pub fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}


#[cfg(test)]
mod tests {
    use super::*;

//    #[test]
    pub fn test_searcher() {
        let mut pool = ThreadPool::setup(1, true);
        let mut board = Board::default();
        let limits = UCILimit::Depth(3);
        pool.search(&board, &limits);
        thread::sleep_ms(3000);
        let moves = pool.all_moves.clone();
        let r = moves.read().unwrap();
        for i in 0..(*r).len() {
            println!("thread {}", i);
            let m: Arc<RwLock<Vec<RootMove>>> = (*r).get(i).unwrap().clone();
            let inner = m.read().unwrap();
            for mov in (*inner).iter() {
                println!("Move: {} score: {} prev_score {} depth {}", mov.bit_move, mov.score, mov.prev_score, mov.depth_reached);
            }
        }

        pool.stop_searching();
        let mov = pool.get_move();
        println!("Bestmove {}", mov);
    }


}
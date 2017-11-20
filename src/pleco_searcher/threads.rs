use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::thread::{JoinHandle,self};
use std::sync::mpsc::{channel,Receiver,Sender};

use std::cmp::{min,max};
use std::{mem,time};
use std::time::Duration;

use board::*;
use core::*;
use board::eval::*;
use core::piece_move::BitMove;
use tools::tt::*;
use engine::*;

use super::thread_search::ThreadSearcher;
use super::misc::*;
use super::{LIMIT,TT_TABLE,THREAD_STACK_SIZE,MAX_PLY};

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
    pos_state: Arc<RwLock<Option<ThreadGo>>>,

    all_moves: AllRootMoves,
    main_thread: Option<JoinHandle<()>>,

    receiver: Receiver<SendData>,
    main_thread_go: Arc<(Mutex<bool>,Condvar)>,

    threads: Vec<JoinHandle<()>>,
    all_thread_go: Arc<(Mutex<bool>,Condvar)>,
    thread_finished: Vec<Arc<AtomicBool>>,
    stop: Arc<AtomicBool>,
    drop: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn setup(num_threads: usize, use_stdout: bool) -> Self {
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
        let main_thread_moves = Arc::new(RwLock::new(Vec::new()));
        all_moves.push(Arc::clone(&main_thread_moves));
        for x in 1..num_threads {
            let builder = thread::Builder::new();
            let thread_moves = Arc::new(RwLock::new(Vec::new()));
            let thread_fin = Arc::new(AtomicBool::new(false));
            all_moves.push(Arc::clone(&thread_moves));
            let new_thread =
                Thread::new(thread_moves, x,
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
        let main_thread_fin = Arc::new(AtomicBool::new(false));
        let main_thread_inner = Thread::new
            (main_thread_moves,
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
            all_stops: all_threads_finished.clone(),
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
    all_stops: Vec<Arc<AtomicBool>>,
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
        for stop in self.all_stops.iter() {
            while(!stop.load(Ordering::Relaxed)) {}
        }
    }

    pub fn num_threads(&self) -> usize {
        self.all_moves.read().unwrap().len()
    }

    pub fn go(&mut self) {
        self.thread.finished.store(false, Ordering::Relaxed);
        self.thread.stop.store(true, Ordering::Relaxed);
        let board = self.thread.retrieve_board().unwrap();
        self.set_all_root_moves(&board);

        self.thread.stop.store(false, Ordering::Relaxed);
        // wakeup all threads
        self.start_threads();

        let limit = self.thread.retrieve_limit().unwrap();
        thread::sleep_ms(1);
        self.lock_threads();

        // start searching
        self.thread.start_searching(board, limit);
        self.thread.finished.store(true, Ordering::Relaxed);
        self.wait_for_finish();

        // find best move
        let mut best_root_move: RootMove = self.thread_best_move(0);
        println!("id: 0, value: {}, depth: {}, mov: {}", best_root_move.score, best_root_move.depth_reached, best_root_move.bit_move);

        for x in 1..self.num_threads() {
            let thread_move = self.thread_best_move(x);
            let depth_diff = thread_move.depth_reached as i16 - best_root_move.depth_reached as i16;
            let value_diff = thread_move.score as i16 - best_root_move.score as i16;


            if self.thread.use_stdout.load(Ordering::Relaxed) {
                println!("id: {}, value: {}, depth: {}, mov: {}",x, thread_move.score, thread_move.depth_reached, thread_move.bit_move);
            }
            // If it has a bigger value and greater or equal depth
            if value_diff > 0 && depth_diff >= 0 {
                best_root_move = thread_move;
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
               id: usize,
               use_stdout: Arc<AtomicBool>,
               stop: Arc<AtomicBool>,
               finished: Arc<AtomicBool>,
               drop: Arc<AtomicBool>,
               pos_state: Arc<RwLock<Option<ThreadGo>>>,
               cond: Arc<(Mutex<bool>,Condvar)>, ) -> Self {
        Thread {
            root_moves,
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
        while(!self.drop()){
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
        self.stop.store(true, Ordering::Relaxed);
        self.drop.store(true, Ordering::Relaxed);

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
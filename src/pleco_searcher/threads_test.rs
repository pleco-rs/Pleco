use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::thread::{JoinHandle,self};

use std::cmp::{min,max};
use std::mem;

use board::*;
use core::*;
use board::eval::*;
use core::piece_move::BitMove;
use tools::tt::*;
use engine::*;

use super::misc::*;
use super::{LIMIT,TT_TABLE,THREAD_STACK_SIZE,MAX_PLY};

pub struct ThreadGo {
    limit: UCILimit,
    board: Board,
    // Options: ?
}

pub type RootMoves = Arc<RwLock<Vec<RootMove>>>;
pub type AllRootMoves = Arc<RwLock<Vec<RootMoves>>>;

pub struct ThreadPool {
    pos_state: Arc<RwLock<Option<ThreadGo>>>,

    all_moves: AllRootMoves,
    main_thread: JoinHandle<()>,

    main_thread_go: Arc<(Mutex<bool>,Condvar)>,
    threads: Vec<JoinHandle<()>>,
    all_thread_go: Arc<(Mutex<bool>,Condvar)>,
    stop: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn setup(num_threads: usize) -> Self {
        let pos_state: Arc<RwLock<Option<ThreadGo>>> = Arc::new(RwLock::new(None));
        let stop= Arc::new(AtomicBool::new(false));
        let main_thread_go = Arc::new((Mutex::new(false), Condvar::new()));
        let all_thread_go = Arc::new((Mutex::new(false), Condvar::new()));

        let mut threads = Vec::new();
        let mut all_moves = Vec::with_capacity(num_threads);
        let main_thread_moves = Arc::new(RwLock::new(Vec::new()));

        all_moves.push(Arc::clone(&main_thread_moves));
        for x in 1..num_threads {
            let builder = thread::Builder::new();
            let thread_moves = Arc::new(RwLock::new(Vec::new()));
            all_moves.push(Arc::clone(&thread_moves));
            let new_thread =
                Thread::new(thread_moves, x, Arc::clone(&stop), Arc::clone(&pos_state), Arc::clone(&all_thread_go)
            );
            let join_handle = builder.spawn(move || {
                let mut current_thread = new_thread;
                current_thread.idle_loop()
            }).unwrap();
            threads.push(join_handle);
        }

        let all_root_moves = Arc::new(RwLock::new(all_moves));

        let main_thread_inner = Thread::new
            (main_thread_moves,0,Arc::clone(&stop), Arc::clone(&pos_state), Arc::clone(&all_thread_go));

        let builder = thread::Builder::new();
        let main_thread = MainThread { all_moves: Arc::clone(&all_root_moves), main_thread_go: Arc::clone(&main_thread_go), thread: main_thread_inner };

        let join_handle = builder.spawn(move || {
            let mut main_thread = main_thread;
            main_thread.main_idle_loop()
        }).unwrap();

        ThreadPool {
            pos_state,
            all_moves: all_root_moves,
            main_thread: join_handle,
            main_thread_go,
            threads,
            all_thread_go,
            stop
        }
    }
}

pub struct MainThread {
    all_moves: AllRootMoves,
    main_thread_go: Arc<(Mutex<bool>,Condvar)>,
    thread: Thread,
}

impl MainThread {
    pub fn main_idle_loop(&mut self) {
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.main_thread_go));
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
            self.go()
        }

    }

    pub fn go(&mut self) {

    }
}

pub struct Thread {
    pub root_moves: RootMoves,
    pub id: usize,
    pub tt: &'static TT,
    pub stop: Arc<AtomicBool>,
    pos_state: Arc<RwLock<Option<ThreadGo>>>,
    cond: Arc<(Mutex<bool>,Condvar)>,
    pub thread_stack: [ThreadStack; THREAD_STACK_SIZE],
}

impl Thread {
    pub fn new(root_moves: Arc<RwLock<Vec<RootMove>>>,
               id: usize,
               stop: Arc<AtomicBool>,
               pos_state: Arc<RwLock<Option<ThreadGo>>>,
               cond: Arc<(Mutex<bool>,Condvar)>, ) -> Self {
        Thread {
            root_moves,
            id,
            tt: &TT_TABLE,
            stop,
            pos_state,
            cond,
            thread_stack:
            init_thread_stack()
        }
    }

    pub fn idle_loop(&mut self) {
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.cond));
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
    }

    pub fn go(&mut self) {

    }
}

pub struct ThreadSearcher<'a> {
    thread: &'a Thread,
    limit: UCILimit,
    board: Board
}

pub fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}
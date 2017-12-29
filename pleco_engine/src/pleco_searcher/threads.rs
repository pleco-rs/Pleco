//! Contains the ThreadPool and the individual Threads.

use std::sync::{Arc,RwLock};
use std::sync::atomic::{AtomicBool,Ordering};
use std::thread::{JoinHandle,self};
use std::sync::mpsc::{channel,Receiver,Sender};

use std::{mem,time};

use pleco::board::*;
use pleco::core::piece_move::BitMove;
use pleco::tools::tt::*;


use super::search::ThreadSearcher;
use super::misc::*;
use super::{TT_TABLE,THREAD_STACK_SIZE};
use super::root_moves::RootMove;
use super::root_moves::root_moves_list::RootMoveList;
use super::root_moves::root_moves_manager::RmManager;
use super::sync::LockLatch;


pub struct ThreadGo {
    limit: Limits,
    board: Board,
    // Options: ?
}

pub enum SendData {
    BestMove(RootMove)
}

pub struct ThreadPool {
    // This is the position information we send to each thread upon
    // starting. Contains stuff like the Board, and the Limit to search
    // to.
    pos_state: Arc<RwLock<Option<ThreadGo>>>,

    // This is all rootmoves for all treads.
    rm_manager: RmManager,



    // Join handle for the main thread.
    main_thread: Option<JoinHandle<()>>,

    // The mainthread will send us information through this! Such as
    // the best move available.
    receiver: Receiver<SendData>,

    // CondVar that the mainthread blocks on. We will notif the main thread
    // to awaken through this.
    main_thread_go: Arc<LockLatch>,

    // Vector of all non-main threads
    threads: Vec<JoinHandle<()>>,

    // Tells all threads to go. This is mostly used by the MainThread, we
    // don't really touch this at all.
    all_thread_go: Arc<LockLatch>,

    // For each thread (including the mainthread), is it finished?

    // Tells all threads to stop and return the ebstmove found
    stop: Arc<AtomicBool>,

    // Tells the threads to drop.
    drop: Arc<AtomicBool>,

    use_stdout: Arc<AtomicBool>,

    stop_guard: Arc<AtomicBool>
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
    fn init(rx: Receiver<SendData>) -> Self {
        ThreadPool {
            pos_state: Arc::new(RwLock::new(None)),
            rm_manager: RmManager::new(),
            main_thread: None,
            receiver: rx,
            main_thread_go: Arc::new(LockLatch::new()),
            threads: Vec::with_capacity(8),
            all_thread_go: Arc::new(LockLatch::new()),
            stop: Arc::new(AtomicBool::new(false)),
            drop: Arc::new(AtomicBool::new(false)),
            use_stdout: Arc::new(AtomicBool::new(false)),
            stop_guard: Arc::new(AtomicBool::new(true))
        }
    }

    fn create_thread(&self, id: usize, root_moves: RootMoveList) -> Thread {
        Thread {
            root_moves: root_moves,
            id: id,
            tt: &super::TT_TABLE,
            use_stdout: Arc::clone(&self.use_stdout),
            stop: Arc::clone(&self.stop),
            drop: Arc::clone(&self.drop),
            pos_state: Arc::clone(&self.pos_state),
            cond: Arc::clone(&self.all_thread_go),
            thread_stack: init_thread_stack(),
        }
    }

    fn spawn_main_thread(&mut self, tx: Sender<SendData>) {
        let root_moves = self.rm_manager.add_thread().unwrap();
        let thread = self.create_thread(0, root_moves);
        let main_thread = MainThread {
            per_thread: self.rm_manager.clone(),
            main_thread_go: Arc::clone(&self.main_thread_go),
            sender: tx,
            thread,
            stop_guard: self.stop_guard.clone()
        };


        let builder = thread::Builder::new().name(String::from("0"));
        self.main_thread = Some(
            builder.spawn(move || {
                let mut main_thread = main_thread;
                main_thread.main_idle_loop()
            }).unwrap());
    }

    pub fn new() -> Self {
        let (tx, rx) = channel();
        let mut pool = ThreadPool::init(rx);
        pool.spawn_main_thread(tx);
        pool
    }

    pub fn stdout(&mut self, use_stdout: bool) {
        self.use_stdout.store(use_stdout, Ordering::Relaxed)
    }

    pub fn set_thread_count(&mut self, num: usize) {
        // TODO: Check for overflow
        let curr_num: usize = self.rm_manager.size();

        let mut i: usize = curr_num;
        while i < num {
            let root_moves = self.rm_manager.add_thread().unwrap();
            let thread = self.create_thread(i, root_moves);
            let builder = thread::Builder::new().name(i.to_string());
            self.threads.push(builder.spawn(move || {
                let mut current_thread = thread;
                current_thread.idle_loop()
            }).unwrap());
            i += 1;
        }
    }

    pub fn uci_search(&mut self, board: &Board, limits: &PreLimits) {
        self.stop_guard.store(false, Ordering::SeqCst);
        {
            let mut thread_go = self.pos_state.write().unwrap();
            *thread_go = Some(ThreadGo {
                board: board.shallow_clone(),
                limit: (limits.clone()).create()
            });
        }
        self.main_thread_go.set();
        while !self.stop_guard.load(Ordering::SeqCst) {}
    }

    pub fn search(&mut self, board: &Board, limits: &PreLimits) -> BitMove {
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
    per_thread: RmManager,
    main_thread_go: Arc<LockLatch>,
    sender: Sender<SendData>,
    thread: Thread,
    stop_guard: Arc<AtomicBool>
}

impl MainThread {
    pub fn main_idle_loop(&mut self) {
        while !self.thread.drop() {
            self.main_thread_go.wait();
            if self.thread.drop() {
                return;
            }
            self.go();
        }
    }

    pub fn lock_threads(&mut self) {
        self.thread.cond.lock();
    }

    pub fn start_threads(&mut self) {
        self.thread.cond.set();
    }

    pub fn lock_self(&mut self) {
        self.main_thread_go.lock();
    }

    pub fn go(&mut self) {
        self.thread.root_moves.set_finished(true);
        self.thread.stop.store(true, Ordering::Relaxed);
        self.per_thread.wait_for_finish();
        self.per_thread.reset_depths();
        let board = self.thread.retrieve_board().unwrap();
        unsafe {
            self.per_thread.replace_moves(&board);
        }

        // turn stop searching off
        self.thread.stop.store(false, Ordering::Relaxed);
        // wakeup all threads
        self.start_threads();

        let limit = self.thread.retrieve_limit().unwrap();
        self.thread.root_moves.set_finished(false);
        self.per_thread.wait_for_start();
        self.lock_threads();

        // start searching
        self.stop_guard.store(true, Ordering::SeqCst);
        self.thread.start_searching(board, limit);
        self.thread.root_moves.set_finished(true);
        self.thread.stop.store(true, Ordering::Relaxed);
        self.per_thread.wait_for_finish();

        // find best move
        let best_root_move: RootMove = self.per_thread.best_rootmove(self.thread.use_stdout.load(Ordering::Relaxed));

        self.sender.send(SendData::BestMove(best_root_move)).unwrap();

        if self.thread.use_stdout.load(Ordering::Relaxed) {
            println!("bestmove {}", best_root_move.bit_move);
        }

        // return to idle loop
        self.lock_self();
    }
}

pub struct Thread {
    pub root_moves: RootMoveList,
    pub id: usize,
    pub tt: &'static TranspositionTable,
    pub use_stdout: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub drop: Arc<AtomicBool>,
    pub pos_state: Arc<RwLock<Option<ThreadGo>>>,
    pub cond: Arc<LockLatch>,
    pub thread_stack: [ThreadStack; THREAD_STACK_SIZE],
}

impl Thread {

    pub fn drop(&self) -> bool {
        self.drop.load(Ordering::Relaxed)
    }

    pub fn retrieve_board(&self) -> Option<Board> {
        let s: &Option<ThreadGo> = &*(self.pos_state.read().unwrap());
        let board = s.as_ref().map(|ref tg| (*tg).board.shallow_clone());
        board
    }

    pub fn retrieve_limit(&self) -> Option<Limits> {
        let s: &Option<ThreadGo> = &*(self.pos_state.read().unwrap());
        let board = s.as_ref().map(|ref tg| (*tg).limit.clone());
        board
    }

    pub fn idle_loop(&mut self) {
        self.root_moves.set_finished(true);
        while !self.drop(){
            self.cond.wait();
            if self.drop() {
                return;
            }
            self.root_moves.set_finished(false);
            self.go();
            self.root_moves.set_finished(true);
        }
    }

    fn start_searching(&mut self, board: Board, limit: Limits) {
        let mut thread_search = ThreadSearcher {
            thread: self,
            limit: limit,
            board: board
        };
        thread_search.search_root();
    }

    pub fn go(&mut self) {
        let board = self.retrieve_board().unwrap();
        let limit = self.retrieve_limit().unwrap();
        self.start_searching(board, limit);
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Store that we are dropping
        self.drop.store(true, Ordering::Relaxed);
        thread::sleep(time::Duration::new(0,100));
        self.stop.store(true, Ordering::Relaxed);

        // Notify the main thread to wakeup and stop
        self.main_thread_go.set();

        // Notify the other threads to wakeup and stop
        self.all_thread_go.set();

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

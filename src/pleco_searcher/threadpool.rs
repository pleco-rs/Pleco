use num_cpus;

use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::thread::{JoinHandle,self};
use std::cmp::max;


use super::misc::*;
use super::search::Thread;
use super::LIMIT;

use board::*;
use core::piece_move::BitMove;
use engine::*;

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
const THREAD_DIST: usize = 20;

pub struct ThreadPool {
    gui_stop: Arc<AtomicBool>,
    cond_var: Arc<(Mutex<bool>,Condvar)>,
    all_moves: Vec<Arc<RwLock<Vec<RootMove>>>>,
    threads: Vec<JoinHandle<()>>,
    main_thread: Thread,
}

impl ThreadPool {
    pub fn setup(board: Board) -> (ThreadPool, Arc<AtomicBool>) {
        let stop = Arc::new(AtomicBool::new(false));

        let num_threads = max(num_cpus::get(),1);

        let nodes = Arc::new(AtomicU64::new(0));
        let cond_var = Arc::new((Mutex::new(false), Condvar::new()));

        let root_moves: Vec<RootMove> = board.generate_moves().into_iter().map( RootMove::new).collect();


        let mut all_moves = Vec::with_capacity(num_threads);
        let mut threads = Vec::with_capacity(num_threads);

        let main_thread_moves = Arc::new(RwLock::new(root_moves.clone()));
        all_moves.push(Arc::clone(&main_thread_moves)); // index 0, aka the main thread

        for x in 1..num_threads {
            let builder = thread::Builder::new();
            let shared_moves = Arc::new(RwLock::new(root_moves.clone()));
            all_moves.push(Arc::clone(&shared_moves));

            let new_thread = Thread::new(&board, Arc::clone(&shared_moves), x, &nodes, &stop, &cond_var);

            let join_handle = builder.spawn(move || {
                let mut current_thread = new_thread;
                current_thread.idle_loop()
            }).unwrap();

            threads.push(join_handle);
        }

        let main_thread = Thread::new(&board, main_thread_moves, 0, &nodes, &stop, &cond_var);
        (ThreadPool {
            gui_stop: Arc::clone(&stop),
            cond_var: cond_var,
            all_moves: all_moves,
            threads: threads,
            main_thread: main_thread, },
         stop)
    }


    pub fn start_searching(&mut self, limit: UCILimit, use_stdout: bool) -> BitMove {
        // Make sure there is no stop command
        assert!(!(self.gui_stop.load(Ordering::Relaxed)));

        // Check if Moves is empty
        {
            if self.main_thread.root_moves.read().unwrap().is_empty() {
                return BitMove::null();
            }
        }

        // Set the global limit
        unsafe { LIMIT = limit; }

        // get cond_var and notify the threads to wake up
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.cond_var));
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }

        // Main thread needs to start searching
        self.main_thread.thread_search();

        // Make sure the remaining threads have finished.
        while !self.threads.is_empty() {
            self.threads.pop().unwrap().join().unwrap();
        }

        let mut best_root_move: RootMove = { *self.main_thread.root_moves.read().unwrap().get(0).unwrap() };

        // Find out if there is a better found move
        for thread_moves in &self.all_moves {
            let root_move: RootMove = *thread_moves.read().unwrap().get(0).unwrap();
            let depth_diff = root_move.depth_reached as i16 - best_root_move.depth_reached as i16;
            let value_diff = root_move.score as i16 - best_root_move.score as i16;

            // If it has a bigger value and greater or equal depth
            if value_diff > 0 && depth_diff >= 0 {
                best_root_move = root_move;
            }
        }

        if use_stdout {
            println!("bestmove {}", best_root_move.bit_move);
        }

        best_root_move.bit_move
    }

    pub fn perft(depth: u16) -> u64 {
        unimplemented!()
    }
}


impl Drop for ThreadPool {
    fn drop(&mut self) {
        while !self.threads.is_empty() {
            let thread_handle = self.threads.pop().unwrap();
            thread_handle.join().unwrap();
        }
    }
}
//! Contains the ThreadPool and the individual Threads.

use std::heap::{Alloc, Layout, Heap};
use std::sync::atomic::{AtomicBool,Ordering};
use std::thread::{JoinHandle,self};
use std::sync::{Once, ONCE_INIT};
use std::ptr::NonNull;
use std::ptr;
use std::cell::UnsafeCell;

use crossbeam_utils::scoped;

use pleco::MoveList;
use pleco::tools::pleco_arc::Arc;
use pleco::board::*;
use pleco::core::piece_move::BitMove;

use sync::LockLatch;
use time::uci_timer::*;
use time::time_management::TimeManager;
use search::Searcher;

use consts::*;

pub static mut THREADPOOL: NonNull<ThreadPool> = unsafe {NonNull::new_unchecked(ptr::null_mut())};

static THREADPOOL_INIT: Once = ONCE_INIT;

pub fn init_threadpool() {
    THREADPOOL_INIT.call_once(|| {
        unsafe {
            let layout = Layout::new::<ThreadPool>();
            let result = Heap.alloc_zeroed(layout);
            let new_ptr: *mut ThreadPool = match result {
                Ok(ptr) => ptr as *mut ThreadPool,
                Err(err) => Heap.oom(err),
            };
            ptr::write(new_ptr, ThreadPool::new());
            THREADPOOL = NonNull::new_unchecked(new_ptr);
        }
    });
}

/// Returns access to the global thread pool.z
pub fn threadpool() -> &'static mut ThreadPool {
    unsafe {
        THREADPOOL.as_mut()
    }
}

/// Global Timer
lazy_static! {
    pub static ref TIMER: TimeManager = TimeManager::uninitialized();
}

struct SearcherPtr {
    ptr: UnsafeCell<*mut Searcher>
}

unsafe impl Sync for SearcherPtr {}
unsafe impl Send for SearcherPtr {}

/// The thread-pool for the chess engine
pub struct ThreadPool {
    /// Access to each thread's Structure
    pub threads: Vec<UnsafeCell<*mut Searcher>>,
    /// Handles of each thread
    handles: Vec<JoinHandle<()>>,
    /// Condition for the main thread to start.
    pub main_cond: Arc<LockLatch>,
    /// Condition for all non-main threads
    pub thread_cond: Arc<LockLatch>,
    /// Stop condition, if true the threads should halt.
    pub stop: AtomicBool
}

// Okay, this all looks like madness, but there is some reason to it all.
// Basically, `ThreadPool` manages spawning and despawning threads, as well
// as passing state to / from those threads, telling them to stop, go, drop,
// and lastly determining the "best move" from all the threads.
///
// While we spawn all the other threads, We mostly communicate with the
// MainThread to do anything useful. We let the mainthread handle anything fun.
// The goal of the ThreadPool is to be NON BLOCKING, unless we want to await a
// result.
impl ThreadPool {

    /// Creates a new `ThreadPool`
    pub fn new() -> Self {
        let mut pool: ThreadPool = ThreadPool {
            threads: Vec::new(),
            handles: Vec::new(),
            main_cond: Arc::new(LockLatch::new()),
            thread_cond: Arc::new(LockLatch::new()),
            stop: AtomicBool::new(true)
        };
        // Lock both the cond variables
        pool.main_cond.lock();
        pool.thread_cond.lock();

        // spawn the main thread
        pool.attach_thread();
        pool
    }

    /// Spawns a new thread and appends it to our vector of JoinHandles.
    fn attach_thread(&mut self) {
         unsafe {
             let thread_ptr: SearcherPtr = self.create_thread();
             let builder = thread::Builder::new()
                 .name(self.size().to_string());

             let handle = scoped::builder_spawn_unsafe(builder,
                move || {
                    let thread = &mut **thread_ptr.ptr.get();
                    thread.cond.lock();
                    thread.idle_loop();
             }).unwrap();
             self.handles.push(handle);
        };
    }

    /// Returns a pointer to the main thread.
    fn main(&mut self) -> &mut Searcher {
        unsafe {
            let main_thread: *mut Searcher = *self.threads.get_unchecked(0).get();
            return &mut *main_thread;
        }
    }

    /// Returns the number of threads
    #[inline(always)]
    fn size(&self) -> usize {
        self.threads.len()
    }

    /// Allocates a thread structure and pushes it to the threadstack.
    fn create_thread(&mut self) -> SearcherPtr {
        let len: usize = self.threads.len();
        let layout = Layout::new::<Searcher>();
        let cond = if len == 0 {self.main_cond.clone()} else {self.thread_cond.clone()};
        unsafe {
            let result = Heap.alloc_zeroed(layout);
            let new_ptr: *mut Searcher = match result {
                Ok(ptr) => ptr as *mut Searcher,
                Err(err) => Heap.oom(err),
            };
            ptr::write(new_ptr, Searcher::new(len, cond));
            self.threads.push(UnsafeCell::new(new_ptr));
            SearcherPtr {ptr: UnsafeCell::new(new_ptr)}
        }
    }


    /// Sets the use of standard out. This can be changed mid search as well.
    #[inline(always)]
    pub fn stdout(&mut self, use_stdout: bool) {
        USE_STDOUT.store(use_stdout, Ordering::Relaxed);
    }

    /// Sets the thread count of the pool. If num is less than 1, nothing will happen.
    ///
    /// # Safety
    ///
    /// Completely unsafe to use when the pool is searching.
    pub fn set_thread_count(&mut self, mut num: usize) {
        if num >= 1 {
            num = num.min(MAX_THREADS);
            self.wait_for_finish();
            self.kill_all();
            while self.size() < num {
                self.attach_thread();
            }
        }
    }

    /// Kills and de-allocates all the threads that are running. This function will also
    /// block on waiting for the search to finish.
    pub fn kill_all(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.wait_for_finish();
        let mut join_handles = Vec::with_capacity(self.size());
        unsafe {
            // tell each thread to drop
            self.threads.iter()
                .map(|s| &**s.get())
                .for_each(|s: &Searcher| {
                    s.kill.store(true, Ordering::SeqCst)
                });

            // If awaiting a signal, wake up each thread so each can drop
            self.threads.iter()
                .map(|s| &**s.get())
                .for_each(|s: &Searcher| {
                    s.cond.set();
                });


            // Start connecting each join handle. We don't unwrap here, as if one of the
            // threads fail, the other threads remain un-joined.
            while let Some(handle) = self.handles.pop() {
                join_handles.push(handle.join());
            }

            // De-allocate each thread.
            while let Some(unc) = self.threads.pop() {
                let th: *mut Searcher = *unc.get();
                let layout = Layout::new::<Searcher>();
                Heap.dealloc(th as *mut _, layout);
            }
        }

        // Unwrap the results from each `thread::join`,
        while let Some(handle_result) = join_handles.pop() {
            handle_result.unwrap_or_else(|e| println!("Thread failed: {:?}",e));
        }
    }

    /// Sets the threads to stop (or not!).
    #[inline(always)]
    pub fn set_stop(&mut self, stop: bool) {
        self.stop.store(stop, Ordering::Relaxed);
    }

    /// Waits for all the threads to finish
    pub fn wait_for_finish(&self) {
        unsafe {
            self.threads.iter()
                .map(|t| &**t.get())
                .for_each(|t| t.searching.await(false));
        }
    }

    /// Waits for all the threads to start.
    pub fn wait_for_start(&self) {
        unsafe {
            self.threads.iter()
                .map(|t| &**t.get())
                .for_each(|t| t.searching.await(true));
        }
    }

    /// Waits for all non-main threads to finish.
    pub fn wait_for_non_main(&self) {
        unsafe {
            self.threads.iter()
                .map(|s| &**s.get())
                .for_each(|t: &Searcher|{
                    if t.id != 0 {
                        t.searching.await(false);
                    }
            });
        }
    }

    /// Waits for all the non-main threads to start running
    pub fn wait_for_main_start(&self) {
        unsafe {
            self.threads.iter()
                .map(|s| &**s.get())
                .for_each(|t: &Searcher|{
                    if t.id == 0 {
                        t.searching.await(true);
                    }
                });
        }
    }

    pub fn clear_all(&mut self) {
        for thread_ptr in self.threads.iter_mut() {
            let mut thread: &mut Searcher = unsafe { &mut **(*thread_ptr).get() };
            thread.clear();
        }
    }

    /// Starts a UCI search. The result will be printed to stdout if the stdout setting
    /// is true.
    pub fn uci_search(&mut self, board: &Board, limits: &Limits) {
        let root_moves: MoveList = board.generate_moves();

        assert!(!root_moves.is_empty());
        self.wait_for_finish();
        self.stop.store(false, Ordering::Relaxed);

        for thread_ptr in self.threads.iter_mut() {
            let mut thread: &mut Searcher = unsafe {&mut **(*thread_ptr).get()};
            thread.depth_completed = 0;
            thread.board = board.shallow_clone();
            thread.limit = limits.clone();
            thread.root_moves().replace(&root_moves);
        }

        self.main_cond.set();
        self.wait_for_main_start();
        self.main_cond.lock();
    }


    /// Performs a standard search, and blocks waiting for a returned `BitMove`.
    pub fn search(&mut self, board: &Board, limits: &Limits) -> BitMove {
        self.uci_search(board, limits);
        self.wait_for_finish();
        self.best_move()
    }

    /// Returns the best move of a search
    pub fn best_move(&mut self) -> BitMove {
        self.main().root_moves().get(0).unwrap().bit_move
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.kill_all();
    }
}


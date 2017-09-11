use std::ptr::{Unique, self};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::heap::{Alloc, Layout, Heap};
use std::sync::atomic::{AtomicUsize,AtomicU16};

use piece_move::BitMove;

pub type Key = u64;

// 2 bytes + 2 bytes + 1 Byte + 1 byte = ?6 Bytes
#[derive(Clone)]
pub struct Entry {
    pub partial_key: u16,
    pub best_move: BitMove, // What was the best move found here?
    pub score: i16, // What was the Score of this node?
    pub eval: i16, // What is the evaluation of this node
    pub depth: u8, // How deep was this Score Found?
    pub time_node_bound: NodeTypeTimeBound,
}

impl Entry {
    pub fn place(&mut self, key: Key, best_move: BitMove, score: i16, eval: i16, depth: u8, node_type: NodeType, time_bound: u8) {
        let partial_key = key.wrapping_shr(48) as u16;

        if partial_key != self.partial_key {
            self.best_move = best_move;
        }

        if partial_key != self.partial_key || node_type == NodeType::Exact {
            self.partial_key = partial_key;
            self.score = score;
            self.eval = eval;
            self.depth = depth;
            self.time_node_bound = NodeTypeTimeBound::create(node_type, time_bound);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NodeTypeTimeBound {
    data: u8
}

impl NodeTypeTimeBound {
    pub fn create(node_type: NodeType, time_bound: u8) -> Self {
        NodeTypeTimeBound {
            data: time_bound + (node_type as u8)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum NodeType {
    NoBound = 0,
    LowerBound = 1,
    UpperBound = 2,
    Exact = 3,
}


pub const CLUSTER_SIZE: usize = 3;


pub struct Cluster {
    pub entry: [Entry; CLUSTER_SIZE],
    pub padding: [u8; 2],
}

pub struct TT {
    clusters: Unique<Cluster>,
    cap: usize,
    time_age: u8,
}



impl TT {
    pub fn new_round_up(size: usize) -> Self {
        TT::new(size.next_power_of_two())
    }

    pub fn new(size: usize) -> Self {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        TT {
            clusters: alloc_room(size),
            cap: size,
            time_age: 0,
        }
        
    }

    pub fn resize_round_up(self, size: usize) {
        self.resize(size.next_power_of_two());
    }

    pub fn resize(&self, size: usize) {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        self.de_alloc();
        self.re_alloc(size);
    }

    pub fn clear(&self) {
        self.resize(self.cap);
    }

    fn re_alloc(&self, size: usize) {
        unsafe {
            let clust_ptr: *mut Unique<Cluster> = mem::transmute::<&Unique<Cluster>,*mut Unique<Cluster>>(&self.clusters);
            *clust_ptr = alloc_room(size);
        }
    }

    pub fn new_search(&mut self) {
        self.time_age += 8;
    }

    pub fn time_age(&self) -> u8 {
        self.time_age
    }

    pub fn probe(&self, key: Key) -> (bool, *mut Entry) {
        unimplemented!()
    }

    pub fn first_entry(key: Key) -> *mut Entry {
        unimplemented!()
    }

    fn de_alloc(&self) {
        unsafe {
            Heap.dealloc(self.clusters.as_ptr() as *mut _,
                         Layout::array::<Cluster>(self.cap).unwrap());
        }
    }
}

impl Drop for TT {
    fn drop(&mut self) {
        self.de_alloc();
    }
}

fn alloc_room(size: usize) -> Unique<Cluster> {
    unsafe {
        let ptr = Heap.alloc_zeroed(Layout::array::<Cluster>(size).unwrap());

        let new_ptr = match ptr {
            Ok(ptr) => ptr,
            Err(err) => Heap.oom(err),
        };
        Unique::new(new_ptr as *mut Cluster).unwrap()
    }

}


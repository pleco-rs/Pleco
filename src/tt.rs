use std::ptr::{Unique, self};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::heap::{Alloc, Layout, Heap};
use std::sync::atomic::{AtomicUsize,AtomicU16};

use piece_move::BitMove;

pub type Key = u64;

//
//

// 2 bytes + 2 bytes + 2 Byte + 2 byte + 1 + 1 = 10 Bytes
#[derive(Clone,PartialEq)]
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

    pub fn time(&self) -> u8 {
        self.time_node_bound.data & TIME_MASK
    }


    pub fn node_type(&self) -> NodeType {
        match self.time_node_bound.data & NODE_TYPE_MASK {
            0 => NodeType::NoBound,
            1 => NodeType::LowerBound,
            2 => NodeType::UpperBound,
            _ => NodeType::Exact,
        }
    }

    pub fn time_value(&self, curr_time: u8) -> u8 {
        let inner: u8 = (259 as u8).wrapping_add((curr_time).wrapping_sub(self.time_node_bound.data)) & 0b1111_1100;
        (self.depth).wrapping_sub((inner).wrapping_mul(2 as u8))
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NodeTypeTimeBound {
    data: u8
}

pub const TIME_MASK: u8 = 0b1111_1100;
pub const NODE_TYPE_MASK: u8 = 0b0000_0011;

impl NodeTypeTimeBound {
    pub fn create(node_type: NodeType, time_bound: u8) -> Self {
        NodeTypeTimeBound {
            data: time_bound + (node_type as u8)
        }
    }

    pub fn update_time(&mut self, time_bound: u8) {
        self.data = (self.data & NODE_TYPE_MASK) | time_bound;
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

// 30 bytes + 2 = 32 Bytes
pub struct Cluster {
    pub entry: [Entry; CLUSTER_SIZE],
    pub padding: [u8; 2],
}

// clusters -> Pointer to the clusters
// cap -> n number of clusters (So n * CLUSTER_SIZE) number of entries
// time age -> documenting when an entry was placed
pub struct TT {
    clusters: Unique<Cluster>,
    cap: usize,
    time_age: u8,
}



impl TT {

    // Creates new TT rounded up in size
    pub fn new_round_up(size: usize) -> Self {
        TT::new(size.next_power_of_two())
    }

    // Creates new TT
    fn new(size: usize) -> Self {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        TT {
            clusters: alloc_room(size),
            cap: size,
            time_age: 0,
        }

    }

    pub fn num_clusters(&self) -> usize {
        self.cap
    }

    // Resizes and deletes all data
    pub fn resize_round_up(mut self, size: usize) {
        self.resize(size.next_power_of_two());
    }

    //
    fn resize(&mut self, size: usize) {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        self.de_alloc();
        self.re_alloc(size);
    }

    // clears the entire tt
    pub fn clear(&mut self) {
        let size = self.cap;
        self.resize(size);
    }

    // Called each time a new position is searched
    pub fn new_search(&mut self) {
        self.time_age = (self.time_age).wrapping_add(8);
    }

    // the current time age
    pub fn time_age(&self) -> u8 {
        self.time_age
    }

    // returns (true, entry) is the key is found
    // if not, returns (false, entry) where the entry is the least valuable entry;
    pub fn probe(&self, key: Key) -> (bool, &mut Entry) {
        let partial_key: u16 = (key).wrapping_shr(48) as u16;

        unsafe {
            let cluster: *mut Cluster = self.cluster(key);
            let init_entry: *mut Entry = cluster_first_entry(cluster);

            // for each entry
            for i in 0..CLUSTER_SIZE {
                // get a pointer to the specified entry
                let entry_ptr: *mut Entry = init_entry.offset(i as isize);
                // convert to &mut
                let entry: &mut Entry = &mut (*entry_ptr);

                // found a spot
                if entry.partial_key == 0 || entry.partial_key == partial_key {

                    // if age is incorrect, make it correct
                    if entry.time() != self.time_age && entry.partial_key != 0 {
                        entry.time_node_bound.update_time(self.time_age);
                    }

                    // Return the spot
                    return (true, entry);
                }
            }

            let mut replacement: *mut Entry = init_entry;
            let mut replacement_score: u8 = (&*replacement).time_value(self.time_age);
            // gotta find a replacement
            for i in 1..CLUSTER_SIZE {
                let entry_ptr: *mut Entry = init_entry.offset(i as isize);
                let entry_score: u8 = (&*entry_ptr).time_value(self.time_age);
                if entry_score < replacement_score {
                    replacement = entry_ptr;
                    replacement_score = replacement_score;
                }
            }
            // return the best place to replace
            (false, &mut (*replacement))
        }
    }

    // returns the cluster for a given key
    pub fn cluster(&self, key: Key) -> *mut Cluster {
        let index: usize = ((self.num_clusters() - 1) as u64 & key) as usize;
        unsafe {
            self.clusters.as_ptr().offset(index as isize)
        }
    }

    fn re_alloc(&mut self, size: usize) {
        unsafe {
            // let clust_ptr: *mut Unique<Cluster> = mem::transmute::<&Unique<Cluster>,*mut Unique<Cluster>>(&self.clusters.);
//            *clust_ptr = alloc_room(size);
            self.clusters = alloc_room(size);
        }
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


#[inline]
unsafe fn cluster_first_entry(cluster: *mut Cluster) -> *mut Entry {
    mem::transmute::<*mut Cluster,*mut Entry>(cluster)
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

#[cfg(test)]
mod tests {

    extern crate rand;
    use tt::*;
    use std::ptr::null;


    // around 0.5 GB
    const HALF_GIG: usize = 2 << 24;
    // around 30 MB
    const THIRTY_MB: usize = 2 << 20;


    #[test]
    fn tt_alloc_realloc() {
        let size: usize = 8;
        let tt = TT::new(size);
        assert_eq!(tt.num_clusters(), size);

        let key = create_key(32, 44);

        let (found,entry) = tt.probe(key);
    }

    #[test]
    fn tt_null_ptr() {
        let size: usize = 2 << 20;
        println!("hello");
        let mut tt = TT::new_round_up(size);

        for x  in 0..1_000_000 as u64 {
            let key: u64 = rand::random::<u64>();
            {
                let (found, entry) = tt.probe(key);
                entry.depth = (x % 0b1111_1111) as u8;
                entry.partial_key = key.wrapping_shr(48) as u16;
                assert_ne!((entry as * const _), null());
            }
            tt.new_search();
        }

    }

    fn create_key(partial_key: u16, full_key: u64) -> u64 {
        (partial_key as u64).wrapping_shl(48) | (full_key & 0x0000_FFFF_FFFF_FFFF)
    }

}


//! Module for the TranspositionTable, a type of hashmap where Zobrist Keys map to information about a position.
//!
//!
//! A Transposition Table is a structure to quickly lookup chess positions and determine information from them.
//! It maps from Board positions to Information such as the evaluation of that position, the best move found so far,
//! the depth that move was found at, etc.

use std::ptr::Unique;
use std::mem;
use std::heap::{Alloc, Layout, Heap};
use core::piece_move::BitMove;


/// Value used to retrieve and store Entries.
pub type Key = u64;

/// BitMask for the [NodeTypeTimeBound]'s time data.
pub const TIME_MASK: u8 = 0b1111_1100;

/// BitMask for the retrieving a [NodeTypeTimeBound]'s [NodeType].
pub const NODE_TYPE_MASK: u8 = 0b0000_0011;

/// Number of Entries per Cluster.
pub const CLUSTER_SIZE: usize = 3;

const BYTES_PER_KB: usize = 1000;
const BYTES_PER_MB: usize = BYTES_PER_KB * 1000;
const BYTES_PER_GB: usize = BYTES_PER_MB * 1000;

/// Designates the type of Node in the Chess Search tree.
/// See the [ChessWiki](https://chessprogramming.wikispaces.com/Node+Types) for more information
/// about PV Node types and their use.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum NodeBound {
    NoBound = 0,
    LowerBound = 1,
    UpperBound = 2,
    Exact = 3,
}

/// Abstraction for combining the 'time' a node was found alongside the `NodeType`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NodeTypeTimeBound {
    data: u8
}

impl NodeTypeTimeBound {
    /// Creates a NodeTypeTimeBound with the designated node_type and time.
    ///
    /// # Usage
    ///
    /// time_bound must be divisible by 8 or else Undefined behavior will follow.
    pub fn create(node_type: NodeBound, time_bound: u8) -> Self {
        NodeTypeTimeBound {
            data: time_bound + (node_type as u8)
        }
    }

    /// Updates the [NodeType] of an entry.
    pub fn update_bound(&mut self, node_type: NodeBound) {
        self.data = (self.data & TIME_MASK) | node_type as u8;
    }

    /// Updates the time field of an entry.
    pub fn update_time(&mut self, time_bound: u8) {
        self.data = (self.data & NODE_TYPE_MASK) | time_bound;
    }
}



// 2 bytes + 2 bytes + 2 Byte + 2 byte + 1 + 1 = 10 Bytes

/// Structure defining a singular Entry in a table, containing the `BestMove` found,
/// the score of that node, the type of Node, depth found, as well as a key uniquely defining
/// the node.
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

    pub fn is_empty(&self) -> bool {
        self.node_type() == NodeBound::NoBound
    }

    /// Rewrites over an Entry.
    pub fn place(&mut self, key: Key, best_move: BitMove, score: i16, eval: i16, depth: u8, node_type: NodeBound) {
        let partial_key = key.wrapping_shr(48) as u16;

        if partial_key != self.partial_key {
            self.best_move = best_move;
        }

        if partial_key != self.partial_key || node_type == NodeBound::Exact {
            self.partial_key = partial_key;
            self.score = score;
            self.eval = eval;
            self.depth = depth;
            self.time_node_bound.update_bound(node_type);
        }
    }

    /// Returns the current search time of the node.
    pub fn time(&self) -> u8 {
        self.time_node_bound.data & TIME_MASK
    }

    /// Returns the [NodeType] of an Entry.
    pub fn node_type(&self) -> NodeBound {
        match self.time_node_bound.data & NODE_TYPE_MASK {
            0 => NodeBound::NoBound,
            1 => NodeBound::LowerBound,
            2 => NodeBound::UpperBound,
            _ => NodeBound::Exact,
        }
    }

    /// Returns the value of the node in respect to the depth searched && when it was placed into the TranspositionTable.
    pub fn time_value(&self, curr_time: u8) -> u16 {
        let inner: u16 = ((259u16).wrapping_add(curr_time as u16)).wrapping_sub(self.time_node_bound.data as u16) & 0b1111_1100;
        (self.depth as u16).wrapping_sub((inner).wrapping_mul(2 as u16))
    }
}

// 30 bytes + 2 = 32 Bytes
/// Structure containing multiple Entries all mapped to by the same zobrist key.
pub struct Cluster {
    pub entry: [Entry; CLUSTER_SIZE],
    pub padding: [u8; 2],
}

// clusters -> Pointer to the clusters
// cap -> n number of clusters (So n * CLUSTER_SIZE) number of entries
// time age -> documenting when an entry was placed
/// Structure for representing a `TranspositionTable`. A Transposition Table is a type
/// of HashTable that maps Zobrist Keys to information about that position, including the best move
/// found, score, depth the move was found at, and other information.
pub struct TT {
    clusters: Unique<Cluster>, // pointer to the heap
    cap: usize, // number of clusters
    time_age: u8, // documenting at whichh root position an entry was placed
}

impl TT {

    /// Creates new with a size of around 'mb_size'. Actual size is the nearest power
    /// of 2 times the size of a Cluster.
    ///
    /// # Panics
    ///
    /// mb_size should be > 0, or else a panic will occur
    pub fn new(mb_size: usize) -> Self {
        TT::new_num_clusters((mb_size * BYTES_PER_MB) / mem::size_of::<Cluster>())
    }

    /// Creates new TT rounded up to the nearest power of two number of entries.
    ///
    /// # Panics
    ///
    /// num_entries should be > 0, or else a panic will occur
    pub fn new_num_entries(num_entries: usize) -> Self {
        TT::new_num_clusters(num_entries * CLUSTER_SIZE)
    }

    /// Creates new TT rounded up to the nearest power of two number of Clusters.
    ///
    /// # Panics
    ///
    /// Size should be > 0, or else a panic will occur
    pub fn new_num_clusters(num_clusters: usize) -> Self {
        TT::create(num_clusters.next_power_of_two())
    }

    // Creates new TT with the number of Clusters being size. size must be a power of two.
    fn create(size: usize) -> Self {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        TT {
            clusters: alloc_room(size),
            cap: size,
            time_age: 0,
        }
    }

    /// Returns the size of the heap allocated portion of the TT in KiloBytes.
    pub fn size_kilobytes(&self) -> usize {
        (mem::size_of::<Cluster>() * self.cap) / BYTES_PER_KB
    }

    /// Returns the size of the heap allocated portion of the TT in MegaBytes.
    pub fn size_megabytes(&self) -> usize {
        (mem::size_of::<Cluster>() * self.cap) / BYTES_PER_MB
    }

    /// Returns the size of the heap allocated portion of the TT in GigaBytes.
    pub fn size_gigabytes(&self) -> usize {
        (mem::size_of::<Cluster>() * self.cap) / BYTES_PER_GB
    }

    /// Returns the number of clusters the Transposition Table holds.
    pub fn num_clusters(&self) -> usize {
        self.cap
    }

    /// Returns the number of Entries the Transposition Table holds.
    pub fn num_entries(&self) -> usize {self.cap * CLUSTER_SIZE}

    /// Re-sizes to 'size' number of Clusters and deletes all data
    ///
    /// # Panic
    ///
    /// size must be greater then 0
    pub fn resize_round_up(&mut self, size: usize) {
        self.resize(size.next_power_of_two());
    }

    /// Re-sizes to the the mb_size number of megabytes, rounded up for power of 2
    /// number of clusters. Returns the actual size.
    ///
    /// # Panic
    ///
    /// mb_size must be greater then 0
    pub fn resize_to_megabytes(&mut self, mb_size: usize) -> usize {
        let mut num_clusters: usize = (mb_size * BYTES_PER_MB) / mem::size_of::<Cluster>();
        num_clusters = num_clusters.next_power_of_two();
        self.resize(num_clusters);
        self.size_megabytes()
    }

    // resizes the tt to a certain type
    fn resize(&mut self, size: usize) {
        assert_eq!(size.count_ones(), 1);
        assert!(size > 0);
        self.de_alloc();
        self.re_alloc(size);
    }

    /// Clears the entire TranspositionTable
    pub fn clear(&mut self) {
        let size = self.cap;
        self.resize(size);
    }

    // Called each time a new position is searched
    pub fn new_search(&mut self) {
        self.time_age = (self.time_age).wrapping_add(4);
    }

    /// Returns the current time age of a TT.
    pub fn time_age(&self) -> u8 {
        self.time_age
    }

    /// Returns the current number of cycles a TT has gone through. Cycles is simply the
    /// number of times refresh has been called.
    pub fn time_age_cylces(&self) -> u8 {
        (self.time_age).wrapping_shr(2)
    }

    /// Probes the Transposition Table for a specified Key. Returns (true, entry) if either (1) an
    /// Entry corresponding to the current key is found, or an Open Entry slot is found for the key.
    /// In the case of an open Entry, the entry can be tested for its contents by using [entry.is_empty()].
    /// If no entry is found && there are no open entries, returns the entry that is is most irrelevent to
    /// the current search, e.g. has the shallowest depth or was found in a previous search.
    ///
    /// If 'true' is returned, the Entry is guaranteed to have the correct time.
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
            let mut replacement_score: u16 = (&*replacement).time_value(self.time_age);

            // Table is full, find the best replacement based on depth and time placed there
            for i in 1..CLUSTER_SIZE {
                let entry_ptr: *mut Entry = init_entry.offset(i as isize);
                let entry_score: u16 = (&*entry_ptr).time_value(self.time_age);
                if entry_score < replacement_score {
                    replacement = entry_ptr;
                    replacement_score = replacement_score;
                }
            }
            // return the best place to replace
            (false, &mut (*replacement))
        }
    }

    /// Returns the cluster of a given key.
    fn cluster(&self, key: Key) -> *mut Cluster {
        let index: usize = ((self.num_clusters() - 1) as u64 & key) as usize;
        unsafe {
            self.clusters.as_ptr().offset(index as isize)
        }
    }

    // Re-Allocates the current TT to a specified size.
    fn re_alloc(&mut self, size: usize) {
        self.clusters = alloc_room(size);
    }

    /// de-allocates the current heap.
    fn de_alloc(&self) {
        unsafe {
            Heap.dealloc(self.clusters.as_ptr() as *mut _,
                         Layout::array::<Cluster>(self.cap).unwrap());
        }
    }

    // TODO: Add Hash % generation.

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

// Return a Heap Allocation of Size number of Clusters.
#[inline]
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
        let tt = TT::create(size);
        assert_eq!(tt.num_clusters(), size);

        let key = create_key(32, 44);
        let (_found,_entry) = tt.probe(key);
    }

    #[test]
    fn tt_test_sizes() {
        let tt = TT::new_num_clusters(100);
        assert_eq!(tt.num_clusters(), (100 as usize).next_power_of_two());
        assert_eq!(tt.num_entries(), (100 as usize).next_power_of_two() * CLUSTER_SIZE);
    }

    #[test]
    fn tt_null_ptr() {
        let size: usize = 2 << 20;
        let mut tt = TT::new_num_clusters(size);

        for x  in 0..1_000_000 as u64 {
            let key: u64 = rand::random::<u64>();
            {
                let (_found, entry) = tt.probe(key);
                entry.depth = (x % 0b1111_1111) as u8;
                entry.partial_key = key.wrapping_shr(48) as u16;
                assert_ne!((entry as * const _), null());
            }
            tt.new_search();
        }
    }

    #[test]
    fn tt_basic_insert() {
        let tt = TT::new_num_clusters(THIRTY_MB);
        let partial_key_1: u16 = 17773;
        let key_index: u64 = 0x5556;

        let key_1 = create_key(partial_key_1, 0x5556);
        let (found, entry) = tt.probe(key_1);
        assert!(found);
        entry.partial_key = partial_key_1;
        entry.depth = 2;

        let (found, entry) = tt.probe(key_1);
        assert!(found);
        assert!(entry.is_empty());
        assert_eq!(entry.partial_key,partial_key_1);
        assert_eq!(entry.depth,2);

        let partial_key_2: u16 = 8091;
        let partial_key_3: u16 = 12;
        let key_2: u64 = create_key(partial_key_2, key_index);
        let key_3: u64 = create_key(partial_key_3, key_index);

        let (found, entry) = tt.probe(key_2);
        assert!(found);
        assert!(entry.is_empty());
        entry.partial_key = partial_key_2;
        entry.depth = 3;

        let (found, entry) = tt.probe(key_3);
        assert!(found);
        assert!(entry.is_empty());
        entry.partial_key = partial_key_3;
        entry.depth = 6;

        // key that should find a good replacement
        let partial_key_4: u16 = 18;
        let key_4: u64 = create_key(partial_key_4, key_index);

        let (found, entry) = tt.probe(key_4);
        assert!(!found);

        // most vulnerable should be key_1
        assert_eq!(entry.partial_key, partial_key_1);
        assert_eq!(entry.depth, 2);
    }

    /// Helper function to create a key of specified index / partial_key
    fn create_key(partial_key: u16, full_key: u64) -> u64 {
        (partial_key as u64).wrapping_shl(48) | (full_key & 0x0000_FFFF_FFFF_FFFF)
    }
}


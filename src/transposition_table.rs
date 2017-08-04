use piece_move::*;


pub type Key = u64;



// 2 bytes + 2 bytes + 1 Byte + 1 byte = ?6 Bytes
#[derive(Clone)]
pub struct Value {
    pub best_move: BitMove, // What was the best move found here?
    pub score: i16, // What was the Evlauation of this Node?
    pub depth: u16, // How deep was this Score Found?
    pub node_type: NodeType,
}


#[derive(Clone)]
pub enum NodeType {
    UpperBound,
    LowerBound,
    Exact
}

use owning_ref::{OwningHandle, OwningRef};
use parking_lot::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::collections::hash_map;
use std::hash::{Hash, Hasher, BuildHasher};
use std::sync::atomic::{self, AtomicUsize};
use std::{mem, ops, cmp, fmt, iter};

/// The atomic ordering used throughout the code.
const ORDERING: atomic::Ordering = atomic::Ordering::Relaxed;
/// The length-to-capacity factor.
const LENGTH_MULTIPLIER: usize = 4;
/// The maximal load factor's numerator.
const MAX_LOAD_FACTOR_NUM: usize = 100 - 15;
/// The maximal load factor's denominator.
const MAX_LOAD_FACTOR_DENOM: usize = 100;
/// The default initial capacity.
const DEFAULT_INITIAL_CAPACITY: usize = 64;
/// The lowest capacity a table can have.
const MINIMUM_CAPACITY: usize = 8;


/// A bucket state.
///
/// Buckets are the bricks of hash tables. They represent a single entry into the table.
#[derive(Clone)]
enum Bucket {
    Contains(u64, Value),
    Empty,
    Removed,
}

impl Bucket {
    /// Is this bucket 'empty'?
    fn is_empty(&self) -> bool {
        if let Bucket::Empty = *self { true } else { false }
    }


    fn is_removed(&self) -> bool {
        if let Bucket::Removed = *self { true } else { false }
    }

    fn is_free(&self) -> bool {
        match *self {
            // The two replacable bucket types are removed buckets and empty buckets.
            Bucket::Removed | Bucket::Empty => true,
            // KV pairs can't be replaced as they contain data.
            Bucket::Contains(..) => false,
        }
    }

    fn value(self) -> Option<Value> {
        if let Bucket::Contains(_, val) = self {
            Some(val)
        } else { None }
    }


    fn value_ref(&self) -> Result<&Value, ()> {
        if let Bucket::Contains(_, ref val) = *self {
            Ok(val)
        } else {
            Err(())
        }
    }

    fn key_matches(&self, key: &u64) -> bool {
        if let Bucket::Contains(ref candidate_key, _) = *self {
            // Check if the keys matches.
            candidate_key == key
        } else {
            // The bucket isn't a KV pair, so we'll return false, since there is no key to test
            // against.
            false
        }
    }
}

struct Table {
    buckets: Vec<RwLock<Bucket>>
}


impl Table {
    fn new(buckets: usize) -> Self {
        let mut vec = Vec::with_capacity(buckets);
        for _ in 0..buckets {
            vec.push(RwLock::new(Bucket::Empty));
        }
        Table { buckets: vec}
    }

    fn with_capacity(cap: usize) -> Table {
        Table::new(cmp::max(MINIMUM_CAPACITY, cap * LENGTH_MULTIPLIER))
    }

}

pub struct IntoIter {
    table: Table,
}

impl Iterator for IntoIter {
    type Item = (Key,Value);

    fn next(&mut self) -> Option<(Key, Value)> {

        while let Some(bucket) = self.table.buckets.pop() {
            if let Bucket::Contains(key, val) = bucket.into_inner() {
                return Some((key, val));
            }
        }
        None
    }

}

impl IntoIterator for Table {
    type Item = (Key, Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            table: self
        }
    }
}

pub struct ReadGuard<'a> {
    /// The inner hecking long type.
    inner: OwningRef<OwningHandle<RwLockReadGuard<'a, Table>, RwLockReadGuard<'a, Bucket>>, Value>,
}

impl<'a> ops::Deref for ReadGuard<'a> {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.inner
    }
}

impl<'a> cmp::PartialEq for ReadGuard<'a> {
    fn eq(&self, other: &ReadGuard<'a>) -> bool {
        self == other
    }
}
impl<'a> cmp::Eq for ReadGuard<'a> {}

impl<'a> fmt::Debug for ReadGuard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReadGuard({:?})", self)
    }
}

/// A mutable RAII guard for reading an entry of a hash map.
///
/// This is an access type dereferencing to the inner value of the entry. It will handle unlocking
/// on drop.
pub struct WriteGuard<'a> {
    /// The inner hecking long type.
    inner: OwningHandle<OwningHandle<RwLockReadGuard<'a, Table>, RwLockWriteGuard<'a, Bucket>>, &'a mut Value>,
}

impl<'a> ops::Deref for WriteGuard<'a> {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.inner
    }
}

impl<'a> ops::DerefMut for WriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.inner
    }
}

impl<'a> cmp::PartialEq for WriteGuard<'a> {
    fn eq(&self, other: &WriteGuard<'a>) -> bool {
        self == other
    }
}
impl<'a> cmp::Eq for WriteGuard<'a> {}

impl<'a> fmt::Debug for WriteGuard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WriteGuard({:?})", self)
    }
}

pub struct TranspositionTable {
    table: RwLock<Table>,
    len: AtomicUsize
}

impl TranspositionTable {
    pub fn with_capacity(cap: usize) -> Self {
        TranspositionTable {
            table:  RwLock::new(Table::with_capacity(cap)),
            len: AtomicUsize::new(0)
        }
    }

    pub fn capacity(&self) -> usize {
        self.buckets() * MAX_LOAD_FACTOR_NUM / MAX_LOAD_FACTOR_DENOM
    }

    pub fn buckets(&self) -> usize {
        self.table.read().buckets.len()
    }



    pub fn clear(&self) -> TranspositionTable {
//        // Acquire a writable lock.
//        let mut lock = self.table.write();
//
//        CHashMap {
//            table: RwLock::new(mem::replace(&mut *lock, Table::new(DEFAULT_INITIAL_CAPACITY))),
//            // Replace the length with 0 and use the old length.
//            len: AtomicUsize::new(self.len.swap(0, ORDERING)),
//        }
        unimplemented!()
    }



}
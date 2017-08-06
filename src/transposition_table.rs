use piece_move::*;


pub type Key = u64;



// 2 bytes + 2 bytes + 1 Byte + 1 byte = ?6 Bytes
#[derive(Clone)]
pub struct Value {
    pub best_move: BitMove, // What was the best move found here?
    pub score: i16, // What was the Evlauation of this Node?
    pub ply: u16, // How deep was this Score Found?
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

    fn key_matches(&self, key: u64) -> bool {
        if let Bucket::Contains(candidate_key, _) = *self {
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

    fn hash(&self, key: u64) -> usize {
       key as usize
    }

    fn scan<F>(&self, key: u64, matches: F) -> RwLockReadGuard<Bucket>
        where F: Fn(&Bucket) -> bool {
        // Hash the key.
        let hash = self.hash(key);

        // Start at the first priority bucket, and then move upwards, searching for the matching
        // bucket.
        for i in 0.. {
            // Get the lock of the `i`'th bucket after the first priority bucket (wrap on end).
            let lock = self.buckets[(hash + i) % self.buckets.len()].read();

            // Check if it is a match.
            if matches(&lock) {
                // Yup. Return.
                return lock;
            }
        }

        // TODO
        unreachable!();
    }

    fn scan_mut<F>(&self, key: u64, matches: F) -> RwLockWriteGuard<Bucket>
        where F: Fn(&Bucket) -> bool {
        // Hash the key.
        let hash = self.hash(key);

        // Start at the first priority bucket, and then move upwards, searching for the matching
        // bucket.
        for i in 0.. {
            // Get the lock of the `i`'th bucket after the first priority bucket (wrap on end).
            let lock = self.buckets[(hash + i) % self.buckets.len()].write();

            // Check if it is a match.
            if matches(&lock) {
                // Yup. Return.
                return lock;
            }
        }

        // TODO
        unreachable!();
    }

    fn scan_mut_no_lock<F>(&mut self, key: u64, matches: F) -> &mut Bucket
        where F: Fn(&Bucket) -> bool {
        // Hash the key.
        let hash = self.hash(key);
        // TODO: To tame the borrowchecker, we fetch this in advance.
        let len = self.buckets.len();

        // Start at the first priority bucket, and then move upwards, searching for the matching
        // bucket.
        for i in 0.. {
            // TODO: hacky hacky
            let idx = (hash + i) % len;

            // Get the lock of the `i`'th bucket after the first priority bucket (wrap on end).

            // Check if it is a match.
            if {
                let bucket = self.buckets[idx].get_mut();
                matches(&bucket)
            } {
                // Yup. Return.
                return self.buckets[idx].get_mut();
            }
        }

        // TODO
        unreachable!();
    }

    fn lookup_or_free(&self, key: u64) -> RwLockWriteGuard<Bucket> {
        // Hash the key.
        let hash = self.hash(key);
        // The encountered free bucket.
        let mut free = None;

        // Start at the first priority bucket, and then move upwards, searching for the matching
        // bucket.
        for i in 0..self.buckets.len() {
            // Get the lock of the `i`'th bucket after the first priority bucket (wrap on end).
            let lock = self.buckets[(hash + i) % self.buckets.len()].write();

            if lock.key_matches(key) {
                // We found a match.
                return lock;
            } else if lock.is_empty() {
                // The cluster is over. Use the encountered free bucket, if any.
                return free.unwrap_or(lock);
            } else if lock.is_removed() && free.is_none() {
                // We found a free bucket, so we can store it to later (if we don't already have
                // one).
                free = Some(lock)
            }
        }

        free.expect("No free buckets found")
    }

    fn lookup(&self, key: u64) -> RwLockReadGuard<Bucket> {
        self.scan(key, |x| match *x {
            // We'll check that the keys does indeed match, as the chance of hash collisions
            // happening is inevitable
            Bucket::Contains(candidate_key, _) if key == candidate_key => true,
            // We reached an empty bucket, meaning that there are no more buckets, not even removed
            // ones, to search.
            Bucket::Empty => true,
            _ => false,
        })
    }

    fn lookup_mut(&self, key: u64) -> RwLockWriteGuard<Bucket> {
        self.scan_mut(key, |x| match *x {
            // We'll check that the keys does indeed match, as the chance of hash collisions
            // happening is inevitable
            Bucket::Contains(candidate_key, _) if key == candidate_key => true,
            // We reached an empty bucket, meaning that there are no more buckets, not even removed
            // ones, to search.
            Bucket::Empty => true,
            _ => false,
        })
    }

    fn fill(&mut self, table: Table) {
        // Run over all the buckets.
        for i in table.buckets {
            // We'll only transfer the bucket if it is a KV pair.
            if let Bucket::Contains(key, val) = i.into_inner() {
                // Find a bucket where the KV pair can be inserted.
                let mut bucket = self.scan_mut_no_lock(key, |x| match *x {
                    // Halt on an empty bucket.
                    Bucket::Empty => true,
                    // We'll assume that the rest of the buckets either contains other KV pairs (in
                    // particular, no buckets have been removed in the newly construct table).
                    _ => false,
                });

                // Set the bucket to the KV pair.
                *bucket = Bucket::Contains(key, val);
            }
        }
    }

    fn find_free(&self, key: u64) -> RwLockWriteGuard<Bucket> {
        self.scan_mut(key, |x| x.is_free())
    }

    /// Find a free bucket in the same cluster as some key (bypassing locks).
    ///
    /// This is similar to `find_free`, except that it safely bypasses locks through the aliasing
    /// guarantees of `&mut`.
    fn find_free_no_lock(&mut self, key: u64) -> &mut Bucket {
        self.scan_mut_no_lock(key, |x| x.is_free())
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

    pub fn get(&self, key: u64) -> Option<ReadGuard> {
        // Acquire the read lock and lookup in the table.
        if let Ok(inner) = OwningRef::new(OwningHandle::new(self.table.read(), |x| unsafe { &*x }.lookup(key)))
            .try_map(|x| x.value_ref()) {
            // The bucket contains data.
            Some(ReadGuard {
                inner: inner,
            })
        } else {
            // The bucket is empty/removed.
            None
        }
    }

    pub fn get_mut(&self, key: u64) -> Option<WriteGuard> {
        // Acquire the write lock and lookup in the table.
        if let Ok(inner) = OwningHandle::try_new(OwningHandle::new(self.table.read(), |x| unsafe { &*x }.lookup_mut(key)), |x| if let &mut Bucket::Contains(_, ref mut val) = unsafe { &mut *(x as *mut Bucket) } {
            // The bucket contains data.
            Ok(val)
        } else {
            // The bucket is empty/removed.
            Err(())
        }) {
            Some(WriteGuard {
                inner: inner,
            })
        } else { None }
    }

    pub fn clear(&self) -> TranspositionTable {
        // Acquire a writable lock.
        let mut lock = self.table.write();

        TranspositionTable {
            table: RwLock::new(mem::replace(&mut *lock, Table::new(DEFAULT_INITIAL_CAPACITY))),
            // Replace the length with 0 and use the old length.
            len: AtomicUsize::new(self.len.swap(0, ORDERING)),
        }
    }

    pub fn insert(&self, key: u64, val: Value) -> Option<Value> {
        let ret;
        // Expand and lock the table. We need to expand to ensure the bounds on the load factor.
        let lock = self.table.read();
        {
            // Lookup the key or a free bucket in the inner table.
            let mut bucket = lock.lookup_or_free(key);

            // Replace the bucket.
            ret = mem::replace(&mut *bucket, Bucket::Contains(key, val)).value();
        }

        // Expand the table if no bucket was overwritten (i.e. the entry is fresh).
        if ret.is_none() {
            self.expand(lock);
        }

        ret
    }

    pub fn len(&self) -> usize {
        self.len.load(ORDERING)
    }

    pub fn contains_key(&self, key: u64) -> bool {
        // Acquire the lock.
        let lock = self.table.read();
        // Look the key up in the table
        let bucket = lock.lookup(key);
        // Test if it is free or not.
        !bucket.is_free()

        // fuck im sleepy rn
    }

    fn expand(&self, lock: RwLockReadGuard<Table>) {
        // Increment the length to take the new element into account.
        let len = self.len.fetch_add(1, ORDERING) + 1;

        // Extend if necessary. We multiply by some constant to adjust our load factor.
        if len * MAX_LOAD_FACTOR_DENOM > lock.buckets.len() * MAX_LOAD_FACTOR_NUM {
            // Drop the read lock to avoid deadlocks when acquiring the write lock.
            drop(lock);
            // Reserve 1 entry in space (the function will handle the excessive space logic).
            self.reserve(1);
        }
    }

    pub fn reserve(&self, additional: usize) {
        // Get the new length.
        let len = self.len() + additional;
        // Acquire the write lock (needed because we'll mess with the table).
        let mut lock = self.table.write();
        // Handle the case where another thread has resized the table while we were acquiring the
        // lock.
        if lock.buckets.len() < len * LENGTH_MULTIPLIER {
            // Swap the table out with a new table of desired size (multiplied by some factor).
            let table = mem::replace(&mut *lock, Table::with_capacity(len));
            // Fill the new table with the data from the old table.
            lock.fill(table);
        }
    }

}
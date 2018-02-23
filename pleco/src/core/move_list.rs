//! Contains the `MoveList` & `ScoreMoveList` structures, akin to a `Vec<BitMove>` but faster for
//! our purposes.
//!
//! A [`MoveList`] structure is guaranteed to be exactly 512 bytes long, containing a maximum of 256
//! moves. This number was chosen as no possible chess position has been found to contain more than
//! 232 possible moves.
//!
//! This structure is intended to mainly be used for generation of moves for a certain position. If
//! you need to a more versatile collection of moves to manipulate, considering using a `Vec<BitMove>`
//! instead.
//!
//! The [`ScoreMoveList`] is practically the same as the [`MoveList`], but it allows for each move to
//! have a score attached to it as well.
//!
//! [`MoveList`]: struct.MoveList.html
//! [`ScoreMoveList`]: struct.MoveList.html

use super::piece_move::{BitMove,ScoringBitMove};

use std::slice;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator,FromIterator};


pub trait MVPushable: Sized + IndexMut<usize> + Index<usize> {

    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    fn push_mv(&mut self, mv: BitMove);

    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// Undefined behavior if pushing to the list when `MoveList::len() = 256`.
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove);

    /// Set the length of the list.
    ///
    /// # Safety
    ///
    /// Unsafe due to overwriting the length of the list
    unsafe fn unchecked_set_len(&mut self, len: usize);
}

const MAX_MOVES: usize = 256;

/// This is the list of possible moves for a current position. Think of it alike a faster
/// version of `Vec<BitMove>`, as all the data is stored in the Stack rather than the Heap.
pub struct MoveList {
    inner: [BitMove; 256],
    len: usize,
}

impl Default for MoveList {
    #[inline]
    fn default() -> Self {
        MoveList {
            inner: [BitMove::null(); 256],
            len: 0,
        }
    }
}

impl From<Vec<BitMove>> for MoveList {
    fn from(vec: Vec<BitMove>) -> Self {
        let mut list = MoveList::default();
        vec.iter().for_each(|m| list.push(*m));
        list
    }
}


impl From<ScoringMoveList> for MoveList {
    fn from(sc_list: ScoringMoveList) -> Self {
        let mut mv_list = MoveList::default();
        sc_list.iter().for_each(|m| mv_list.push(m.bitmove()));
        mv_list
    }
}

impl Into<Vec<BitMove>> for MoveList {
    #[inline]
    fn into(self) -> Vec<BitMove> {
        self.vec()
    }
}

impl MoveList {
    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    #[inline(always)]
    pub fn push(&mut self, mv: BitMove) {
        self.push_mv(mv);
    }

    /// Returns true if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// assert!(list.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a `Vec<BitMove>` from this `MoveList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    ///
    /// let vec: Vec<BitMove> = list.vec();
    /// ```
    pub fn vec(&self) -> Vec<BitMove> {
        let mut vec = Vec::with_capacity(self.len);
        for mov in self.iter() {
            vec.push(*mov);
        }
        assert_eq!(vec.len(),self.len);
        vec
    }

    /// Returns the number of moves inside the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    /// list.push(BitMove::null());
    /// list.push(BitMove::null());
    /// assert_eq!(list.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the `MoveList` as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    ///
    /// let slice: &[BitMove] = list.as_slice();
    /// ```
    #[inline(always)]
    pub fn as_slice(&self) -> &[BitMove] {
        self
    }
}


impl Deref for MoveList {
    type Target = [BitMove];

    #[inline]
    fn deref(&self) -> &[BitMove] {
        unsafe {
            let p = self.inner.as_ptr();
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [BitMove] {
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Index<usize> for MoveList {
    type Output = BitMove;

    #[inline(always)]
    fn index(&self, index: usize) -> &BitMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for MoveList {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut BitMove {
        &mut (**self)[index]
    }
}


impl MVPushable for MoveList {
    #[inline(always)]
    fn push_mv(&mut self, mv: BitMove) {
        if self.len() < MAX_MOVES {
            unsafe{ self.unchecked_push_mv(mv) }
        }
    }

    #[inline(always)]
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove) {
        let end = self.inner.get_unchecked_mut(self.len);
        *end = mv;
        self.len += 1;
    }

    #[inline(always)]
    unsafe fn unchecked_set_len(&mut self, len: usize) {
        self.len = len
    }
}

pub struct MoveIter<'a> {
    movelist: &'a MoveList,
    idx: usize
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = BitMove;
    type IntoIter = MoveIter<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MoveIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl<'a> ExactSizeIterator for MoveIter<'a> {}

impl<'a> FusedIterator for MoveIter<'a> {}

unsafe impl<'a> TrustedLen for MoveIter<'a> {}

// Iterator for the `MoveList`.
pub struct MoveIntoIter {
    movelist: MoveList,
    idx: usize
}

impl Iterator for MoveIntoIter {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl IntoIterator for MoveList {
    type Item = BitMove;
    type IntoIter = MoveIntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MoveIntoIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl FromIterator<BitMove> for MoveList {
    fn from_iter<T: IntoIterator<Item=BitMove>>(iter: T) -> Self {
        let mut list = MoveList::default();
        for i in iter {
            list.push(i);
        }
        list
    }
}

impl ExactSizeIterator for MoveIntoIter {}

impl FusedIterator for MoveIntoIter {}

unsafe impl TrustedLen for MoveIntoIter {}


/// This is similar to a `MoveList`, but also keeps the scores for each move as well.
pub struct ScoringMoveList {
    inner: [ScoringBitMove; 256],
    len: usize,
}

impl Default for ScoringMoveList {
    #[inline]
    fn default() -> Self {
        ScoringMoveList {
            inner: [ScoringBitMove::default(); 256],
            len: 0,
        }
    }
}

impl From<Vec<BitMove>> for ScoringMoveList {
    fn from(vec: Vec<BitMove>) -> Self {
        let mut list = ScoringMoveList::default();
        vec.iter().for_each(|m| list.push(*m));
        list
    }
}

impl From<MoveList> for ScoringMoveList {
    fn from(mv_list: MoveList) -> Self {
        let mut sc_list = ScoringMoveList::default();
        mv_list.iter().for_each(|m| sc_list.push(*m));
        sc_list
    }
}

impl Into<Vec<ScoringBitMove>> for ScoringMoveList {
    #[inline]
    fn into(self) -> Vec<ScoringBitMove> {
        self.vec()
    }
}

impl ScoringMoveList {
    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    #[inline(always)]
    pub fn push(&mut self, mov: BitMove) {
        self.push_mv(mov)
    }

    /// Returns true if empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a vector from this `MoveList`.
    pub fn vec(&self) -> Vec<ScoringBitMove> {
        let mut vec = Vec::with_capacity(self.len);
        for pair in self.iter() {
            vec.push(*pair);
        }
        assert_eq!(vec.len(),self.len);
        vec
    }

    /// Returns the number of moves inside the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the `MoveList` as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[ScoringBitMove] {
        self
    }
}


impl Deref for ScoringMoveList {
    type Target = [ScoringBitMove];

    #[inline]
    fn deref(&self) -> &[ScoringBitMove] {
        unsafe {
            let p = self.inner.as_ptr();
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl DerefMut for ScoringMoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [ScoringBitMove] {
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Index<usize> for ScoringMoveList {
    type Output = ScoringBitMove;

    #[inline(always)]
    fn index(&self, index: usize) -> &ScoringBitMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for ScoringMoveList {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut ScoringBitMove {
        &mut (**self)[index]
    }
}

impl MVPushable for ScoringMoveList {
    #[inline(always)]
    fn push_mv(&mut self, mv: BitMove) {
        if self.len() < MAX_MOVES {
            unsafe{ self.unchecked_push_mv(mv) }
        }
    }

    #[inline(always)]
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove) {
        let end = self.inner.get_unchecked_mut(self.len);
        *end = ScoringBitMove::new(mv);
        self.len += 1;
    }


    #[inline(always)]
    unsafe fn unchecked_set_len(&mut self, len: usize) {
        self.len = len
    }
}

pub struct ScoreMoveIter<'a> {
    movelist: &'a ScoringMoveList,
    idx: usize
}

impl<'a> Iterator for ScoreMoveIter<'a> {
    type Item = ScoringBitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a ScoringMoveList {
    type Item = ScoringBitMove;
    type IntoIter = ScoreMoveIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ScoreMoveIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl<'a> ExactSizeIterator for ScoreMoveIter<'a> {}

impl<'a> FusedIterator for ScoreMoveIter<'a> {}

unsafe impl<'a> TrustedLen for ScoreMoveIter<'a> {}

// Iterator for the `ScoringMoveList`.
pub struct ScoreMoveIntoIter {
    movelist: ScoringMoveList,
    idx: usize
}

impl Iterator for ScoreMoveIntoIter {
    type Item = ScoringBitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl IntoIterator for ScoringMoveList {
    type Item = ScoringBitMove;
    type IntoIter = ScoreMoveIntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ScoreMoveIntoIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl FromIterator<BitMove> for ScoringMoveList {
    fn from_iter<T: IntoIterator<Item=BitMove>>(iter: T) -> Self {
        let mut list = ScoringMoveList::default();
        for i in iter {
            list.push(i);
        }
        list
    }
}

impl ExactSizeIterator for ScoreMoveIntoIter {}

impl FusedIterator for ScoreMoveIntoIter {}

unsafe impl TrustedLen for ScoreMoveIntoIter {}


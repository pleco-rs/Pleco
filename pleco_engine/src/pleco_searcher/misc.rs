//! Miscellaneous structures.


use THREAD_STACK_SIZE;

use std::mem;

pub trait PVNode {
    fn is_pv() -> bool;
}

pub struct PV {}
pub struct NonPV {}

impl PVNode for PV {
    fn is_pv() -> bool {
        true
    }
}

impl PVNode for NonPV {
    fn is_pv() -> bool {
        false
    }
}


pub struct ThreadStack {
    pub pos_eval: i32,
}

impl ThreadStack {
    pub fn new() -> Self {
        ThreadStack {
            pos_eval: 0
        }
    }
}

pub fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}


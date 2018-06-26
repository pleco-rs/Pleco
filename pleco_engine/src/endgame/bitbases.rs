use std::sync::{ONCE_INIT,Once};
use std::mem;

use pleco::{SQ,Player,Board};
use pleco::core::masks::PLAYER_CNT;

const BITBASE_RESULT_INVALID: u8    = 0b0000;
const BITBASE_RESULT_UNKNOWN: u8    = 0b0001;
const BITBASE_RESULT_DRAW: u8       = 0b0010;
const BITBASE_RESULT_WIN: u8        = 0b0100;

static INITALIZED: Once = ONCE_INIT;

pub fn init() {
    INITALIZED.call_once(|| {

    });
}

bitflags! {
    pub struct BitbaseResult: u8 {
        const INVALID  = BITBASE_RESULT_INVALID;
        const UNKNOWN  = BITBASE_RESULT_UNKNOWN;
        const DRAW     = BITBASE_RESULT_DRAW;
        const WIN      = BITBASE_RESULT_WIN;
    }
}

struct KPKPosition {
    us: Player,
    ksq: [SQ; PLAYER_CNT],
    psq: SQ,
    result: BitbaseResult
}

impl KPKPosition {
//    fn new(index: u32) -> Self {
//        let us: Player = unsafe {
//            mem::transmute((idx >> 12) as u8 &0b1)
//        };
//
//
//
//        KPKPosition {
//            us: us
//        }
//    }
}

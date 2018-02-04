use pleco::{Player, File, SQ, BitBoard, Board, PieceType, Rank};
use pleco::core::masks::{PLAYER_CNT,RANK_CNT};
use pleco::core::score::*;
use pleco::core::mono_traits::*;
use pleco::board::castle_rights::Castling;
use pleco::core::masks::FILE_DISPLAYS;
use pleco::core::CastleType;

use super::TableBase;

use std::mem::transmute;


pub struct Material {
    table: TableBase<MaterialEntry>,
}

pub struct MaterialEntry {
    key: u64,
    value: Value
}

impl Material {
    pub fn probe(&mut self, board: &Board) -> &mut MaterialEntry {
        unimplemented!()
    }
}
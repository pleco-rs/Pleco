
use templates::*;
use std::fmt;

// A move needs 16 bits to be stored
//
// bit  0- 5: destination square (from 0 to 63)
// bit  6-11: origin square (from 0 to 63)
// bit 12-13: promotion piece type - 2 (from KNIGHT-2 to QUEEN-2)
// bit 14-15: special move flag: promotion (1), en passant (2), castling (3)
// NOTE: EN-PASSANT bit is set only when a pawn can be captured
//
// Special cases are MOVE_NONE and MOVE_NULL. We can sneak these in because in
// any normal move destination square is always different from origin square
// while MOVE_NONE and MOVE_NULL have the same origin and destination square.

// x??? --> Promotion bit
// ?x?? --> Capture bit
// ??xx --> flaf Bit

// 0000  ===> Quiet move
// 0001  ===> Double Pawn Push
// 0010  ===> King Castle
// 0011  ===> Queen Castle
// 0100  ===> Capture
// 0101  ===> EP Capture
// 0110  ===>
// 0111  ===>
// 1000  ===> Knight Promotion
// 1001  ===> Bishop Promo
// 1010  ===> Rook   Promo
// 1011  ===> Queen  Capture  Promo
// 1100  ===> Knight Capture  Promotion
// 1101  ===> Bishop Capture  Promo
// 1110  ===> Rook   Capture  Promo
// 1111  ===> Queen  Capture  Promo


// Castles have the src as the king bit and the dst as the rook


static SRC_MASK: u16 =  0b0000_000000_111111;
static DST_MASK: u16 =  0b0000_111111_000000;
static PR_MASK: u16 =   0b1000_000000_000000;
static CP_MASK: u16 =   0b0100_000000_000000;
static FLAG_MASK: u16 = 0b1111_000000_000000;
static SP_MASK: u16 =   0b0011_000000_000000;

#[derive(Copy, Clone, PartialEq)]
pub struct BitMove {
    data: u16,
}

#[derive(Copy, Clone, PartialEq)]
pub enum MoveFlag {
    Promotion { capture: bool, prom: Piece },
    Castle { king_side: bool },
    DoublePawnPush,
    Capture { ep_capture: bool },
    QuietMove,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MoveType {
    Promotion,
    Castle,
    EnPassant,
    Normal
}

#[derive(Copy, Clone,PartialEq)]
pub struct PreMoveInfo {
    pub src: SQ,
    pub dst: SQ,
    pub flags: MoveFlag,
}

impl fmt::Display for BitMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}



// https://chessprogramming.wikispaces.com/Encoding+Moves
impl BitMove {
    pub fn new(input: u16) -> BitMove {
        BitMove { data: input }
    }
    pub fn init(info: PreMoveInfo) -> BitMove {
        let src = info.src as u16;
        let dst = (info.dst as u16) << 6;
        let flags = info.flags;
        let flag_bits: u16 = match flags {
            MoveFlag::Promotion { capture, prom } => {
                let p_bit: u16 = match prom {
                    Piece::R => { 2 }
                    Piece::B => { 1 }
                    Piece::N => { 0 }
                    Piece::Q | _ => { 3 }
                };
                let cp_bit = if capture { 4 } else { 0 };
                p_bit + cp_bit + 8
            }
            MoveFlag::Capture { ep_capture } => {
                if ep_capture { 5 } else { 4 }
            }
            MoveFlag::Castle { king_side } => {
                if king_side { 2 } else { 3 }
            }
            MoveFlag::DoublePawnPush => { 1 }
            MoveFlag::QuietMove => { 0 }
        };
        BitMove { data: (flag_bits << 12) | src | dst }
    }

    pub fn null() -> Self {
        BitMove {data: 0}
    }

    pub fn is_null(&self) -> bool {
        self.data == 0
    }

    // Note: Encompasses two missing Spots
    #[inline(always)]
    pub fn is_capture(&self) -> bool { ((self.data & CP_MASK) >> 14) == 1 }

    #[inline(always)]
    pub fn is_quiet_move(&self) -> bool { ((self.data & FLAG_MASK) >> 12) == 0 }

    #[inline(always)]
    pub fn is_promo(&self) -> bool { (self.data & PR_MASK) != 0 }

    #[inline(always)]
    pub fn get_dest(&self) -> SQ { ((self.data & DST_MASK) >> 6) as u8 }

    #[inline(always)]
    pub fn get_src(&self) -> SQ { (self.data & SRC_MASK) as u8 }

    #[inline(always)]
    pub fn is_castle(&self) -> bool { ((self.data & FLAG_MASK) >> 13) == 1 }

    #[inline(always)]
    pub fn is_king_castle(&self) -> bool { ((self.data & FLAG_MASK) >> 12) == 2 }

    #[inline(always)]
    pub fn is_queen_castle(&self) -> bool { ((self.data & FLAG_MASK) >> 12) == 3 }

    #[inline(always)]
    pub fn is_en_passant(&self) -> bool { (self.data & FLAG_MASK) >> 12 == 5 }

    #[inline(always)]
    pub fn is_double_push(&self) -> (bool, u8) {
        let is_double_push: u8 = ((self.data & FLAG_MASK) >> 12) as u8;
        match is_double_push {
            1 => (true, self.get_dest() as u8),
            _ => (false, 64)
        }
    }

    #[inline(always)]
    pub fn dest_row(&self) -> u8 {
        ((self.data & DST_MASK) >> 6) as u8 / 8
    }

    #[inline(always)]
    pub fn dest_col(&self) -> u8 {
        ((self.data & DST_MASK) >> 6) as u8 % 8
    }
    #[inline(always)]
    pub fn src_row(&self) -> u8 {
        (self.data & SRC_MASK) as u8 / 8
    }
    #[inline(always)]
    pub fn src_col(&self) -> u8 {
        (self.data & SRC_MASK) as u8 % 8
    }

    // Assume Piece is promoted
    #[inline(always)]
    pub fn promo_piece(&self) -> Piece {
        match (self.data >> 12) & 0b0011 {
            0 => Piece::N,
            1 => Piece::B,
            2 => Piece::R,
            3 | _ => Piece::Q,
        }
    }

    #[inline(always)]
    pub fn move_type(&self) -> MoveType {
        if self.is_castle() {return MoveType::Castle}
        if self.is_promo() {return MoveType::Promotion}
        if self.is_en_passant() { return MoveType::EnPassant}
        MoveType::Normal
    }

    pub fn stringify(&self) -> String {
        let src = parse_sq(self.get_src());
        let dst = parse_sq(self.get_dest());
        let mut s = format!("{}{}",src,dst);
        if self.is_promo() {
            let char = match self.promo_piece() {
                Piece::B => 'b',
                Piece::N => 'n',
                Piece::R => 'r',
                Piece::Q => 'q',
                _ => unreachable!()
            };
            s.push(char);
        }
        s
    }

    pub fn get_raw(&self) -> u16 {
        self.data
    }
}


//! Table to map from position -> material value;

use pleco::core::masks::{PIECE_TYPE_CNT, PLAYER_CNT};
use pleco::core::mono_traits::*;
use pleco::core::score::*;
use pleco::tools::{prefetch_write, PreFetchable};
use pleco::{Board, PieceType, Player};

use super::{TableBase, TableBaseConst};

pub const PHASE_END_GAME: u16 = 0;
pub const PHASE_MID_GAME: u16 = 128;

pub const SCALE_FACTOR_DRAW: u8 = 0;
pub const SCALE_FACTOR_ONEPAWN: u8 = 48;
pub const SCALE_FACTOR_NORMAL: u8 = 64;
pub const SCALE_FACTOR_MAX: u8 = 128;
pub const SCALE_FACTOR_NONE: u8 = 255;

// Polynomial material imbalance parameters
const QUADRATIC_OURS: [[i32; PIECE_TYPE_CNT - 2]; PIECE_TYPE_CNT - 2] = [
    [1667, 0, 0, 0, 0, 0],           // Bishop pair
    [40, 0, 0, 0, 0, 0],             // Pawn
    [32, 255, -3, 0, 0, 0],          // Knight      OUR PIECES
    [0, 104, 4, 0, 0, 0],            // Bishop
    [-26, -2, 47, 105, -149, 0],     // Rook
    [-189, 24, 117, 133, -134, -10], // Queen
]; // pair pawn knight bishop rook queen
   //            OUR PIECES

const QUADRATIC_THEIRS: [[i32; PIECE_TYPE_CNT - 2]; PIECE_TYPE_CNT - 2] = [
    [0, 0, 0, 0, 0, 0],          // Bishop pair
    [36, 0, 0, 0, 0, 0],         // Pawn
    [9, 63, 0, 0, 0, 0],         // Knight      OUR PIECES
    [59, 65, 42, 0, 0, 0],       // Bishop
    [46, 39, 24, -24, 0, 0],     // Rook
    [97, 100, -42, 137, 268, 0], // Queen
]; // pair pawn knight bishop rook queen
   //           THEIR PIECES

pub struct MaterialEntry {
    key: u64,
    pub value: Value,
    pub factor: [u8; PLAYER_CNT],
    pub phase: u16,
}

impl MaterialEntry {
    #[inline(always)]
    pub fn score(&self) -> Score {
        Score(self.value, self.value)
    }

    #[inline(always)]
    pub fn scale_factor(&self, player: Player) -> u8 {
        self.factor[player as usize]
    }
}

// TODO: Use const-generics once it becomes available
impl TableBaseConst for MaterialEntry {
    const ENTRY_COUNT: usize = 8192;
}

//pawns: PawnTable::new(16384),
//material: Material::new(8192),
pub struct Material {
    table: TableBase<MaterialEntry>,
}

impl PreFetchable for Material {
    /// Pre-fetches a particular key. This means bringing it into the cache for faster eventual
    /// access.
    #[inline(always)]
    fn prefetch(&self, key: u64) {
        unsafe {
            let ptr = self.table.get_ptr(key);
            prefetch_write(ptr);
        }
    }
}

unsafe impl Send for Material {}

impl Material {
    /// Creates a new `Material` of `size` entries.
    ///
    /// # Panics
    ///
    /// Panics if size is not a power of 2.
    pub fn new() -> Self {
        Material {
            table: TableBase::new().unwrap(),
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn probe(&mut self, board: &Board) -> &mut MaterialEntry {
        let key: u64 = board.material_key();
        let entry: &mut MaterialEntry = self.table.get_mut(key);
        if entry.key == key {
            return entry;
        }

        entry.key = key;
        entry.factor = [SCALE_FACTOR_NORMAL; PLAYER_CNT];

        let npm_w: Value = board.non_pawn_material(Player::White);
        let npm_b: Value = board.non_pawn_material(Player::Black);
        let npm: Value = END_GAME_LIMIT.max(MID_GAME_LIMIT.min(npm_w + npm_b));

        entry.phase = (((npm - END_GAME_LIMIT) * PHASE_MID_GAME as i32)
            / (MID_GAME_LIMIT - END_GAME_LIMIT)) as u16;

        let w_pawn_count: u8 = board.count_piece(Player::White, PieceType::P);
        let w_knight_count: u8 = board.count_piece(Player::White, PieceType::N);
        let w_bishop_count: u8 = board.count_piece(Player::White, PieceType::B);
        let w_rook_count: u8 = board.count_piece(Player::White, PieceType::R);
        let w_queen_count: u8 = board.count_piece(Player::White, PieceType::Q);

        let b_pawn_count: u8 = board.count_piece(Player::Black, PieceType::P);
        let b_knight_count: u8 = board.count_piece(Player::Black, PieceType::N);
        let b_bishop_count: u8 = board.count_piece(Player::Black, PieceType::B);
        let b_rook_count: u8 = board.count_piece(Player::Black, PieceType::R);
        let b_queen_count: u8 = board.count_piece(Player::Black, PieceType::Q);

        if w_pawn_count == 0 && npm_w - npm_b <= BISHOP_MG {
            entry.factor[Player::White as usize] = if npm_w < ROOK_MG {
                SCALE_FACTOR_DRAW
            } else if npm_b <= BISHOP_MG {
                4
            } else {
                14
            };
        }

        if b_pawn_count == 0 && npm_b - npm_w <= BISHOP_MG {
            entry.factor[Player::Black as usize] = if npm_b < ROOK_MG {
                SCALE_FACTOR_DRAW
            } else if npm_w <= BISHOP_MG {
                4
            } else {
                14
            };
        }

        if w_pawn_count == 1 && npm_w - npm_b <= BISHOP_MG {
            entry.factor[Player::White as usize] = SCALE_FACTOR_ONEPAWN;
        }

        if b_pawn_count == 1 && npm_b - npm_w <= BISHOP_MG {
            entry.factor[Player::Black as usize] = SCALE_FACTOR_ONEPAWN;
        }

        let w_pair_bish: u8 = (w_bishop_count > 1) as u8;
        let b_pair_bish: u8 = (b_bishop_count > 1) as u8;

        let piece_counts: [[u8; PIECE_TYPE_CNT - 2]; PLAYER_CNT] = [
            [
                w_pair_bish,
                w_pawn_count,
                w_knight_count,
                w_bishop_count,
                w_rook_count,
                w_queen_count,
            ],
            [
                b_pair_bish,
                b_pawn_count,
                b_knight_count,
                b_bishop_count,
                b_rook_count,
                b_queen_count,
            ],
        ];

        entry.value =
            (imbalance::<WhiteType>(&piece_counts) - imbalance::<BlackType>(&piece_counts)) / 16;

        entry
    }
}

fn imbalance<P: PlayerTrait>(piece_counts: &[[u8; PIECE_TYPE_CNT - 2]; PLAYER_CNT]) -> i32 {
    let mut bonus: i32 = 0;

    for pt1 in 0..6 {
        if piece_counts[P::player() as usize][pt1] == 0 {
            continue;
        }

        let mut v: i32 = 0;

        for pt2 in 0..6 {
            v += QUADRATIC_OURS[pt1][pt2] * piece_counts[P::player() as usize][pt2] as i32
                + QUADRATIC_THEIRS[pt1][pt2] * piece_counts[P::opp_player() as usize][pt2] as i32;
        }

        bonus += piece_counts[P::player() as usize][pt1] as i32 * v;
    }
    bonus
}

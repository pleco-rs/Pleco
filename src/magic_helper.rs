

use bit_twiddles::*;
use templates::*;
use std::{mem,slice,cmp};
use test::Bencher;
use test;


const ROOK_M_SIZE: usize = 102400;
const BISHOP_M_SIZE: usize = 5248;
const B_DELTAS: [i8; 4] = [7,9,-9,-7];
const R_DELTAS: [i8; 4] = [8,1,-8,-1];
const DELTAS: [[i8; 4]; 2] = [B_DELTAS, R_DELTAS];
const SEEDS: [[u64;8]; 2] = [[ 8977, 44560, 54343, 38998,  5731, 95205, 104912, 17020 ],
                             [  728, 10316, 55013, 32803, 12281, 15100,  16645,   255 ]];



// Object for helping the Board with various functions. Pre-computes everything on initialization
// Thread safe. Once initializes, IT SHOULD NOT BE MODIFIED
// Currently does the following:
//      - Generates King and Rook Move Bitboards
//      - Generates Rook, Bishop, Queen Magic Bitboards for Move generation
//      - Generates distance table for quick lookup of distance
//
//
// Size (Bytes) of each field in the Stack / Heap
//              |  STACK  |  HEAP   |  TOTAL  | KiloBytes   |
// magic_rook   |   2563  |  819200 |  821763 | ~819.2 KB   |
// magic_bishop |   2563  |   41984 |   44547 |  ~44.5 KB   |
// knight_table |    512  |       0 |     512 |   ~0.5 KB   |
// king_table   |    512  |       0 |     512 |   ~0.5 KB   |
// dist_table   |   4096  |       0 |     512 |   ~4.0 KB   |
// line_bb      |    512  |       0 |     512 |   ~0.5 KB   |
//
//
pub struct MagicHelper<'a, 'b> {
    magic_rook: MRookTable<'a>,
    magic_bishop: MBishopTable<'b>,
    knight_table: [u64; 64],
    king_table: [u64; 64],
    dist_table: [[SQ; 64]; 64],
    line_bitboard:[[u64; 64]; 64],
    between_sqs_bb: [[u64; 64]; 64],
    adjacent_files_bb: [u64;8],
    pawn_attacks_from: [[u64;64]; 2],
    pub zobrist: Zobrist,
}

// Structure for helping determine Zobrist hashes.
pub struct Zobrist {
    pub sq_piece: [[u64; PIECE_CNT]; SQ_CNT],
    pub en_p: [u64; FILE_CNT],
    pub castle: [u64; TOTAL_CASTLING_CNT],
    pub side: u64,
}

// Creates zobrist hashes based on a Pseudo Random Number generator.
impl Zobrist {
    fn default() -> Zobrist {
        let mut zob = Zobrist {
            sq_piece: [[0; PIECE_CNT]; SQ_CNT],
            en_p: [0; FILE_CNT],
            castle: [0; TOTAL_CASTLING_CNT],
            side: 0,
        };

        let zobrist_seed: u64 = 23081;
        let mut rng = PRNG::init(zobrist_seed);

        for i in 0..SQ_CNT {
            for j in 0..PIECE_CNT {
                zob.sq_piece[i][j] = rng.rand_change();
            }
        }

        for i in 0..FILE_CNT {
            zob.en_p[i] = rng.rand_change()
        }

        for i in 0..TOTAL_CASTLING_CNT {
            zob.castle[i] = rng.rand_change()
        }

        zob.side = rng.rand_change();
        zob
    }
}

unsafe impl<'a,'b> Send for MagicHelper<'a,'b> {}

unsafe impl<'a,'b> Sync for MagicHelper<'a,'b> {}

// TO IMPLEMENT:
//      Adjacent Files BitBoard
//      Between Squares BitBoard

impl <'a,'b>MagicHelper<'a,'b> {

    // Create a new Magic Helper
    pub fn new() -> MagicHelper<'a,'b> {
        let mut mhelper = MagicHelper {
            magic_rook: MRookTable::init(),
            magic_bishop: MBishopTable::init(),
            knight_table: gen_knight_moves(),
            king_table: gen_king_moves(),
            dist_table: init_distance_table(),
            line_bitboard: [[0; 64]; 64],
            between_sqs_bb: [[0; 64]; 64],
            adjacent_files_bb: [0; 8],
            pawn_attacks_from: [[0; 64]; 2],
            zobrist: Zobrist::default(),
        };
        mhelper.gen_line_bbs();
        mhelper.gen_between_bbs();
        mhelper.gen_adjacent_file_bbs();
        mhelper.gen_pawn_attacks();
        mhelper
    }

    // Returns the Zobrist Hash for a given piece as a given Square
    #[inline(always)]
    pub fn z_piece_at_sq(&self, piece: Piece, square: SQ) -> u64 {
        assert!(sq_is_okay(square));
        self.zobrist.sq_piece[square as usize][piece as usize]
    }

    // Returns the zobrist hash for the given Square of Enpassant
    // Doesnt assume the EP square is a valid square. It will take the file of the square regardless.
    #[inline(always)]
    pub fn z_ep_file(&self, square: SQ) -> u64 {
        self.zobrist.en_p[file_of_sq(square) as usize]
    }

    // Returns a zobrast hash of the castling rights, as defined by the Board
    #[inline(always)]
    pub fn z_castle_rights(&self, castle: u8) -> u64 {
        assert!((castle as usize) < TOTAL_CASTLING_CNT);
        self.zobrist.castle[castle as usize]
    }

    // Returns Zobrist Hash of flipping sides
    #[inline(always)]
    pub fn z_side(&self) -> u64 {
        self.zobrist.side
    }

    // Generate Knight Moves bitboard from a source square
    #[inline(always)]
    pub fn knight_moves(&self, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        self.knight_table[square as usize]
    }

    // Generate King moves bitboard from a source  square
    #[inline(always)]
    pub fn king_moves(&self, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        self.king_table[square as usize]
    }

    // Generate Bishop Moves from a bishop square and all occupied squares on the board
    #[inline(always)]
    pub fn bishop_moves(&self, occupied: BitBoard, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        self.magic_bishop.bishop_attacks(occupied, square)
    }

    // Generate Rook Moves from a bishop square and all occupied squares on the board
    #[inline(always)]
    pub fn rook_moves(&self, occupied: BitBoard, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        self.magic_rook.rook_attacks(occupied, square)
    }

    // Generate Queen Moves from a bishop square and all occupied squares on the board
    #[inline(always)]
    pub fn queen_moves(&self, occupied: BitBoard, square: SQ) -> BitBoard {
        assert!(sq_is_okay(square));
        self.magic_rook.rook_attacks(occupied, square) | self.magic_bishop.bishop_attacks(occupied, square)
    }

    // get the distance of two squares
    #[inline(always)]
    pub fn distance_of_sqs(&self, square_one: SQ, square_two: SQ) -> u8 {
        assert!(sq_is_okay(square_one));
        assert!(sq_is_okay(square_two));
        self.dist_table[square_one as usize][square_two as usize]
    }

    // Get the line (diagonal / file / rank) two squares, if it exists
    #[inline(always)]
    pub fn line_bb(&self, square_one: SQ, square_two: SQ) -> BitBoard {
        assert!(sq_is_okay(square_one));
        assert!(sq_is_okay(square_two));
        self.line_bitboard[square_one as usize][square_two as usize]
    }

    // Get the line between two squares, not including the squares, if it exists
    #[inline(always)]
    pub fn between_bb(&self, square_one: SQ, square_two: SQ) -> BitBoard {
        assert!(sq_is_okay(square_one));
        assert!(sq_is_okay(square_two));
        self.between_sqs_bb[square_one as usize][square_two as usize]
    }

    // Gets the adjacent files of the square
    #[inline(always)]
    pub fn adjacent_file(&self, square: SQ,) -> BitBoard {
        assert!(sq_is_okay(square));
        self.adjacent_files_bb[file_of_sq(square) as usize]
    }

    // Pawn attacks from a given square, per player,
    // Basically, given square x,returns the bitboard of squares a pawn on x attacks
    #[inline(always)]
    pub fn pawn_attacks_from(&self, square: SQ, player: Player) -> BitBoard {
        assert!(sq_is_okay(square));
        match player {
            Player::White => self.pawn_attacks_from[0][square as usize],
            Player::Black => self.pawn_attacks_from[1][square as usize],
        }
    }


    // Returns in three Squares are in the same diagonal, file, or rank
    #[inline(always)]
    pub fn aligned(&self, s1: SQ, s2: SQ, s3: SQ) -> bool {
        self.line_bb(s1, s2) & sq_to_bb(s3) != 0
    }

    fn gen_line_bbs(&mut self) {
        for d in 0..DELTAS.len() {
            for i in 0..64 as SQ {
                for j in 0..64 as SQ {
                    let mut line_board: BitBoard = sliding_attack(&DELTAS[d], i, 0) & sliding_attack(&DELTAS[d], j, 0);
                    line_board |= ((1 as u64) << i) | ((1 as u64) << j);
                    self.line_bitboard[i as usize][i as usize] |= line_board;
                }
            }
        }
    }
    fn gen_between_bbs(&mut self) {
        for d in 0..DELTAS.len() {
            for i in 0..64 as SQ {
                for j in 0..64 as SQ {
                    self.between_sqs_bb[i as usize][i as usize] |= sliding_attack(&DELTAS[d], j, ((1 as u64) << i))
                        & sliding_attack(&DELTAS[d], i, ((1 as u64) << j));
                }
            }
        }
    }

    // Generates adjacent files of a given file
    // Files go from 0..7, representing files 1..8
    fn gen_adjacent_file_bbs(&mut self) {
        for file in 0..8 as SQ {
            if file != 0 { self.adjacent_files_bb[file as usize] |= file_bb(file - 1)}
            if file != 7 { self.adjacent_files_bb[file as usize] |= file_bb(file + 1)}
        }
    }



    fn gen_pawn_attacks(&mut self) {
        // gen white pawn attacks
        for i in 0..56 as u8 {
            let mut bb: u64 = 0;
            if file_of_sq(i) != File::A {
                bb |= sq_to_bb(i + 7)
            }
            if file_of_sq(i) != File::H {
                bb |= sq_to_bb(i + 9)
            }
            self.pawn_attacks_from[0][i as usize] = bb;
        }

        // Black pawn attacks
        for i in 8..64 as u8 {
            let mut bb: u64 = 0;
            if file_of_sq(i) != File::A {
                bb |= sq_to_bb(i - 9)
            }
            if file_of_sq(i) != File::H {
                bb |= sq_to_bb(i - 7)
            }
            self.pawn_attacks_from[1][i as usize] = bb;
        }
    }


}


// the REAL magic bitboard structure. For a certain square, contains a mask,
// magic number, num to shift by, and a pointer into the array where this is held
#[warn(dead_code)]
struct SMagic<'a> {
    ptr: &'a [u64],
    mask: u64,
    magic: u64,
    shift: u32
}

// Temporary struct used to Create an actual Magic BitBoard Object
#[warn(dead_code)]
struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32
}

impl PreSMagic {
    pub fn init() -> PreSMagic {
        PreSMagic {start: 0, len: 0, mask: 0, magic: 0, shift: 0}
    }

    // creates an array of PreSMagic
    pub unsafe fn init64() -> [PreSMagic; 64] {
        let arr: [PreSMagic; 64] = mem::uninitialized();
        arr
    }

    // Helper method to compute the next index
    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

// Struct containing our Rook Magic BitBoards
#[allow(dead_code)]
struct MRookTable<'a> {
    sq_magics: [SMagic<'a>; 64],
    attacks: Vec<BitBoard> // 102400 long
}

// Struct containing Bishop Rook Magic BitBoards
#[allow(dead_code)]
struct MBishopTable<'a> {
    sq_magics: [SMagic<'a>; 64],
    attacks: Vec<BitBoard> // 5248 long
}

impl <'a> MRookTable<'a>  {

    // simple version that creates the table with an empty array.
    // used for testing purposes where MagicStruct is not needed
    pub fn simple() -> MRookTable<'a> {
        let sq_table: [SMagic<'a>; 64] =  unsafe{mem::uninitialized()};
        MRookTable{sq_magics: sq_table, attacks: Vec::new()}
    }

    // Creates the Magic Rook Table Struct
    pub fn init() -> MRookTable<'a> {
        // Creates PreSMagic to hold raw numbers. Technically jsut adds room to stack
        let mut pre_sq_table: [PreSMagic; 64] = unsafe {PreSMagic::init64() };

        // Initializes each PreSMagic
        for i in 0..64 { pre_sq_table[i] = PreSMagic::init(); }

        // Creates Vector to hold attacks. Has capacity as we know the exact size of this.
        let mut attacks: Vec<BitBoard> = vec![0; ROOK_M_SIZE];

        // Occupancy tracks occupancy permutations. MAX permutations = subset of 12 bits = 2^12
        // Reference is similar, tracks the sliding moves from a given occupancy
        // Age tracks the best index for a current permutation
        let mut occupancy: [u64; 4096] = [0; 4096];
        let mut reference: [u64; 4096] = [0; 4096];
        let mut age: [i32; 4096] =  [0; 4096];

        // Size tracks the size of permutations of the current block
        let mut size: usize;

        // b is used for generating the permutations through ripple - carry
        let mut b: u64;

        // current and i is a placeholder for actually generating correct magic numbers
        let mut current: i32 = 0;
        let mut i: usize;

        // set the first PreSMagic start = 0. Just in case.
        pre_sq_table[0].start = 0;

        // Loop through each square! s is a SQ
        for s in 0..64 as SQ {
            // Magic number for later
            let mut magic: u64;

            // edges is the bitboard represenation of the edges s is not on.
            // e.g. sq A1 is on FileA and Rank1, so edges = bitboard of FileH and Rank8
            // mask = occupancy mask of square s
            let edges: BitBoard = ((RANK_1 | RANK_8) & !rank_bb(s)) | ((FILE_A | FILE_H) & !file_bb(s));
            let mask: BitBoard = sliding_attack(&R_DELTAS, s, 0) & !edges;

            // Shift = number of bits in 64 - bits in mask = log2(size)
            let shift: u32 = (64 - popcount64(mask)) as u32;
            b = 0;
            size = 0;

            // Ripple carry to determine occupancy, reference, and size
            'bit: loop {
                occupancy[size] = b;
                reference[size] = sliding_attack(&R_DELTAS, s, b);
                size += 1;
                b = ((b).wrapping_sub(mask)) as u64 & mask;
                if b == 0 { break 'bit; }
            }

            // Set current PreSMagic length to be of size
            pre_sq_table[s as usize].len = size;

            // If there is a next square, set the start of it.
            if s < 63 {
                pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();

            }
            // Create our Random Number Generator with a seed
            let mut rng = PRNG::init(SEEDS[1][rank_of_sq(s) as usize]);

            // Loop until we have found our magics!
            'outer: loop {
                // Create a magic with our desired number of bits in the first 8 places
                'first_in: loop {
                    magic = rng.sparse_rand();
                    if popcount64((magic.wrapping_mul(mask)).wrapping_shr(56)) >= 6 {
                        break 'first_in;
                    }
                }
                current += 1;
                i = 0;

                // Filling the attacks Vector up to size digits
                while i < size {
                    // Magic part! The index is = ((occupancy[s] & mask) * magic >> shift)
                    let index: usize = ((occupancy[i as usize] & mask).wrapping_mul(magic) as u64).wrapping_shr(shift) as usize;

                    // Checking to see if we have visited this index already with a lower current number
                    if age[index] < current {

                        // If we have visited with lower current, we replace it with this current number,
                        // as this current is higher and has gone through more passes
                        age[index] = current;
                        attacks[pre_sq_table[s as usize].start + index] = reference[i];

                    } else if attacks[pre_sq_table[s as usize].start + index] != reference[i] {
                        // If a magic maps to the same index but different result, either magic is bad or we are done
                        break;
                    }
                    i += 1;
                }
                // If we have filled it up to size or greater, we are done
                if i >= size {
                    break 'outer;
                }
            }
            // Set the remaining variables for the PreSMagic Struct
            pre_sq_table[s as usize].magic = magic;
            pre_sq_table[s as usize].mask = mask;
            pre_sq_table[s as usize].shift = shift;
        }

        // Now the fun part. We got to convert all the PreMagicStructs to MStructs
        // UNSAFE as we are initializing raw memory, AND creating a Slice of our array from raw pointers. scary!
        unsafe {
            // Make Memory for our SMagics!
            let mut sq_table: [SMagic<'a>; 64] = mem::uninitialized();

            // size = running total of total size
            let mut size = 0;
            for i in 0.. 64 {
                // begin ptr points to the beginning of the current slice in the vector
                let beginptr = attacks.as_ptr().offset(size as isize);
                let mut table_i: SMagic = SMagic {
                    ptr: mem::uninitialized(),
                    mask: pre_sq_table[i].mask,
                    magic: pre_sq_table[i].magic,
                    shift: pre_sq_table[i].shift,
                };
                // Create the pointer to the slice with begin_ptr / length
                table_i.ptr = slice::from_raw_parts(beginptr,pre_sq_table[i].len);
                size += pre_sq_table[i].len;
                sq_table[i] = table_i;
            }
            // Sanity check
            assert_eq!(size, ROOK_M_SIZE);
            MRookTable{sq_magics: sq_table, attacks: attacks}
        }
    }

    //NOTE: Result needs to be AND'd with player's occupied bitboard, so doesnt allow capturing self.
    #[inline(always)]
    pub fn rook_attacks(&self, mut occupied: BitBoard, square: SQ) -> BitBoard {
        let magic_entry: &SMagic = &self.sq_magics[square as usize];
        occupied &= magic_entry.mask;
        occupied = occupied.wrapping_mul(magic_entry.magic);
        occupied = occupied.wrapping_shr(magic_entry.shift);
        magic_entry.ptr[occupied as usize]
    }
}

impl <'a> MBishopTable<'a> {

    // simple version that creates the table with an empty array.
    // used for testing purposes where MagicStruct is not needed
    pub fn simple() -> MBishopTable<'a> {
        let sq_table: [SMagic<'a>; 64] =  unsafe{mem::uninitialized()};
        MBishopTable{sq_magics: sq_table, attacks: Vec::new()}
    }

    // Create MagicBishopBitBoards
    pub fn init() -> MBishopTable<'a> {
        // Creates PreSMagic to hold raw numbers. Technically jsut adds room to stack
        let mut pre_sq_table: [PreSMagic; 64] = unsafe {PreSMagic::init64() };

        // Initializes each PreSMagic
        for i in 0..64 { pre_sq_table[i] = PreSMagic::init(); }

        // Creates Vector to hold attacks. Has capacity as we know the exact size of this.
        let mut attacks: Vec<BitBoard> = vec![0; BISHOP_M_SIZE];

        // Occupancy tracks occupancy permutations. MAX permutations = subset of 12 bits = 2^12
        // Reference is similar, tracks the sliding moves from a given occupancy
        // Age tracks the best index for a current permutation
        let mut occupancy: [u64; 4096] = [0; 4096];
        let mut reference: [u64; 4096] = [0; 4096];
        let mut age: [i32; 4096] =  [0; 4096];

        // Size tracks the size of permutations of the current block
        let mut size: usize;

        // b is used for generating the permutations through ripple - carry
        let mut b: u64;

        // current and i is a placeholder for actually generating correct magic numbers
        let mut current: i32 = 0;
        let mut i: usize;

        // set the first PreSMagic start = 0. Just in case.
        pre_sq_table[0].start = 0;

        // Loop through each square! s is a SQ
        for s in 0..64 as SQ {
            // Magic number for later
            let mut magic: u64;

            // edges is the bitboard represenation of the edges s is not on.
            // e.g. sq A1 is on FileA and Rank1, so edges = bitboard of FileH and Rank8
            // mask = occupancy mask of square s
            let edges: BitBoard = ((RANK_1 | RANK_8) & !rank_bb(s)) | ((FILE_A | FILE_H) & !file_bb(s));
            let mask: BitBoard = sliding_attack(&B_DELTAS, s, 0) & !edges;

            // Shift = number of bits in 64 - bits in mask = log2(size)
            let shift: u32 = (64 - popcount64(mask)) as u32;
            b = 0;
            size = 0;

            // Ripple carry to determine occupancy, reference, and size
            'bit: loop {
                occupancy[size] = b;
                reference[size] = sliding_attack(&B_DELTAS, s, b);
                size += 1;
                b = ((b).wrapping_sub(mask)) as u64 & mask;
                if b == 0 { break 'bit; }
            }

            // Set current PreSMagic length to be of size
            pre_sq_table[s as usize].len = size;

            // If there is a next square, set the start of it.
            if s < 63 {
                pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();

            }
            // Create our Random Number Generator with a seed
            let mut rng = PRNG::init(SEEDS[1][rank_of_sq(s) as usize]);

            // Loop until we have found our magics!
            'outer: loop {
                // Create a magic with our desired number of bits in the first 8 places
                'first_in: loop {
                    magic = rng.sparse_rand();
                    if popcount64((magic.wrapping_mul(mask)).wrapping_shr(56)) >= 6 {
                        break 'first_in;
                    }
                }
                current += 1;
                i = 0;

                // Filling the attacks Vector up to size digits
                while i < size {
                    // Magic part! The index is = ((occupancy[s] & mask) * magic >> shift)
                    let index: usize = ((occupancy[i as usize] & mask).wrapping_mul(magic) as u64).wrapping_shr(shift) as usize;

                    // Checking to see if we have visited this index already with a lower current number
                    if age[index] < current {

                        // If we have visited with lower current, we replace it with this current number,
                        // as this current is higher and has gone through more passes
                        age[index] = current;
                        attacks[pre_sq_table[s as usize].start + index] = reference[i];

                    } else if attacks[pre_sq_table[s as usize].start + index] != reference[i] {
                        // If a magic maps to the same index but different result, either magic is bad or we are done
                        break;
                    }
                    i += 1;
                }
                // If we have filled it up to size or greater, we are done
                if i >= size {
                    break 'outer;
                }
            }
            // Set the remaining variables for the PreSMagic Struct
            pre_sq_table[s as usize].magic = magic;
            pre_sq_table[s as usize].mask = mask;
            pre_sq_table[s as usize].shift = shift;
        }

        // Now the fun part. We got to convert all the PreMagicStructs to MStructs
        // UNSAFE as we are initializing raw memory, AND creating a Slice of our array from raw pointers. scary!
        unsafe {
            // Make Memory for our SMagics!
            let mut sq_table: [SMagic<'a>; 64] = mem::uninitialized();

            // size = running total of total size
            let mut size = 0;
            for i in 0.. 64 {
                // begin ptr points to the beginning of the current slice in the vector
                let beginptr = attacks.as_ptr().offset(size as isize);
                let mut table_i: SMagic = SMagic {
                    ptr: mem::uninitialized(),
                    mask: pre_sq_table[i].mask,
                    magic: pre_sq_table[i].magic,
                    shift: pre_sq_table[i].shift,
                };
                // Create the pointer to the slice with begin_ptr / length
                table_i.ptr = slice::from_raw_parts(beginptr,pre_sq_table[i].len);
                size += pre_sq_table[i].len;
                sq_table[i] = table_i;
            }
            // Sanity check
            assert_eq!(size, BISHOP_M_SIZE);
            MBishopTable{sq_magics: sq_table, attacks: attacks}
        }
    }

    //NOTE: Result needs to be AND'd with player's occupied bitboard, so doesnt allow capturing self.
    #[inline(always)]
    pub fn bishop_attacks(&self, mut occupied: BitBoard, square: SQ) -> BitBoard {
        let magic_entry: &SMagic = &self.sq_magics[square as usize];
        occupied &= magic_entry.mask;
        occupied = occupied.wrapping_mul(magic_entry.magic);
        occupied = occupied.wrapping_shr(magic_entry.shift);
        magic_entry.ptr[occupied as usize]
    }

}

// Object to assist with Generating Random numbers for Magics
struct PRNG {
    seed: u64
}

impl PRNG {
    // Creates PRNG from a seed, seed cannot be zero
    pub fn init(s: u64) -> PRNG {
        assert_ne!(s,0);
        PRNG {seed: s}
    }

    // Returns pseudo random number
    #[allow(dead_code)]
    pub fn rand(&mut self) -> u64 {
        self.rand_change()
    }

    // Returns a number with on average 8 bits being set.
    pub fn sparse_rand(&mut self) -> u64 {
        let mut s = self.rand_change();
        s &= self.rand_change();
        s &= self.rand_change();
        s
    }

    fn rand_change(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685821657736338717)
    }
}

// Returns an array of king moves, seeing as kings can only move up to
// 8 static places no matter the square
fn gen_king_moves() -> [u64; 64] {
    let mut moves: [u64;64] = [0; 64];

    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;
        // LEFT
        if file != 0 {
            mask |= 1 << (index - 1);
        }
        // RIGHT
        if file != 7 {
            mask |= 1 << (index + 1);
        }
        // UP
        if index < 56  {
            mask |= 1 << (index + 8);
        }
        // DOWN
        if index > 7  {
            mask |= 1 << (index - 8);
        }
        // LEFT UP
        if file != 0 && index < 56 {
            mask |= 1 << (index + 7);
        }
        // LEFT DOWN
        if file != 0 && index > 7 {
            mask |= 1 << (index - 9);
        }
        // RIGHT DOWN
        if file!= 7 && index > 7 {
            mask |= 1 << (index - 7);
        }
        // RIGHT UP
        if file != 7 && index < 56 {
            mask |= 1 << (index + 9);
        }
        moves[index] = mask;
    }
    moves
}

// Returns an array of knight moves, seeing as kings can only move up to
// 8 static places no matter the square
fn gen_knight_moves() -> [u64; 64] {
    let mut moves: [u64;64] = [0; 64];
    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;

        // 1 UP   + 2 LEFT
        if file > 1 && index < 56 {
            mask |= 1 << (index + 6);
        }
        // 2 UP   + 1 LEFT
        if file != 0 && index < 48 {
            mask |= 1 << (index + 15);
        }
        // 2 UP   + 1 RIGHT
        if file != 7 && index < 48 {
            mask |= 1 << (index + 17);
        }
        // 1 UP   + 2 RIGHT
        if file < 6 && index < 56 {
            mask |= 1 << (index + 10);
        }
        // 1 DOWN   + 2 RIGHT
        if file < 6 && index > 7 {
            mask |= 1 << (index - 6);
        }
        // 2 DOWN   + 1 RIGHT
        if file != 7 && index > 15 {
            mask |= 1 << (index - 15 );
        }
        // 2 DOWN   + 1 LEFT
        if file != 0 && index > 15 {
            mask |= 1 << (index - 17 );
        }
        // 1 DOWN   + 2 LEFT
        if file > 1 && index > 7 {
            mask |= 1 << (index - 10 );
        }
        moves[index] = mask;
    }
    moves
}

// Returns a bitboards of sliding attacks given an array of 4 deltas
// Does not include the original position
// includes occupied bits if it runs into them, but stops before going further
fn sliding_attack(deltas: &[i8; 4], sq: SQ, occupied: BitBoard) -> BitBoard {
    assert!(sq < 64);
    let mut attack: BitBoard = 0;
    let square: i16 = sq as i16;
    for delta in deltas.iter().take(4 as usize) {
        let mut s: SQ = ((square as i16) + (*delta as i16)) as u8;
        'inner: while sq_is_okay(s as u8) && sq_distance(s as u8, ((s as i16) - (*delta as i16)) as u8) == 1 {
            attack |= (1 as u64).wrapping_shl(s as u32);
            if occupied & (1 as u64).wrapping_shl(s as u32) != 0 {break 'inner;}
            s = ((s as i16) + (*delta as i16)) as u8;
        }
    }
    attack
}

// Return a quick lookup table of the distance of any two pieces
// distance is in terms of squares away, not algebraic distance
fn init_distance_table() -> [[SQ; 64]; 64] {
    let mut arr: [[SQ; 64]; 64] = [[0; 64]; 64];
    for i in 0..64 as u8{
        for j in 0..64 as u8 {
            arr[i as usize][j as usize] = sq_distance(i,j);
        }
    }
    arr
}



// Returns distance of two squares
pub fn sq_distance(sq1: SQ, sq2: SQ) -> u8 {
    let x = diff(rank_idx_of_sq(sq1),rank_idx_of_sq(sq2));
    let y = diff(file_idx_of_sq(sq1),file_idx_of_sq(sq2));
    cmp::max(x,y)
}

// returns the difference between two unsigned u8s
pub fn diff(x: u8, y: u8) -> u8 {
    if x < y { y - x } else { x - y }
}






#[test]
fn test_king_mask_gen() {
    let arr = gen_king_moves().to_vec();
    let sum = arr.iter().fold(0 as  u64,|a, &b| a + (popcount64(b) as u64));
    assert_eq!(sum, (3*4) + (5 * 6 * 4) + (8 * 6 * 6));
}

#[test]
fn test_knight_mask_gen() {
    let arr = gen_knight_moves().to_vec();
    let sum = arr.iter().fold(0 as  u64,|a, &b| a + (popcount64(b) as u64));
    assert_eq!(sum, (2 * 4) + (4 * 4) + (3 * 2 * 4) + (4 * 4 * 4) + (6 * 4 * 4) + (8 * 4 * 4));
}

#[test]
fn occupancy_and_sliding() {
    let rook_deltas: [i8; 4] = [8,1,-8,-1];
    assert_eq!(popcount64(sliding_attack(&rook_deltas, 0, 0)),14);
    assert_eq!(popcount64(sliding_attack(&rook_deltas, 0, 0xFF00)),8);
    assert_eq!(popcount64(sliding_attack(&rook_deltas, 19, 0)),14);
}

#[test]
fn rmagics() {
    let mstruct = MRookTable::init();
    assert_eq!(mem::size_of_val(&mstruct), 2584);
    let bstruct = MBishopTable::init();
    assert_eq!(mem::size_of_val(&bstruct), 2584);
}


#[bench]
fn bench_rook_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.rook_moves(a,c);
            a ^ (x) }
        )
    })
}


#[bench]
fn bench_bishop_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.bishop_moves(a,c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_queen_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.queen_moves(a,c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_king_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.king_moves(c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_knight_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.knight_moves(c);
            a ^ (x) }
        )
    })
}

// Benefits from locality
#[bench]
fn bench_sequential_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.knight_moves(c);
            x ^= m.king_moves(c);
            x ^= m.bishop_moves(x,c);
            x ^= m.rook_moves(x,c);
            x ^= m.queen_moves(x,c);
            a ^ (x) }
        )
    })
}


// Stutters so Cache must be refreshed more often
#[bench]
fn bench_stutter_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.queen_moves(a,c);
            x ^= m.king_moves(c);
            x ^= m.bishop_moves(x,c);
            x ^= m.knight_moves(c);
            x ^= m.rook_moves(x,c);
            a ^ (x) }
        )
    })
}

#[bench]
// NOTE: This takes a while :/
fn bench_creation(b: &mut Bencher) {
    b.iter(|| {
        let n: u8 = test::black_box(1);
        (0..n).fold(0, |a: u64, c| {
            let m = MagicHelper::new();
            let x: u64 = m.king_moves(c);
            a ^ (x) }
        )
    })
}
#![feature(test)]
extern crate pleco;
extern crate test;

use pleco::{SQ,Player,Piece,Board};
use pleco::core::*;
use pleco::board::piece_locations::PieceLocations;

use test::{black_box, Bencher};


#[bench]
fn bench_piece_contains_default_16(b: &mut Bencher) {
    let s: PieceLocations = Board::default().get_piece_locations();
    b.iter(|| {
        black_box(
            for piece in black_box(ALL_PIECES.iter()) {
                for player in black_box(ALL_PLAYERS.iter()) {
                    black_box(&s).contains(black_box(*piece), black_box(*player));
                }
            })
    })
}

#[bench]
fn bench_piece_contains_sparse_16_even(b: &mut Bencher) {
    let mut s: PieceLocations = PieceLocations::blank();
    for x in 0..6u8 {
        for y in 0..2u8 {
            let sq = SQ(((x * 32) + (y * 5)) % 64);
            s.place(sq ,ALL_PLAYERS[y as usize], ALL_PIECES[x as usize]);
        }
    }
    b.iter(|| {
        black_box(
            for piece in black_box(ALL_PIECES.iter()) {
                for player in black_box(ALL_PLAYERS.iter()) {
                    black_box(&s).contains(black_box(*piece), black_box(*player));
                }
            })
    })
}

#[bench]
fn bench_piece_contains_singular(b: &mut Bencher) {
    let mut s: PieceLocations = PieceLocations::blank();
    let piece = Piece::P;
    let player = Player::White;
    s.place(SQ::H8, player, piece);
    b.iter(|| {
        black_box({
            black_box(&s).contains(black_box(piece), black_box(player))
        })
    })
}

#[bench]
fn bench_piece_eq(b: &mut Bencher) {
    let s: PieceLocations = Board::default().get_piece_locations();
    let s2: PieceLocations = Board::default().get_piece_locations();
    b.iter(|| {
        black_box({
            black_box(&s).eq(black_box(&s2))
        })
    })
}


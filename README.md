# Pleco

Pleco is a chess Engine inspired by Stockfish, written entirely in Rust.

This project aims to utilize the efficiency of Rust to create a Chess Bot with the speed of modern chess engines.

[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=master)](https://travis-ci.org/sfleischman105/Pleco)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=Beta-Branch)](https://travis-ci.org/sfleischman105/Pleco)

- [Documentation](https://docs.rs/pleco)

Planned & Implemented features
-------


The internal Board Implementation aims to have the following features upon completion
- [x] Bitboard Representation of Piece Locations:
- [x] Ability for concurrent Board State access, for use by parallel searchers
- [x] Full Move-generation Capabilities
- [x] Statically computed information (including Magic-Bitboards)
- [x] Zobrist Hashing
- [ ] UCI protocol implementation
- [ ] Allowing matches against Human Player



The AI Bot aims to have the following features:
- [x] Alpha-Beta pruning
- [x] Multi-threaded search with rayon.rs
- [x] Queiscience-search
- [x] MVV-LVA sorting
- [x] Iterative Deepening
- [x] Aspiration Windows
- [x] Futility Pruning
- [x] Transposition Tables
- [ ] Null Move Heuristic
- [ ] Killer Moves

#### Installation and Use

Currently, Pleco's use as a standalone program is limited in functionality. A UCI client is needed to properly interact with the program. As a recommendation, check out [Arena](http://www.playwitharena.com/).

Firstly, clone the repo and navigate into the created folder with the following commands:

```
$ git clone https://github.com/sfleischman105/Pleco --branch master
$ cd Pleco/
```
Once inside the pleco directory, build the binaries using `cargo`:
```
$ cargo build --release
```

The compiled program will appear in `./target/release/`.

Pleco can now be run with a `./Pleco` on Linux or a `./Pleco.exe` on Windows.


#### Using Pleco as a Library

To use Pleco inside your own Rust projects, [Pleco.rs is available as a library on crates.io.](https://crates.io/crates/pleco) Simply include the following in your Cargo.toml:

```rust
[dependencies]
pleco = "0.1.1"
```

and add the following to a `main.rs` or `lib.rs`:
```rust
extern crate pleco;
```

  
Contributing
-------

Any and all contributions are welcome! Open up a PR to contribute some improvements. Look at the Issues tab to see what needs some help. 


  
License
-------
Pleco is distributed under the terms of the MIT license. See LICENSE-MIT for details. Opening a pull requests is assumed to signal agreement with these licensing terms.
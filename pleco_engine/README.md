# Pleco Engine

Pleco Engine is a chess Engine inspired by Stockfish, written entirely in Rust.

This project aims to utilize the efficiency of Rust to create a Chess Bot with the speed of modern chess engines.


[![Pleco crate](https://img.shields.io/crates/v/pleco_engine.svg)](https://crates.io/crates/pleco_engine)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=master)](https://travis-ci.org/sfleischman105/Pleco)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=Beta-Branch)](https://travis-ci.org/sfleischman105/Pleco)
[![Coverage Status](https://coveralls.io/repos/github/sfleischman105/Pleco/badge.svg?branch=master)](https://coveralls.io/github/sfleischman105/Pleco?branch=master)


This project is split into two crates, `pleco_engine` (the current folder), which contains the
UCI (Universal Chess Interface) compatible Engine & AI, and `pleco`, which contains the library functionality. 

The overall goal for this project is to utilize the efficiency of Rust to create a Chess AI matching the speed of modern chess engines.

- [Documentation](https://docs.rs/pleco_engine)
- [crates.io](https://crates.io/crates/pleco_engine)

Planned & Implemented features
-------


The AI  aims to have the following features:
- [x] Alpha-Beta pruning
- [x] Multi-threaded search with rayon.rs
- [ ] Queiscience-search
- [x] MVV-LVA sorting
- [x] Iterative Deepening
- [x] Aspiration Windows
- [x] Futility Pruning
- [x] Transposition Tables
- [ ] Null Move Heuristic
- [ ] Killer Moves

Standalone Installation and Use
-------

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

  
Contributing
-------

Any and all contributions are welcome! Open up a PR to contribute some improvements. Look at the Issues tab to see what needs some help. 


  
License
-------
Pleco is distributed under the terms of the MIT license. See LICENSE-MIT for details. Opening a pull requests is assumed to signal agreement with these licensing terms.
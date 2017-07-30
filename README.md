# Pleco

[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=master)](https://travis-ci.org/sfleischman105/Pleco)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=Beta-Branch)](https://travis-ci.org/sfleischman105/Pleco)

##### Pleco is a chess Engine inspired by Stockfish, written entirely in Rust.

This project aims to utilize the efficiency of Rust to create a Chess Bot with the speed of modern chess engines.



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
- [ ] Transposition Tables
- [ ] Null Move Heuristic
- [ ] Killer Moves


  
Contributing
-------

Any and all contributions are welcome! Open up a PR to contribute some improvements.



  
License
-------
None yet.
# Pleco Engine

Pleco Engine is a Rust re-write of the [Stockfish](https://stockfishchess.org/) chess engine.

[![Pleco crate](https://img.shields.io/crates/v/pleco_engine.svg)](https://crates.io/crates/pleco_engine)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=master)](https://travis-ci.org/sfleischman105/Pleco)

This project is split into two crates, `pleco_engine` (the current folder), which contains the
UCI (Universal Chess Interface) compatible Engine & AI, and `pleco`, which contains the library functionality.

The overall goal of pleco is to recreate the Stockfish engine in rust, for comparison and
educational purposes. As such, the majority of the algorithms used here are a direct port of stockfish's, and the
credit for all of the advanced algorithms used for searching, evaluation, and many others, go directly to the
maintainers and authors of Stockfish.

- [Documentation](https://docs.rs/pleco_engine)
- [crates.io](https://crates.io/crates/pleco_engine)

## Standalone Installation and Use

Currently, Pleco's use as a standalone program is limited in functionality. A UCI client is needed to properly interact with the program.
As a recommendation, check out [Arena](http://www.playwitharena.com/).

The easiest way to use the engine would be to check out the "releases" tab,
[here](https://github.com/sfleischman105/Pleco/releases).

If you would rather build it yourself (for a specific architecture, or otherwise), clone the repo
and navigate into the created folder with the following commands:

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

## Rust Toolchain Version

Currently, `pleco_engine` requires **nightly** rust to use.

## Contributing

Any and all contributions are welcome! Open up a PR to contribute some improvements. Look at the Issues tab to see what needs some help.

## License

Pleco is distributed under the terms of the MIT license. See LICENSE-MIT for details. Opening a pull requests is assumed to signal agreement with these licensing terms.

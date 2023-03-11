<div align="center">

[![Pleco][pleco-engine-badge]][pleco-engine-link]

[![Build][build-badge]][build-link]
[![License][license-badge]][license-link]
[![Commits][commits-badge]][commits-link]

</div>

# Overview

Pleco Engine is a Rust re-write of the [Stockfish](https://stockfishchess.org/) chess engine.

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
$ git clone https://github.com/sfleischman105/Pleco --branch main
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

Pleco Engine is distributed under the GNU General Public License version 3 (or any later version at your option). See [LICENSE](LICENSE) for full details. Opening a pull requests is assumed to signal agreement with these licensing terms.

[build-link]: https://github.com/pleco-rs/Pleco/blob/main/.github/workflows/test.yml
[build-badge]: https://img.shields.io/github/actions/workflow/status/pleco-rs/Pleco/test.yml?branch=main&style=for-the-badge&label=tanton&logo=github
[license-badge]: https://img.shields.io/github/license/pleco-rs/Pleco?style=for-the-badge&label=license&color=success
[license-link]: https://github.com/pleco-rs/Pleco/blob/main/LICENSE
[commits-badge]: https://img.shields.io/github/commit-activity/m/pleco-rs/Pleco?style=for-the-badge
[commits-link]: https://github.com/pleco-rs/Pleco/commits/main
[pleco-badge]: https://img.shields.io/crates/v/pleco.svg?style=for-the-badge
[pleco-link]: https://crates.io/crates/pleco
[pleco-engine-badge]: https://img.shields.io/crates/v/pleco_engine.svg?style=for-the-badge
[pleco-engine-link]: https://crates.io/crates/pleco_engine

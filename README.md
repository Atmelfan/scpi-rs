# scpi-rs
![Quickstart](https://github.com/Atmelfan/scpi-rs/workflows/Quickstart/badge.svg)
![Fuzzing](https://github.com/Atmelfan/scpi-rs/workflows/Fuzzing/badge.svg)
[![codecov](https://codecov.io/gh/Atmelfan/scpi-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Atmelfan/scpi-rs)
[![](http://meritbadge.herokuapp.com/scpi)](https://crates.io/crates/scpi)
[![](https://img.shields.io/github/license/Atmelfan/scpi-rs)](https://img.shields.io/github/license/Atmelfan/scpi-rs)

These crates attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.

* [SCPI-1999](https://www.ivifoundation.org/downloads/SCPI/scpi-99.pdf)
* [IEEE 488.2](http://dx.doi.org/10.1109/IEEESTD.2004.95390) *Non-free standard, SCPI-1999 above repeats most of the important stuff*

## Scope
The crate does not support any transport layer, it only reads ascii-strings (`[u8]`) and writes ascii responses.

It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers/commands(optional).

## Project organisation:
 * `scpi` - SCPI/488.2 parser and command tree library.
 * `scpi-contrib` - Contribution library, provides default implementations for mandatory commands and abstractions for SCPI subsystems.
 * `scpi-derive` - Procedural macro support library, creates enums understood by Scpi (See ScpiEnum) and some internal library stuff.

## Getting started
Look at the examples in [`scpi`](https://github.com/Atmelfan/scpi-rs/tree/master/scpi/examples) or [`scpi-contrib`](https://github.com/Atmelfan/scpi-rs/tree/master/scpi-contrib/examples).

## Contribution
Contributions are welcome in the form of pull request, issues or examples are welcome.

# License
This project is licensed under the following licenses:
 * Apache version 2 - See [LICENSE-APACHE](./LICENSE-APACHE)
 * MIT - See [LICENSE-MIT](./LICENSE-MIT)

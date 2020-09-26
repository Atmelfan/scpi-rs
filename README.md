# scpi 0.3.0

![Quickstart](https://github.com/Atmelfan/scpi-rs/workflows/Quickstart/badge.svg)
![Fuzzing](https://github.com/Atmelfan/scpi-rs/workflows/Fuzzing/badge.svg)
[![codecov](https://codecov.io/gh/Atmelfan/scpi-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Atmelfan/scpi-rs)
[![](http://meritbadge.herokuapp.com/scpi)](https://crates.io/crates/scpi)
[![](https://img.shields.io/github/license/Atmelfan/scpi-rs)](https://img.shields.io/github/license/Atmelfan/scpi-rs)

This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.

* [SCPI-1999](http://www.ivifoundation.org/docs/scpi-99.pdf)
* [IEEE 488.2](http://dx.doi.org/10.1109/IEEESTD.2004.95390)

It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).

**API is unstable (as of 0.2.\*)**

## Scope
The crate does not support any transport layer, it only reads ascii-strings (`[u8]`) and writes ascii responses.

It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers/commands(optional).

## Using this crate
Add `scpi` to your dependencies:
```toml
[dependencies]
scpi = "0.x"
```
The API is still work in progress so the minor version should be specified.

## Features
These features are by default turned *OFF*.
- `extended-error` - Allows extended error messages of the form `<error code>, "error message;extended message"`.
Requires more data and program memory.
- `arbitrary-utf8-string` - Allows UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver`.
Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed. **This is not a part of the SCPI standard**
- `use_libm` - Uses libm for math operations instead of intrinsics on target which does not support them. **Use this if you get linker errors about round/roundf**

These features are by default turned **ON**.
- `build-info` - Includes build info in the library and creates a `LIBrary[:VERsion]?` command macro to query it.
- `unit-*` - Creates conversion from a argument \[and suffix] into corresponding [uom](https://crates.io/crates/uom) unit. Disable the ones you don't need to save space and skip uom.

## Getting started
Look at the [`example`](https://github.com/Atmelfan/scpi-rs/tree/master/example) for how to create a tree and run commands.

## Character coding
SCPI is strictly ASCII and will throw a error InvalidCharacter if any non-ascii `(>127)` characters are encountered (Exception: Arbitrary data blocks).
This library uses ASCII `[u8]` and not Rust UTF-8 `str`, use `to/from_bytes()` to convert in between them.

String/arbitrary-block data may be converted to str with the try_into trait which will throw a SCPI error if the data is not valid UTF8.

## Error handling
The `Context::run(...)` function aborts execution and returns on the first error it encounters.
Execution may be resumed where it aborted by calling exec again with the same tokenizer.

User commands will often use functions which may return an error, these should mostly be propagated down to the parser by rusts `?` operator.

_The documentation often uses the term 'throw' for returning an error, this should not be confused with exceptions etc which are not used._

## Limitations and differences
These are the current limitations and differences from SCPI-99 specs (that I can remember) that needs to be addressed before version 1.0.0.
They are listed in the rough order of which I care to fix them.

 * [x] Response data formatting, currently each command is responsible for formatting their response. __Done__
 * [x] Better command data operators with automatic error checking. __TryInto and TrayFrom traits are implemented for Integer, float and string types__
 * [x] Automatic suffix/special number handling. __Supports all SCPI-99 simple suffixes and decibel__
 * [x] Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands. __All IEEE488.2/SCPI-99 mandated commands have default implementations.__
 * [x] Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla"). __The parser properly handle `''` and `""` but it's up to user to handle the duplicate__
 * [x] Expression data, not handled at all. __Supports non-nested numeric-/channel-list expressions__
 * [ ] Provide a reference instrument class implementation
 * [ ] Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).
 * [x] Working test suite. __Better than nothing I suppose__

## Nice to have
Not necessary for a 1.0.0 version but would be nice to have in no particular order.

 * [x] Double-precision float (`f64`) support.

## Contribution
Contributions are welcome because I don't know what the fuck I'm doing.

Project organisation:

 * `example` - A simple example application used for testing
 * `scpi` - Main library
 * `scpi_derive` - Internal macro support library, used by `scpi` to generate error messages and suffixes (enter at own risk)


# License
This project is licensed under the MIT License, see LICENSE.txt.

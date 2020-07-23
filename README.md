# SCPI

![Quickstart](https://github.com/Atmelfan/scpi-rs/workflows/Quickstart/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Atmelfan/scpi-rs/badge.svg?branch=feature-extended-error)](https://coveralls.io/github/Atmelfan/scpi-rs?branch=feature-extended-error)

This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.
See [IVI Foundation](http://www.ivifoundation.org/specifications/default.aspx) (SCPI-99 and IEE488.2).

It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).

**API is unstable (as of 0.2.\*)**

# Scope
The crate does not support any transport layer, it only reads strings (`&[u8]`/ascii-/byte-slice/\<whatever they are called this week> to be precise) and writes responses.

It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers/commands(optional).



# Using this crate
Add `scpi` to your dependencies. The precise version should be specified as the API is unstable for now:
```
[dependencies]
scpi = "=0.x.y"
```

# Features
These features are by default turned off.
- `extended-error` - Allows extended error messages of the form `<error code>, "error message;extended message"`. 
Requires more data and program memory.
- `arbitrary-utf8-string` - Allows UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver`. 
Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed. 
                             

# Getting started
TODO, look at `example` (or `example-cortexm` for embedded) directory for now

# Character coding
SCPI is strictly ASCII and will throw a error InvalidCharacter if any non-ascii `(>127)` characters are encountered (Exception: Arbitrary data blocks). 
This library uses byte-slices for all strings and must be converted to UTF8 str type. The try_into\<str\> trait will do this automatically and throw an error if unsuccessful. 

# Error handling
The `Context::exec(...)` function aborts execution and returns on the first error it encounters. 
Execution may be resumed where it aborted by calling exec again with the same tokenizer.

User commands will often use functions which may return an error, these should mostly be propagated down to the parser by rusts `?` operator.

_The documentation often uses the term 'throw' for returning an error, this should not be confused with exceptions etc which are not used._

# Limitations and differences
These are the current limitations and differences from SCPI-99 specs (that I can remember) that needs to be addressed before version 1.0.0.
They are listed in the rough order of which I care to fix them.

 * [ ] Response data formatting, currently each command is responsible for formatting their response. _In progress_
 * [ ] Better command data operators with automatic error checking. _In progress. TryInto and TrayFrom traits are implemented for Integer, float and string types_
 * [x] ~~Automatic suffix/special number handling.~~ _Supports all SCPI-99 simple suffixes and decibel_
 * [x] ~~Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands.~~ All IEEE488.2/SCPI-99 mandated commands (and a few extra for good measure) have default implementations.
 * [ ] Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla").
 * [x] ~~Expression data, not handled at all.~~ Supports non-nested numeric-/channel-list expressions
 * [ ] Provide a reference instrument class implementation
 * [ ] Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).
 * [ ] Working test suite.
 
# Nice to have
Not necessary for a 1.0.0 version but would be nice to have in no particular order.

 * Arbitrary data block struct serializer/deserializer integration with [packed_struct](https://docs.rs/packed_struct/0.3.0/packed_struct/)
 * Support for overlapped commands using futures
 * Double-precision float (`f64`) support.

# Contribution
Contributions are welcome because I don't know what the fuck I'm doing.

Project organisation:

 * `example` - A simple example application used for testing
 * `example-cortexm` - A simple example application used for testing in a embedded environment (**Note: Read `example-cortexm/README.md` for build instructions**)
 * `scpi` - Main library
 * `scpi_derive` - Internal macro support library, used by `scpi` to generate error messages and suffixes (enter at own risk)
 * `scpi_instrument` - Support library which provides standard instrument classes
 
 # License
 This project is licensed under the MIT License, see LICENSE.txt.

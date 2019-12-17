This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.
See [IVI Foundation](http://www.ivifoundation.org/specifications/default.aspx) (SCPI-99 and IEE488.2).

It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).

**Everything is subject to change (as of 0.1.0)**

# Scope
The crate does not support any transport layer, it only reads strings (`&[u8]`/ascii-/bytestrings to be precise) and writes responses.

It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers.



# Using this crate
Add `scpi = 0.1.0` to your dependencies:
```
[dependencies]
scpi = "0.1.0"
```

# Getting started

TODO, look at `example` directory for now

# Limitations and differences
These are the current limitations and differences from SCPI-99 specs (that I can remember) that needs to be addressed before version 1.0.0.
They are listed in the rough order of which I care to fix them.

 * Response data formatting, currently each command is responsible for formatting their response.
 * Better command data operators with automatic error checking.
 * Optional mnemonics
 * Automatic suffix/special number handling
 * Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands.
 * Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla").
 * Expression data, not handled at all.
 * Provide a reference instrument class implementation
 * Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).

 * To be continued...
 
# Nice to have
Not necessary for a 1.0.0 version but would be nice to have in no particular order.

 * Arbitrary data block struct serializer/deserializer integration with [packed_struct](https://docs.rs/packed_struct/0.3.0/packed_struct/)
 * Support for overlapped commands using futures
 * Working test suite.

# Extensions
The parser extends the SCPI-99 standard with some custom syntax:

 * UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver`. Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed.

# Contribution
Contributions are welcome because I don't know what the fuck I'm doing.

Project organisation:

 * `example` - A simple example application used for testing
 * `scpi` - Main library
 * `scpi_derive` - Macro support library which helps with error messages (enter at own risk) 
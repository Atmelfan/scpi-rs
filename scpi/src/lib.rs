#![cfg_attr(not(feature = "std"), no_std)]

//! ![Quickstart](https://github.com/Atmelfan/scpi-rs/workflows/Quickstart/badge.svg)
//! ![Fuzzing](https://github.com/Atmelfan/scpi-rs/workflows/Fuzzing/badge.svg)
//! [![codecov](https://codecov.io/gh/Atmelfan/scpi-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/Atmelfan/scpi-rs)
//! [![](http://meritbadge.herokuapp.com/scpi)](https://crates.io/crates/scpi)
//! [![](https://img.shields.io/github/license/Atmelfan/scpi-rs)](https://img.shields.io/github/license/Atmelfan/scpi-rs)
//!
//! This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.
//!
//! * [SCPI-1999](http://www.ivifoundation.org/docs/scpi-99.pdf)
//! * [IEEE 488.2](http://dx.doi.org/10.1109/IEEESTD.2004.95390)
//!
//! It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).
//!
//! **API is unstable (as of 0.2.\*)**
//!
//! # Scope
//! The crate does not support any transport layer, it only reads ascii-strings (`[u8]`) and writes ascii responses.
//!
//! It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers/commands(optional).
//!
//! # Using this crate
//! Add `scpi` to your dependencies:
//! ```toml
//! [dependencies]
//! scpi = "0.x"
//! ```
//! The API is still work in progress so the minor version should be specified.
//!
//! # Features
//! These features are by default turned *OFF*.
//! - `extended-error` - Allows extended error messages of the form `<error code>, "error message;extended message"`.
//! Requires more data and program memory.
//! - `std` - Use std library, note that libm feature can be disabled with std.
//!
//! These features are by default turned **ON**.
//! - `unit-*` - Creates conversion from a argument \[and suffix] into corresponding [uom](https://crates.io/crates/uom) unit. Disable the ones you don't need to save space and skip uom.
//!
//! # Getting started
//! Look at the [`example`](https://github.com/Atmelfan/scpi-rs/tree/master/example) for how to create a tree and run commands.
//!
//! # Character coding
//! SCPI is strictly ASCII and will throw a error InvalidCharacter if any non-ascii `(>127)` characters are encountered (Exception: Arbitrary data blocks).
//! This library uses ASCII `[u8]` and not Rust UTF-8 `str`, use `to/from_bytes()` to convert in between them.
//!
//! String/arbitrary-block data may be converted to str with the try_into trait which will throw a SCPI error if the data is not valid UTF8.
//!
//! # Error handling
//! The `Context::run(...)` function aborts execution and returns on the first error it encounters.
//! Execution may be resumed where it aborted by calling exec again with the same tokenizer.
//!
//! User commands will often use functions which may return an error, these should mostly be propagated down to the parser by rusts `?` operator.
//!
//! _The documentation often uses the term 'throw' for returning an error, this should not be confused with exceptions etc which are not used._
//!
//! # Limitations and differences
//! These are the current limitations and differences from SCPI-99 specs (that I can remember) that needs to be addressed before version 1.0.0.
//! They are listed in the rough order of which I care to fix them.
//!
//!  * [x] Response data formatting, currently each command is responsible for formatting their response. __Done__
//!  * [x] Better command data operators with automatic error checking. __TryInto and TrayFrom traits are implemented for Integer, float and string types__
//!  * [x] Automatic suffix/special number handling. __Supports all SCPI-99 simple suffixes and decibel__
//!  * [x] Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands. __All IEEE488.2/SCPI-99 mandated commands have default implementations.__
//!  * [x] Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla"). __The parser properly handle `''` and `""` but it's up to user to handle the duplicate__
//!  * [x] Expression data, not handled at all. __Supports non-nested numeric-/channel-list expressions__
//!  * [ ] Provide a reference instrument class implementation
//!  * [ ] Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).
//!  * [x] Working test suite. __Better than nothing I suppose__
//!
//! # Nice to have
//! Not necessary for a 1.0.0 version but would be nice to have in no particular order.
//!
//!  * [x] Double-precision float (`f64`) support.
//!
//! # Contribution
//! Contributions are welcome because I don't know what the fuck I'm doing.
//!
//! Project organisation:
//!
//!  * `example` - A simple example application used for testing
//!  * `scpi` - Main library
//!  * `scpi_derive` - Internal macro support library, used by `scpi` to generate error messages and suffixes (enter at own risk)
//!

#[macro_use]
extern crate scpi_derive;

/* Used to create responses */
extern crate arrayvec;
extern crate lexical_core;
#[cfg(any(feature = "unit-any"))]
pub extern crate uom;

pub mod command;
pub mod error;
pub mod expression;
pub mod ieee488;
pub mod option;
pub mod parameters;
pub mod response;
pub mod scpi1999;
pub mod suffix;
pub mod tokenizer;
pub mod tree;
pub mod util;

use prelude::Node;
use response::Formatter;
// For compatibility
pub use scpi1999 as scpi;

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::command::{Command, CommandTypeMeta};
    pub use crate::error::{ArrayErrorQueue, Error, ErrorCode, ErrorQueue};
    pub use crate::response::{Data, Formatter, ResponseUnit};
    pub use crate::tokenizer::{Token, Tokenizer, Arguments};
    pub use crate::tree::Node::{self, Branch, Leaf};
    pub use crate::{
        expression::{channel_list, numeric_list},
        format,
    };
    pub use crate::{Context, Device};
    #[cfg(any(
        feature = "unit-length",
        feature = "unit-velocity",
        feature = "unit-acceleration",
        feature = "unit-electric-potential",
        feature = "unit-electric-current",
        feature = "unit-electric-conductance",
        feature = "unit-electric-resistance",
        feature = "unit-electric-charge",
        feature = "unit-electric-capacitance",
        feature = "unit-electric-inductance",
        feature = "unit-energy",
        feature = "unit-power",
        feature = "unit-angle",
        feature = "unit-amount-of-substance",
        feature = "unit-magnetic-flux",
        feature = "unit-magnetic-flux-density",
        feature = "unit-ratio",
        feature = "unit-temperature",
        feature = "unit-time",
        feature = "unit-pressure",
        feature = "unit-volume",
        feature = "unit-frequency"
    ))]
    pub use uom;
}

use crate::error::{Error, ErrorQueue, Result};
use crate::scpi::EventRegister;
use crate::tokenizer::Token;

/// Wrappers to format and discriminate SCPI types
pub mod format {

    /// Hexadecimal data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Hex<V>(pub V);

    /// Binary data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Binary<V>(pub V);

    /// Octal data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Octal<V>(pub V);

    /// Arbitrary data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Arbitrary<'a>(pub &'a [u8]);

    /// Expression data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Expression<'a>(pub &'a [u8]);

    /// Character data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Character<'a>(pub &'a [u8]);
}

/// A basic device capable of executing commands and not much else 
pub trait Device {
    fn handle_error(&mut self, err: Error);
}

/// Context in which to execute a message.
///
pub struct Context {
    /// Does output buffer contain data? 
    pub mav: bool,
}

impl Context {
    /// Create a new context
    ///
    /// # Arguments
    ///  * `device` - Device to act upon
    ///  * `writer` - Writer used to write back response messages
    ///  * `root` - SCPI command tree to use
    pub fn new() -> Self {
        Context {
            mav: false,
        }
    }
}

#![no_std]

//! This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.
//! See [IVI Foundation](http://www.ivifoundation.org/specifications/default.aspx) (SCPI-99 and IEE488.2).
//!
//! It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).
//!
//! [Documentation (docs.rs)](https://docs.rs/scpi)
//!
//! **Everything is subject to change (as of 0.1.0)**
//!
//! # Scope
//! The crate does not support any transport layer, it only reads strings (`&[u8]`/ascii-/bytestrings to be precise) and writes responses.
//!
//! It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers.
//!
//! # Using this crate
//! Add `scpi = 0.1.0` to your dependencies:
//! ```toml
//! [dependencies]
//! scpi = "0.1.0"
//! ```
//!
//! # Getting started
//!
//! TODO
//!
//! # Limitations and differences
//! These are the current limitations and differences from SCPI-99 specs (that I can remember) that needs to be addressed before version 1.0.0.
//! They are listed in the rough order of which I care to fix them.
//!
//!  * Response data formatting, currently each command is responsible for formatting their response. _In progress_
//!  * Better command data operators with automatic error checking.
//!  * ~~Optional mnemonics~~ _Done_
//!  * Automatic suffix/special number handling
//!  * Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands. _In progress_
//!  * Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla").
//!  * Expression data, not handled at all.
//!  * Provide a reference instrument class implementation
//!  * Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).
//!  * Working test suite.
//!  * To be continued...
//!
//! # Extensions
//! The parser extends the SCPI-99 standard with some custom syntax:
//!
//!  * UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver"`. Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed.
//!

#[macro_use]
extern crate scpi_derive;

/* Used to create responses */
extern crate arrayvec;
extern crate lexical_core;
extern crate arraydeque;

pub mod error;
pub mod command;
pub mod suffix;
pub mod tokenizer;
pub mod ieee488;
pub mod tree;
pub mod response;
pub mod scpi;

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::error::Error;
    pub use crate::command::Command;
    pub use crate::tree::Node;
}

use crate::error::Error;

/// A SCPI device
///
/// Use this trait and provided mandatory commands to implement the mandatory parts of SCPI for your device.
///
///
pub trait Device {

    /// Called by *CLS
    fn cls(&mut self) -> Result<(), Error>;

    /// Called by *RST
    fn rst(&mut self) -> Result<(), Error>;

    fn oper_event(&self) -> u16;

    fn oper_condition(&self) -> u16;

    fn ques_event(&self) -> u16;

    fn ques_condition(&self) -> u16;

}



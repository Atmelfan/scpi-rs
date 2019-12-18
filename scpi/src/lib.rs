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


pub mod error;
pub mod command;
pub mod suffix;
pub mod tokenizer;
pub mod ieee488;
pub mod tree;
pub mod response;

pub use ieee488::*;

use crate::error::Error;


pub trait Device {

    /// Called by *CLS command
    ///
    ///
    fn cls(&mut self) -> Result<(), Error>;

    fn rst(&mut self) -> Result<(), Error>;

    /**
     * Add an item to the error/event queue.
     * If queue is full, replace last error and return with Error::QueueOverflow
     */
    fn error_enqueue(&self, err: Error) -> Result<(), Error>;

    /**
     * Dequeue an item from the error/event queue.
     * If empty, return Error::NoError
     */
    fn error_dequeue(&self) -> Error;

    /**
     * Return current number of error/events in queue
     */
    fn error_len(&self) -> u32;

    /**
     * Clear error/event queue
     */
    fn error_clear(&self);

    fn oper_status(&self) -> u16;

    fn ques_status(&self) -> u16;

}



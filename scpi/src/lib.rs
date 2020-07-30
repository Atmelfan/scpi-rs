#![no_std]

//! This crate attempts to implement the IEE488.2 / SCPI protocol commonly used by measurement instruments and tools.
//! See [IVI Foundation](http://www.ivifoundation.org/specifications/default.aspx) (SCPI-99 and IEE488.2).
//!
//! It does not require the std library (ie it's `no_std` compatible) or a system allocator (useful for embedded).
//!
//! **API is unstable (as of 0.2.\*)**
//!
//! # Scope
//! The crate does not support any transport layer, it only reads strings (`&[u8]`/ascii-/byte-slice/\<whatever they are called this week> to be precise) and writes responses.
//!
//! It does not implement any higher level functions/error handling other than SCPI parsing and mandated registers/commands(optional).
//!
//!
//!
//! # Using this crate
//! Add `scpi` to your dependencies. The precise version should be specified as the API is unstable for now:
//! ```toml
//! [dependencies]
//! scpi = "=0.x.y"
//! ```
//!
//! # Getting started
//! TODO, look at `example` (or `example-cortexm` for embedded) directory for now
//!
//! # Character coding
//! SCPI is strictly ASCII and will throw a error InvalidCharacter if any non-ascii `(>127)` characters are encountered (Exception: Arbitrary data blocks).
//! This library uses byte-slices for all strings and must be converted to UTF8 str type. The try_into\<str\> trait will do this automatically and throw an error if unsuccessful.
//!
//! # Error handling
//! The `Context::exec(...)` function aborts execution and returns on the first error it encounters.
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
//!  *[ ] Response data formatting, currently each command is responsible for formatting their response. _In progress_
//!  *[ ] Better command data operators with automatic error checking. _In progress. TryInto and TrayFrom traits are implemented for Integer, float and string types_
//!  *[x] ~~Automatic suffix/special number handling.~~ _Supports all SCPI-99 simple suffixes and decibel_
//!  *[x] ~~Provide working implementation of all IEEE 488.2 and SCPI-99 mandated commands.~~ All IEEE488.2/SCPI-99 mandated commands (and a few extra for good measure) have default implementations.
//!  *[ ] Quotation marks inside string data, the parser cannot handle escaping `'` and `"` inside their respective block (eg "bla ""quoted"" bla").
//!  *[x] ~~Expression data, not handled at all.~~ Supports non-nested numeric-/channel-list expressions
//!  *[ ] Provide a reference instrument class implementation
//!  *[ ] Error codes returned by the parser does not follow SCPI-99 accurately (because there's a fucking lot of them!).
//!  *[ ] Working test suite.
//!
//! # Nice to have
//! Not necessary for a 1.0.0 version but would be nice to have in no particular order.
//!
//!  * Arbitrary data block struct serializer/deserializer integration with [packed_struct](https://docs.rs/packed_struct/0.3.0/packed_struct/)
//!  * Support for overlapped commands using futures
//!  * Double-precision float (`f64`) support.
//!
//! # Extensions
//! The parser extends the SCPI-99 standard with some custom syntax:
//!
//!  * UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver`. Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed.
//!
//!
//! # Contribution
//! Contributions are welcome because I don't know what the fuck I'm doing.
//!
//! Project organisation:
//!
//!  * `example` - A simple example application used for testing
//!  * `example-cortexm` - A simple example application used for testing in a embedded environment (**Note: Read `example-cortexm/README.md` for build instructions**)
//!  * `scpi` - Main library
//!  * `scpi_derive` - Internal macro support library, used by `scpi` to generate error messages and suffixes (enter at own risk)
//!  * `scpi_instrument` - Support library which provides higher level abstraction
//!
//! # License
//! This project is licensed under the MIT License, see LICENSE.txt.

#[macro_use]
extern crate scpi_derive;

/* Used to create responses */
extern crate arraydeque;
extern crate arrayvec;
extern crate lexical_core;
#[cfg(any(feature = "unit-any"))]
pub extern crate uom;

pub mod command;
pub mod error;
pub mod expression;
pub mod ieee488;
pub mod response;
pub mod scpi;
pub mod suffix;
pub mod tokenizer;
pub mod tree;

mod util;

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::command::{Command, CommandTypeMeta};
    pub use crate::error::{ArrayErrorQueue, Error, ErrorCode, ErrorQueue};
    pub use crate::response::{ArrayVecFormatter, Formatter};
    pub use crate::tokenizer::{Token, Tokenizer};
    pub use crate::tree::Node;
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
        feature = "unit-volume"
    ))]
    pub use uom;
}

use crate::error::{Error, ErrorCode, ErrorQueue, Result};
use crate::response::Formatter;
use crate::scpi::EventRegister;
use crate::tokenizer::{Token, Tokenizer};
use crate::tree::Node;

/// A SCPI device
///
/// Use this trait and provided mandatory commands to implement the mandatory parts of SCPI for your device.
///
///
pub trait Device {
    /// Called by *CLS
    fn cls(&mut self) -> Result<()>;

    /// Called by *RST
    fn rst(&mut self) -> Result<()>;

    /// Called by *TST?
    /// Return zero if self-test is successful or a positive user-error code or a
    /// standard negative error code (as a Error enum variant).
    fn tst(&mut self) -> Result<()> {
        Ok(())
    }

    /// Return true if a message is available in output queue
    fn mav(&self) -> bool {
        false
    }
}

/// Context in which to execute a message.
///
/// Contains registers related to the context and reference to the writer to respond with.
/// Also contains a reference to the Device (may be shared by multiple contexts (**Note! If threadsafe**)).
pub struct Context<'a> {
    /// SCPI command tree root
    root: &'a Node<'a>,
    /// Device executed upon
    pub device: &'a mut dyn Device,
    /// Error queue
    pub errors: &'a mut dyn ErrorQueue,
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8,

    /// OPERation:ENABle register
    pub operation: EventRegister,
    ///QUEStionable:ENABle register
    pub questionable: EventRegister,
}

impl<'a> Context<'a> {
    /// Create a new context
    ///
    /// # Arguments
    ///  * `device` - Device to act upon
    ///  * `writer` - Writer used to write back response messages
    ///  * `root` - SCPI command tree to use
    pub fn new(
        device: &'a mut dyn Device,
        errorqueue: &'a mut dyn ErrorQueue,
        root: &'a Node<'a>,
    ) -> Self {
        Context {
            device,
            errors: errorqueue,
            root,
            esr: 0,
            ese: 0,
            sre: 0,
            operation: EventRegister::new(),
            questionable: EventRegister::new(),
        }
    }

    /// Put an error into the queue and set corresponding ESR bits
    pub fn push_error(&mut self, err: Error) {
        self.esr |= err.esr_mask();
        self.errors.push_back_error(err);
    }

    /// Executes one SCPI message and queue any errors.
    ///
    /// # Arguments
    ///  * `s` - Message to execute
    ///  * `response` - A Formatter used to write a response
    ///
    /// # Returns
    ///  * `Ok(())` - If message (and all units within) was executed successfully
    ///  * `Err(error)` - If parser detected or a command returned an error
    ///
    pub fn run(&mut self, s: &[u8], response: &mut dyn Formatter) -> Result<()> {
        let mut tokenizer = Tokenizer::new(s);
        self.exec(&mut tokenizer, response)
    }

    /// Executes one SCPI message (terminated by `\n`) and queue any errors.
    ///
    /// # Arguments
    ///  * `tokenizer` - A tokenizer created from `Tokenizer::from_str(...)`. May be re-used
    ///  if still has valid tokens and Ok() was returned.
    ///
    /// # Returns
    ///  * `Ok(())` - If message (and all units within) was executed successfully
    ///  * `Err(error)` - If parser detected or a command returned an error
    ///
    pub fn exec(
        &mut self,
        tokenstream: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        response.clear();
        self.execute(tokenstream, response).map_err(|err| {
            //Set appropriate bits in ESR
            self.push_error(err);

            //Original error
            err
        })
    }

    fn execute(&mut self, tokenstream: &mut Tokenizer, response: &mut dyn Formatter) -> Result<()> {
        // Point the current branch to root
        let mut is_query = false;
        let mut is_common = false;

        let mut branch = self.root; //Node parent
        let mut node: Option<&Node> = None; //Current active node

        //Start response message
        response.message_start()?;
        'outer: while let Some(token) = tokenstream.next() {
            let tok = token?;
            //println!(">>> {:?}", tok);
            match tok {
                Token::ProgramMnemonic(_) => {
                    //Common nodes always use ROOT as base node
                    let subcommands = if is_common { self.root.sub } else { branch.sub }
                        .ok_or(ErrorCode::UndefinedHeader)?;

                    for sub in subcommands {
                        if is_common {
                            //Common nodes must match mnemonic and start with '*'
                            if sub.name.starts_with(b"*")
                                && tok.match_program_header(&sub.name[1..])
                            {
                                node = Some(sub);
                                continue 'outer;
                            }
                        } else if tok.match_program_header(sub.name) {
                            //Normal node must match mnemonic
                            node = Some(sub);
                            continue 'outer;
                        } else if sub.optional && sub.sub.is_some() {
                            //A optional node may have matching children
                            for subsub in sub.sub.unwrap() {
                                if tok.match_program_header(subsub.name) {
                                    //Normal node must match mnemonic
                                    node = Some(subsub);
                                    continue 'outer;
                                }
                            }
                        }
                    }

                    return Err(ErrorCode::UndefinedHeader.into());
                }
                Token::HeaderMnemonicSeparator => {
                    //This node will be used as branch
                    if let Some(p) = node {
                        branch = p;
                    } else {
                        branch = self.root;
                    }
                    //println!("branch={}", String::from_utf8_lossy(branch.name));
                }
                Token::HeaderCommonPrefix => {
                    is_common = true;
                }
                Token::HeaderQuerySuffix => {
                    is_query = true;
                }
                Token::ProgramMessageTerminator => {
                    //
                    break 'outer;
                }
                Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator => {
                    // Execute header if available
                    if let Some(n) = node {
                        if tok == Token::ProgramHeaderSeparator {
                            //If a header separator was found, pass tokenizer for arguments and
                            // check that the command consumes them all
                            n.exec(self, tokenstream, response, is_query)?;

                            // Should have a terminator, unit terminator or END after arguments
                            // If, not, the handler has not consumed all arguments (error) or an unexpected token appeared.'
                            // TODO: This should abort above command!
                            if tokenstream.next_data(true)?.is_some() {
                                return Err(ErrorCode::ParameterNotAllowed.into());
                            }
                        } else {
                            //No header separator was found = no arguments, pass an empty tokenizer
                            n.exec(self, &mut Tokenizer::empty(), response, is_query)?;
                        }
                    } else {
                        //return Err(Error::CommandHeaderError);
                    }

                    // Reset unit state
                    node = None;
                    is_query = false;
                    is_common = false;
                }
                _ => (),
            }
        }

        //Execute last command if any (never has arguments or would've been executed earlier)
        if let Some(n) = node {
            n.exec(self, &mut Tokenizer::empty(), response, is_query)?;
        }

        //End response message if anything has written something
        if !response.is_empty() {
            response.message_end()?;
        }

        Ok(())
        //branch.exec(self, tokenstream.clone().borrow_mut(), is_query)?;
    }

    pub fn get_stb(&self) -> u8 {
        let mut reg = 0u8;
        //Set OPERation status bits
        if self.operation.get_summary() {
            reg |= 0x80;
        }
        //Set QUEStionable status bit
        if self.questionable.get_summary() {
            reg |= 0x08;
        }
        //Set error queue empty bit
        if !self.errors.is_empty() {
            reg |= 0x04;
        }
        //Set event bit
        if self.esr & self.ese != 0 {
            reg |= 0x20;
        }
        //Set MSS bit
        if reg & self.sre != 0 {
            reg |= 0x40;
        }
        //Set MAV bit
        if self.device.mav() {
            reg |= 0x10;
        }

        reg
    }

    pub fn set_ese(&mut self, ese: u8) {
        self.ese = ese;
    }

    pub fn get_ese(&mut self) -> u8 {
        self.ese
    }
}

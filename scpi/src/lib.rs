#![no_std]

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
//! - `arbitrary-utf8-string` - Allows UTF8 arbitrary data block, `#s"Detta är en utf8 sträng med roliga bokstäver`.
//! Checked by the parser and emits a InvalidBlockData if the UTF8 data is malformed. **This is not a part of the SCPI standard**
//! - `use_libm` - Uses libm for math operations instead of intrinsics on target which does not support them. **Use this if you get linker errors about round/roundf**
//!
//! These features are by default turned **ON**.
//! - `build-info` - Includes build info in the library and creates a `LIBrary[:VERsion]?` command macro to query it.
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
extern crate arraydeque;
extern crate arrayvec;
extern crate lexical_core;
#[cfg(feature = "use_libm")]
extern crate libm;
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

#[cfg(feature = "build-info")]
pub mod info {
    use crate::error::Result;
    use crate::format::Character;
    use crate::prelude::*;
    use crate::qonly;

    /// Build information
    pub mod built {
        include!(concat!(env!("OUT_DIR"), "/built.rs"));
    }

    /// # `LIBrary[:VERSion]?`
    ///
    /// Returns the library and rustc version ast string data
    /// If available, git tag and hash are included as string data,
    /// if git repo was dirty a character data flag DIRTY is appended.
    ///
    /// Example `"0.3.0","rustc 1.47.0-nightly (6c8927b0c 2020-07-26)","85b43cf","85b43cfa0319e85cff726e2716beca50859a4ef4",DIRTY`
    pub struct LibVersionCommand {}

    impl Command for LibVersionCommand {
        qonly!();

        fn query(
            &self,
            _context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut ResponseUnit,
        ) -> Result<()> {
            response
                .data(built::PKG_VERSION.as_bytes())
                .data(built::RUSTC_VERSION.as_bytes());

            if let (Some(v), Some(dirty), Some(hash)) =
                (built::GIT_VERSION, built::GIT_DIRTY, built::GIT_COMMIT_HASH)
            {
                response.data(v.as_bytes()).data(hash.as_bytes());
                if dirty {
                    response.data(Character(b"DIRTY"));
                }
            }
            response.finish()
        }
    }

    /// Creates a `LIBrary:VERSion` command
    #[macro_export]
    macro_rules! scpi_crate_version {
        () => {
            Node {
                name: b"LIBrary",
                optional: false,
                handler: None,
                sub: &[Node {
                    name: b"VERSion",
                    optional: true,
                    handler: Some(&scpi::info::LibVersionCommand {}),
                    sub: &[],
                }],
            }
        };
    }
}

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::command::{Command, CommandTypeMeta};
    pub use crate::error::{ArrayErrorQueue, Error, ErrorCode, ErrorQueue};
    pub use crate::response::{ArrayVecFormatter, Data, Formatter, ResponseUnit};
    pub use crate::tokenizer::{Token, Tokenizer};
    pub use crate::tree::Node;
    pub use crate::NumericValues;
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

use crate::error::{Error, ErrorCode, ErrorQueue, Result};
use crate::response::Formatter;
use crate::scpi::EventRegister;
use crate::tokenizer::{Token, Tokenizer};
use crate::tree::Node;

/// Wrappers to format and discriminate SCPI types
pub mod format {

    /// Hexadecimal data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Hex<V>(pub V);

    /// Binary data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Binary<V>(pub V);

    /// Octal data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Octal<V>(pub V);

    /// Arbitrary data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Arbitrary<'a>(pub &'a [u8]);

    /// Expression data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Expression<'a>(pub &'a [u8]);

    /// Character data
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct Character<'a>(pub &'a [u8]);
}

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
    /// Return Ok(()) if self-test is successful or a positive user-error code or a
    /// standard negative error code (as a Error enum variant).
    fn tst(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called by STATus:PRESet
    /// Preset any device-specific status.
    fn preset(&mut self) -> Result<()> {
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
    pub fn run<FMT>(&mut self, s: &[u8], response: &mut FMT) -> Result<()>
    where
        FMT: Formatter,
    {
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
    pub fn exec<FMT>(&mut self, tokenstream: &mut Tokenizer, response: &mut FMT) -> Result<()>
    where
        FMT: Formatter,
    {
        response.clear();
        self.execute(tokenstream, response).map_err(|err| {
            //Set appropriate bits in ESR
            self.push_error(err);

            //Original error
            err
        })
    }

    fn execute<FMT>(&mut self, tokenstream: &mut Tokenizer, response: &mut FMT) -> Result<()>
    where
        FMT: Formatter,
    {
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
                    let subcommands = if is_common { self.root.sub } else { branch.sub };
                    if subcommands.is_empty() {
                        return Err(ErrorCode::UndefinedHeader.into());
                    }

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
                        } else if sub.optional && !sub.sub.is_empty() {
                            //A optional node may have matching children
                            for subsub in sub.sub {
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

/// Numeric values that can be substituted for `<numeric>`
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NumericValues<'a> {
    /// `MAXimum`
    Maximum,
    /// `MINimum`
    Minimum,
    /// `DEFault`
    Default,
    /// `UP`
    Up,
    /// `DOWN`
    Down,
    /// `AUTO`
    Auto,
    /// Number
    Numeric(Token<'a>),
}

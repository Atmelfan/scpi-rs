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
#![doc = document_features::document_features!()]
//!
//! # Getting started
//! Look at the [`example`](https://github.com/Atmelfan/scpi-rs/tree/master/example) for how to create a tree and run commands.
//!
//! Here's a good resource general SCPI style and good practices: `<https://www.keysight.com/us/en/assets/9921-01873/miscellaneous/SCPITrainingSlides.pdf>`
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

#[cfg(feature = "std")]
extern crate std as alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

use crate::error::Error;
use core::any::Any;

pub mod error;
pub mod option;
pub mod parser;
pub mod tree;

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::{
        error::{Error, ErrorCode},
        Context, Device,
    };
}

/// Re-export uom if enabled
#[cfg(feature = "uom")]
pub mod units {
    #[doc(no_inline)]
    pub use uom;

    #[cfg(feature = "unit-angle")]
    pub use uom::si::f32::Angle;
    #[cfg(feature = "unit-capacitance")]
    pub use uom::si::f32::Capacitance;
    #[cfg(feature = "unit-electric-charge")]
    pub use uom::si::f32::ElectricCharge;
    #[cfg(feature = "unit-electric-current")]
    pub use uom::si::f32::ElectricCurrent;
    #[cfg(feature = "unit-electric-potential")]
    pub use uom::si::f32::ElectricPotential;
    #[cfg(feature = "unit-electrical-conductance")]
    pub use uom::si::f32::ElectricalConductance;
    #[cfg(feature = "unit-electrical-resistance")]
    pub use uom::si::f32::ElectricalResistance;
    #[cfg(feature = "unit-energy")]
    pub use uom::si::f32::Energy;
    #[cfg(feature = "unit-frequency")]
    pub use uom::si::f32::Frequency;
    #[cfg(feature = "unit-inductance")]
    pub use uom::si::f32::Inductance;
    #[cfg(feature = "unit-power")]
    pub use uom::si::f32::Power;
    #[cfg(feature = "unit-ratio")]
    pub use uom::si::f32::Ratio;
    #[cfg(feature = "unit-thermodynamic-temperature")]
    pub use uom::si::f32::ThermodynamicTemperature;
    #[cfg(feature = "unit-time")]
    pub use uom::si::f32::Time;
}
pub use arrayvec;

/// A basic device capable of executing commands and not much else
pub trait Device {
    fn handle_error(&mut self, err: Error);
}

/// Context in which to execute a message.
///
#[derive(Debug)]
pub struct Context<'a> {
    /// Does output buffer contain data?
    pub mav: bool,

    /// User context data.
    ///
    /// **Do not use this to pass application data!**
    /// Use traits instead. It's only intended to pass along data from whatever context is running a command.
    ///
    /// For example: User authentication information if the call comes from an authenticated interface.
    pub user: &'a dyn Any,
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    /// Create a new context
    pub fn new() -> Self {
        Context {
            mav: false,
            user: &(),
        }
    }

    // Create a new context with user data
    pub fn new_with_user(user: &'a dyn Any) -> Self {
        Context { mav: false, user }
    }

    /// Get user context data.
    ///
    /// **DO NOT USE FOR APPLICATION DATA**
    pub fn user<U: Any>(&'a self) -> Option<&'a U> {
        self.user.downcast_ref()
    }

    pub fn mav(&self) -> bool {
        self.mav
    }
}

#[cfg(test)]
mod tests {
    macro_rules! fixture_device {
        ($dev:ident) => {
            impl $crate::Device for $dev {
                fn handle_error(&mut self, _err: $crate::error::Error) {}
            }
        };
    }
    pub(crate) use fixture_device;
}

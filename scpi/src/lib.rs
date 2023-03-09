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
//!
//! # Scope
//! The crate does not support any transport layer, it only reads ascii-strings (`[u8]`) and writes ascii responses.
//!
//! It does not implement any higher level functions/error handling other than SCPI parsing and response generation.
//! See [scpi-contrib](https://crates.io/crates/scpi) for higher level abstractions.
//!
//! # Using this crate
//! Add `scpi` to your dependencies:
//! ```toml
//! [dependencies]
//! scpi = "1.0"
//! ```
//!
//! # Features
#![doc = document_features::document_features!()]
//! (See rustdoc/docs.rs for available features)
//!
//! # Getting started
//! Look at the [`example`](https://github.com/Atmelfan/scpi-rs/tree/master/example) for how to create a tree and run commands.
//!
//! Here's a good resource general SCPI style and good practices: [Keysight SCPI Training slides](https://www.keysight.com/us/en/assets/9921-01873/miscellaneous/SCPITrainingSlides.pdf)
//!
//! # Character coding
//! SCPI is strictly ASCII and will throw a error InvalidCharacter if any non-ascii `(>127)` characters are encountered (Exception: Arbitrary data blocks).
//!
//! String parameters and reponse data should use byte-slices (`&[u8]`) with valid ASCII data.
//!
//! The str type can be decoded from either a string parameter or arbitrary block and will automatically be checked for UTF8 encoding.
//! When used as response data a str will always return an arbitrary block.
//!
//! # Error handling
//! The `Node::run(...)` function aborts execution and returns on the first error it encounters.
//!
//! User commands will often use functions which may return an error, these should mostly be propagated down to the parser by rusts `?` operator.
//!
//!
//! # Limitations and differences
//! * Overlapping commands are not supported, [Github issue](https://github.com/Atmelfan/scpi-rs/issues/23).
//!
//! # Contribution
//! Contributions are welcome.
//!
//! # Project organisation:
//!  * `scpi` - Core parser crate
//!  * `scpi-contrib` - Mandatory command implementations and higher level abstractions
//!  * `scpi-derive` - Macro derive library, manly used to generate enum type parameters (see [option::ScpiEnum]).
//!

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

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

/// Re-export supported uom types if enabled
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

/// A basic device capable of executing commands and not much else
pub trait Device {
    /// Called when the parser encounters a syntax error or a command handler returns an error.
    fn handle_error(&mut self, err: Error);
}

/// Context in which to execute a message.
///
/// Useful when multiple sources can execute commands.
#[derive(Debug)]
pub struct Context<'a> {
    /// Does output buffer contain data?
    pub mav: bool,

    /// User context data.
    ///
    /// **Do not use this to pass application data!**
    /// Use traits instead. It's only intended to pass along data from whatever context is running a command.
    ///
    /// For example: User authentication information if the call comes from an authenticated interface
    /// or port number if the call comes from a serial port.
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

    /// Returns true if output buffer contains data
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

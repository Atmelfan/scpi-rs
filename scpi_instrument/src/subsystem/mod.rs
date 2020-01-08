//! This module seeks to simplify implementation of standard subsystems and device classes by defining helper
//! structs/traits etc and providing default command implementations.
//!
//! **Note:** Standard implementations of mandated `SYSTem|STATus` subsystems are included in the scpi crate.
//! This crate may expand these subsystems.

pub mod measure;
pub mod calculate;
pub mod calibrate;
pub mod control;
pub mod diagnostic;
pub mod display;
pub mod format;
pub mod hcopy;
pub mod input;
pub mod instrument;
pub mod memory;
pub mod mmemory;
pub mod output;
pub mod program;
pub mod route;
pub mod sense;
pub mod source;
pub mod status;
pub mod system;
pub mod test;
pub mod trace;
pub mod trigger;
pub mod unit;
pub mod vxi;
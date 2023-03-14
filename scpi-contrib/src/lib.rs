//! This crate aims to implement higher level abstraction for the SCPI protocol. See [scpi] crate for the basic command parser.  
//!
//! It does not require the std library (i.e. it's `no_std` compatible) or a system allocator (useful for embedded).
//!
//!
//! # Using this crate
//! Add `scpi` and `scpi-contrib` to your dependencies:
//! ```toml
//! [dependencies]
//! scpi = "1.0"
//! scpi-contrib = "1.0"
//! ```
//!
//! # Features
#![doc = document_features::document_features!()]
//! (See rustdoc/docs.rs for available features)
//!
//! # References
//! 1. [IEEE488.2](https://standards.ieee.org/ieee/488.2/718/)
//! 2. [SCPI-99](https://www.ivifoundation.org/docs/scpi-99.pdf)

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod ieee488;
pub mod scpi1999;

/// Standard SCPI instrument classes
#[cfg(feature = "unproven")]
pub mod classes;

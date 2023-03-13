//!
//!
//!
//!
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

#[cfg(feature = "unproven")]
pub mod classes;

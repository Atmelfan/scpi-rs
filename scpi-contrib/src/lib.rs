//!
//! 
//! 
//! 
//! 
//! # References
//! 1. [IEEE488.2](https://standards.ieee.org/ieee/488.2/718/)
//! 2. [SCPI-99](https://www.ivifoundation.org/docs/scpi-99.pdf)

#![cfg_attr(not(feature = "std"), no_std)]

pub mod scpi1999;
pub mod ieee488;

pub mod classes;

pub use scpi1999::*;
pub use ieee488::*;



/// [SCPI-99](crate#references)
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

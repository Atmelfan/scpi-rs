#![cfg_attr(not(feature = "std"), no_std)]

pub mod scpi1999;
pub mod ieee488;

pub mod classes;

pub use scpi1999::*;
pub use ieee488::*;

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

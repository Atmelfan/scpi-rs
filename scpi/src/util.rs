use core::slice::Iter;

#[cfg(not(feature = "use_libm"))]
#[allow(unused_imports)]
use lexical_core::Float;
#[cfg(feature = "use_libm")]
use libm;

/// Skip continuous digits
///
pub(crate) fn skip_digits(iter: &mut Iter<u8>) -> bool {
    let mut any = false;
    while let Some(digit) = iter.clone().next() {
        if !digit.is_ascii_digit() {
            break;
        }
        any = true;
        iter.next().unwrap();
    }
    any
}

/// Skip one positive or negative sign if any
pub(crate) fn skip_sign(iter: &mut Iter<u8>) {
    if let Some(sign) = iter.clone().next() {
        if *sign == b'+' || *sign == b'-' {
            iter.next().unwrap();
        }
    }
}

/// Skip continuous whitespace
pub(crate) fn skip_ws(iter: &mut Iter<u8>) {
    while iter
        .clone()
        .next()
        .map_or(false, |ch| ch.is_ascii_whitespace())
    {
        iter.next().unwrap();
    }
}

/// Split a mnemonic of the form "ABC123" into ("ABC", "123")
/// Returns None if the mnemonic does not end with digits.
pub(crate) fn mnemonic_split_index(mnemonic: &[u8]) -> Option<(&[u8], &[u8])> {
    let last = mnemonic.iter().rposition(|p| !p.is_ascii_digit());

    if let Some(index) = last {
        if index == mnemonic.len() - 1 {
            None
        } else {
            Some(mnemonic.split_at(index + 1))
        }
    } else {
        None
    }
}

/// Compare a string to a mnemonic
/// # Arguments
/// * `mnemonic` - Reference mnemonic to compare with (Example `TRIGger2`)
/// * `s` - String to compare to mnemonic
///
pub fn mnemonic_compare(mnemonic: &[u8], s: &[u8]) -> bool {
    //LONGform == longform || LONG == long
    //TODO: This sucks.
    let mut optional = true;
    mnemonic.len() >= s.len() && {
        let mut s_iter = s.iter();
        mnemonic.iter().all(|m| {
            let x = s_iter.next();
            if m.is_ascii_lowercase() && x.is_some() {
                optional = false;
            }
            x.map_or(
                !(m.is_ascii_uppercase() || m.is_ascii_digit()) && optional,
                |x| m.eq_ignore_ascii_case(x),
            )
        })
    }
}

///
///
pub(crate) fn ascii_to_digit(digit: u8, radix: u8) -> Option<u32> {
    let lowercase = digit.to_ascii_lowercase();

    if digit.is_ascii_digit() && digit - b'0' < radix {
        Some((digit - b'0') as u32)
    } else if radix > 10 && lowercase.is_ascii_alphabetic() && lowercase - b'a' < radix - 10 {
        Some((lowercase - b'a' + 10) as u32)
    } else {
        None
    }
}

pub(crate) fn roundf32(x: f32) -> f32 {
    #[cfg(feature = "use_libm")]
    {
        libm::roundf(x)
    }
    #[cfg(not(feature = "use_libm"))]
    {
        <f32>::round(x)
    }
}

pub(crate) fn roundf64(x: f64) -> f64 {
    #[cfg(feature = "use_libm")]
    {
        libm::round(x)
    }
    #[cfg(not(feature = "use_libm"))]
    {
        <f64>::round(x)
    }
}

use core::slice::Iter;

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

pub fn mnemonic_match(mnemonic: &[u8], s: &[u8]) -> bool {
    mnemonic_compare(mnemonic, s)
        || match (mnemonic_split_index(mnemonic), mnemonic_split_index(s)) {
            // ABC, ABC
            (None, None) => false,
            // ABC1, ABC
            (Some((m, index)), None) => mnemonic_compare(m, s) && index == b"1",
            // ABC, ABC1
            (None, Some((x, index))) => mnemonic_compare(mnemonic, x) && index == b"1",
            //ABCn, ABCn
            (Some((m, index1)), Some((x, index2))) => mnemonic_compare(m, x) && (index1 == index2),
        }
}

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

use core::slice::Iter;

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

pub(crate) fn skip_sign(iter: &mut Iter<u8>) {
    if let Some(sign) = iter.clone().next() {
        if *sign == b'+' || *sign == b'-' {
            iter.next().unwrap();
        }
    }
}

pub(crate) fn skip_ws(iter: &mut Iter<u8>) {
    while iter
        .clone()
        .next()
        .map_or(false, |ch| ch.is_ascii_whitespace())
    {
        iter.next().unwrap();
    }
}

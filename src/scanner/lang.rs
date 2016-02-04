/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Implementations of `ScanFromStr` for primitive language types.
*/
use itertools::Itertools;
use strcursor::StrCursor;
use ::ScanError;
use ::input::ScanInput;
use super::ScanFromStr;
use super::misc::Word;

parse_scanner! { impl<'a> for bool, from Word, err desc "expected `true` or `false`" }

#[cfg(test)]
#[test]
fn test_scan_bool() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(<bool>::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("y"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("n"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("yes"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("no"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from(" "), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from(" true"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from(" false"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("true"), Ok((true, 4)));
    assert_match!(<bool>::scan_from("false"), Ok((false, 5)));
    assert_match!(<bool>::scan_from("True"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<bool>::scan_from("False"), Err(SE { kind: SEK::Syntax(_), .. }));
}

impl<'a> ScanFromStr<'a> for char {
    type Output = char;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let cur = try!(StrCursor::new_at_start(s.as_str()).at_next_cp()
            .ok_or(ScanError::syntax("expected a character")));
        Ok((cur.cp_before().unwrap(), cur.byte_pos()))
    }
}

#[cfg(test)]
#[test]
fn test_scan_char() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(<char>::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<char>::scan_from(" "), Ok((' ', 1)));
    assert_match!(<char>::scan_from("x"), Ok(('x', 1)));
    assert_match!(<char>::scan_from("xy"), Ok(('x', 1)));
    assert_match!(<char>::scan_from("é"), Ok(('e', 1)));
    assert_match!(<char>::scan_from("é"), Ok(('é', 2)));
    assert_match!(<char>::scan_from("字"), Ok(('字', 3)));
}

parse_scanner! { impl<'a> for f32, matcher match_float, matcher err "expected floating point number", err map ScanError::float }
parse_scanner! { impl<'a> for f64, matcher match_float, matcher err "expected floating point number", err map ScanError::float }

fn match_float(s: &str) -> Option<((usize, usize), usize)> {
    use std::iter::Peekable;

    // First, check for one of the named constants.
    if s.starts_with("inf") {
        if s[3..].chars().next().map(|c| !c.is_alphabetic()).unwrap_or(true) {
            return Some(((0, 3), 3));
        }
    }

    if s.starts_with("-inf") {
        if s[4..].chars().next().map(|c| !c.is_alphabetic()).unwrap_or(true) {
            return Some(((0, 4), 4));
        }
    }

    if s.starts_with("NaN") {
        if s[3..].chars().next().map(|c| !c.is_alphabetic()).unwrap_or(true) {
            return Some(((0, 3), 3));
        }
    }

    // Ok, try scanning an actual number.
    let mut ibs = s.bytes().enumerate().peekable();

    match ibs.peek().map(|&(_, b)| b) {
        Some(b'-') | Some(b'+') => { ibs.next(); },
        _ => ()
    }

    // Skip over leading integer part.
    println!("before: {:?}", ibs.peek());
    let _ = (&mut ibs)
        .take_while_ref(|&(_, b)| matches!(b, b'0'...b'9'))
        .count();
    println!("after:  {:?}", ibs.peek());

    // At this point, we *must* get *either* a decimal point *or* an "e".
    fn match_exp<I: Iterator<Item=(usize, u8)>>(mut ibs: Peekable<I>)
    -> Option<((usize, usize), usize)> {

        match ibs.peek().map(|&(_, b)| b) {
            Some(b'-') | Some(b'+') => { ibs.next(); },
            _ => ()
        }

        ibs.take_while(|&(_, b)| matches!(b, b'0'...b'9'))
            .last()
            .map(|(i, _)| i + 1)
            .map(|n| ((0, n), n))
    }

    match ibs.next() {
        Some((i, b'.')) => {
            // There *might* be another sequence of digits.
            let end = (&mut ibs)
                .take_while_ref(|&(_, b)| matches!(b, b'0'...b'9'))
                .map(|(i, _)| i + 1)
                .last()
                .unwrap_or(i + 1);
            
            // Finally, there *might* be an exponent
            match ibs.peek().map(|&(_, b)| b) {
                Some(b'e') | Some(b'E') => {
                    ibs.next();
                    match_exp(ibs)
                },
                _ => Some(((0, end), end))
            }
        },

        Some((_, b'e')) | Some((_, b'E')) => match_exp(ibs),

        _ => None
    }
}

#[cfg(test)]
#[test]
fn test_scan_f64() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    macro_rules! check_f64 {
        ($f:expr) => {
            assert_match!(
                <f64>::scan_from(stringify!($f)),
                Ok(($f, n)) if n == stringify!($f).len()
            );
            assert_match!(
                <f64>::scan_from(concat!("-", stringify!($f))),
                Ok((-$f, n)) if n == concat!("-", stringify!($f)).len()
            );
        };
    }

    assert_match!(<f64>::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<f64>::scan_from("-"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<f64>::scan_from("+"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<f64>::scan_from("x"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<f64>::scan_from(" "), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<f64>::scan_from(" 0"), Err(SE { kind: SEK::Syntax(_), .. }));

    assert_match!(<f64>::scan_from("inf"), Ok((::std::f64::INFINITY, 3)));
    assert_match!(<f64>::scan_from("-inf"), Ok((::std::f64::NEG_INFINITY, 4)));
    assert_match!(<f64>::scan_from("NaN"), Ok((v, 3)) if v.is_nan());

    check_f64!(0.0);
    check_f64!(1.0);
    check_f64!(0.1);
    check_f64!(12345.);
    check_f64!(0.12345);
    check_f64!(101e-33);
    check_f64!(1e23);
    check_f64!(2075e23);
    check_f64!(8713e-23);
    check_f64!(1e-325);
    check_f64!(1e-326);
    check_f64!(1e-500);
    check_f64!(1.448997445238699);
}

#[cfg(f64_debug_is_roundtrip_accurate)]
#[cfg(test)]
#[test]
fn test_scan_f64_debug_is_roundtrip_accurate() {
    macro_rules! check_f64 {
        ($f:expr) => {
            assert_match!(
                <f64>::scan_from(stringify!($f)),
                Ok(($f, n)) if n == stringify!($f).len()
            );
            assert_match!(
                <f64>::scan_from(concat!("-", stringify!($f))),
                Ok((-$f, n)) if n == concat!("-", stringify!($f)).len()
            );
        };
    }

    check_f64!(3e-5);
    check_f64!(12345.67890);
    check_f64!(2.2250738585072014e-308);
    check_f64!(1e300);
    check_f64!(123456789.34567e250);
    check_f64!(5e-324);
    check_f64!(91e-324);
    check_f64!(1e-322);
    check_f64!(13245643e-320);
    check_f64!(2.22507385851e-308);
    check_f64!(2.1e-308);
    check_f64!(4.9406564584124654e-324);
}

parse_scanner! { impl<'a> for i8, matcher match_sinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for i16, matcher match_sinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for i32, matcher match_sinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for i64, matcher match_sinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for isize, matcher match_sinteger, matcher err "expected integer", err map ScanError::int }

parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for i8, matcher match_bin_int, matcher err "expected binary integer", map |s| i8::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for i16, matcher match_bin_int, matcher err "expected binary integer", map |s| i16::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for i32, matcher match_bin_int, matcher err "expected binary integer", map |s| i32::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for i64, matcher match_bin_int, matcher err "expected binary integer", map |s| i64::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for isize, matcher match_bin_int, matcher err "expected binary integer", map |s| isize::from_str_radix(s, 2), err map ScanError::int }

parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for i8, matcher match_oct_int, matcher err "expected octal integer", map |s| i8::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for i16, matcher match_oct_int, matcher err "expected octal integer", map |s| i16::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for i32, matcher match_oct_int, matcher err "expected octal integer", map |s| i32::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for i64, matcher match_oct_int, matcher err "expected octal integer", map |s| i64::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for isize, matcher match_oct_int, matcher err "expected octal integer", map |s| isize::from_str_radix(s, 8), err map ScanError::int }

parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for i8, matcher match_hex_int, matcher err "expected hex integer", map |s| i8::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for i16, matcher match_hex_int, matcher err "expected hex integer", map |s| i16::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for i32, matcher match_hex_int, matcher err "expected hex integer", map |s| i32::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for i64, matcher match_hex_int, matcher err "expected hex integer", map |s| i64::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for isize, matcher match_hex_int, matcher err "expected hex integer", map |s| isize::from_str_radix(s, 16), err map ScanError::int }

#[cfg(test)]
#[test]
fn test_scan_i32() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(<i32>::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from("-"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from("+"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from("x"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from(" "), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from(" 0"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<i32>::scan_from("0"), Ok((0, 1)));
    assert_match!(<i32>::scan_from("42"), Ok((42, 2)));
    assert_match!(<i32>::scan_from("-312"), Ok((-312, 4)));
    assert_match!(<i32>::scan_from("1_234"), Ok((1, 1)));
}

parse_scanner! { impl<'a> for u8, matcher match_uinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for u16, matcher match_uinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for u32, matcher match_uinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for u64, matcher match_uinteger, matcher err "expected integer", err map ScanError::int }
parse_scanner! { impl<'a> for usize, matcher match_uinteger, matcher err "expected integer", err map ScanError::int }

parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for u8, matcher match_bin_int, matcher err "expected binary integer", map |s| u8::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for u16, matcher match_bin_int, matcher err "expected binary integer", map |s| u16::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for u32, matcher match_bin_int, matcher err "expected binary integer", map |s| u32::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for u64, matcher match_bin_int, matcher err "expected binary integer", map |s| u64::from_str_radix(s, 2), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromBinary::scan_from_binary for usize, matcher match_bin_int, matcher err "expected binary integer", map |s| usize::from_str_radix(s, 2), err map ScanError::int }

parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for u8, matcher match_oct_int, matcher err "expected octal integer", map |s| u8::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for u16, matcher match_oct_int, matcher err "expected octal integer", map |s| u16::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for u32, matcher match_oct_int, matcher err "expected octal integer", map |s| u32::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for u64, matcher match_oct_int, matcher err "expected octal integer", map |s| u64::from_str_radix(s, 8), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromOctal::scan_from_octal for usize, matcher match_oct_int, matcher err "expected octal integer", map |s| usize::from_str_radix(s, 8), err map ScanError::int }

parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for u8, matcher match_hex_int, matcher err "expected hex integer", map |s| u8::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for u16, matcher match_hex_int, matcher err "expected hex integer", map |s| u16::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for u32, matcher match_hex_int, matcher err "expected hex integer", map |s| u32::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for u64, matcher match_hex_int, matcher err "expected hex integer", map |s| u64::from_str_radix(s, 16), err map ScanError::int }
parse_scanner! { impl<'a> ScanFromHex::scan_from_hex for usize, matcher match_hex_int, matcher err "expected hex integer", map |s| usize::from_str_radix(s, 16), err map ScanError::int }

#[cfg(test)]
#[test]
fn test_scan_u32() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(<u32>::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from("-"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from("+"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from("x"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from(" "), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from(" 0"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from("0"), Ok((0, 1)));
    assert_match!(<u32>::scan_from("42"), Ok((42, 2)));
    assert_match!(<u32>::scan_from("-312"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(<u32>::scan_from("1_234"), Ok((1, 1)));
}

fn match_bin_int(s: &str) -> Option<((usize, usize), usize)> {
    s.bytes().enumerate()
        .take_while(|&(_, b)| matches!(b, b'0' | b'1'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

fn match_hex_int(s: &str) -> Option<((usize, usize), usize)> {
    s.bytes().enumerate()
        .take_while(|&(_, b)|
            matches!(b, b'0'...b'9' | b'a'...b'f' | b'A'...b'F'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

fn match_oct_int(s: &str) -> Option<((usize, usize), usize)> {
    s.bytes().enumerate()
        .take_while(|&(_, b)| matches!(b, b'0'...b'7'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

fn match_sinteger(s: &str) -> Option<((usize, usize), usize)> {
    let mut ibs = s.bytes().enumerate().peekable();

    match ibs.peek().map(|&(_, b)| b) {
        Some(b'-') | Some(b'+') => { ibs.next(); },
        _ => (),
    }

    ibs.take_while(|&(_, b)| matches!(b, b'0'...b'9'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

fn match_uinteger(s: &str) -> Option<((usize, usize), usize)> {
    let mut ibs = s.bytes().enumerate().peekable();

    match ibs.peek().map(|&(_, b)| b) {
        Some(b'+') => { ibs.next(); },
        _ => (),
    }

    ibs.take_while(|&(_, b)| matches!(b, b'0'...b'9'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

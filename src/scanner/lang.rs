use regex::Regex;
use strcursor::StrCursor;
use ::ScanErrorKind;
use super::ScanFromStr;
use super::misc::Word;

lazy_static! {
    static ref FLOAT_RE: Regex = Regex::new(r#"(?x)
        ^(
              inf
            | -inf
            | NaN
            | -? (
                  \d+ \. \d+ [eE] -? \d+
                | \d+ \. \d+
                | \d+ [eE] -? \d+
                | \d+ \.
            )
        )
    "#).unwrap();
    static ref SINTEGER_RE: Regex = Regex::new(r"^[+-]?\d+").unwrap();
    static ref UINTEGER_RE: Regex = Regex::new(r"^[+]?\d+").unwrap();
}

parse_scanner! { impl<'a> for bool, from &str }

#[cfg(test)]
#[test]
fn test_scan_bool() {
    use ::ScanErrorKind as SEK;

    assert_match!(<bool>::scan_from(""), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("y"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("n"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("yes"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("no"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from(" "), Err(SEK::Missing));
    assert_match!(<bool>::scan_from(" true"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from(" false"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("true"), Ok((true, 4)));
    assert_match!(<bool>::scan_from("false"), Ok((false, 5)));
    assert_match!(<bool>::scan_from("True"), Err(SEK::Missing));
    assert_match!(<bool>::scan_from("False"), Err(SEK::Missing));
}

impl<'a> ScanFromStr<'a> for char {
    type Output = char;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        let cur = try!(StrCursor::new_at_start(s).at_next_cp().ok_or(ScanErrorKind::Missing));
        Ok((cur.cp_before().unwrap(), cur.byte_pos()))
    }
}

#[cfg(test)]
#[test]
fn test_scan_char() {
    use ::ScanErrorKind as SEK;

    assert_match!(<char>::scan_from(""), Err(SEK::Missing));
    assert_match!(<char>::scan_from(" "), Ok((' ', 1)));
    assert_match!(<char>::scan_from("x"), Ok(('x', 1)));
    assert_match!(<char>::scan_from("xy"), Ok(('x', 1)));
    assert_match!(<char>::scan_from("é"), Ok(('e', 1)));
    assert_match!(<char>::scan_from("é"), Ok(('é', 2)));
    assert_match!(<char>::scan_from("字"), Ok(('字', 3)));
}

parse_scanner! { impl<'a> for f32, regex FLOAT_RE }
parse_scanner! { impl<'a> for f64, regex FLOAT_RE }

#[cfg(test)]
#[test]
fn test_scan_f64() {
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

    assert_match!(<f64>::scan_from(""), Err(SEK::Missing));
    assert_match!(<f64>::scan_from("-"), Err(SEK::Missing));
    assert_match!(<f64>::scan_from("+"), Err(SEK::Missing));
    assert_match!(<f64>::scan_from("x"), Err(SEK::Missing));
    assert_match!(<f64>::scan_from(" "), Err(SEK::Missing));
    assert_match!(<f64>::scan_from(" 0"), Err(SEK::Missing));

    assert_match!(<f64>::scan_from("inf"), Ok((::std::f64::INFINITY, 3)));
    assert_match!(<f64>::scan_from("-inf"), Ok((::std::f64::NEG_INFINITY, 4)));
    assert_match!(<f64>::scan_from("NaN"), Ok((v, 3)) if v.is_nan());

    check_f64!(0.0);
    check_f64!(1.0);
    check_f64!(3e-5);
    check_f64!(0.1);
    check_f64!(12345.);
    check_f64!(0.12345);
    check_f64!(12345.67890);
    check_f64!(2.2250738585072014e-308);
    check_f64!(101e-33);
    check_f64!(1e23);
    check_f64!(2075e23);
    check_f64!(8713e-23);
    check_f64!(1e300);
    check_f64!(123456789.34567e250);
    check_f64!(5e-324);
    check_f64!(91e-324);
    check_f64!(1e-322);
    check_f64!(13245643e-320);
    check_f64!(2.22507385851e-308);
    check_f64!(2.1e-308);
    check_f64!(4.9406564584124654e-324);
    check_f64!(1e-325);
    check_f64!(1e-326);
    check_f64!(1e-500);
    check_f64!(1.448997445238699);
}

parse_scanner! { impl<'a> for i8, regex SINTEGER_RE }
parse_scanner! { impl<'a> for i16, regex SINTEGER_RE }
parse_scanner! { impl<'a> for i32, regex SINTEGER_RE }
parse_scanner! { impl<'a> for i64, regex SINTEGER_RE }
parse_scanner! { impl<'a> for isize, regex SINTEGER_RE }

#[cfg(test)]
#[test]
fn test_scan_i32() {
    use ::ScanErrorKind as SEK;

    assert_match!(<i32>::scan_from(""), Err(SEK::Missing));
    assert_match!(<i32>::scan_from("-"), Err(SEK::Missing));
    assert_match!(<i32>::scan_from("+"), Err(SEK::Missing));
    assert_match!(<i32>::scan_from("x"), Err(SEK::Missing));
    assert_match!(<i32>::scan_from(" "), Err(SEK::Missing));
    assert_match!(<i32>::scan_from(" 0"), Err(SEK::Missing));
    assert_match!(<i32>::scan_from("0"), Ok((0, 1)));
    assert_match!(<i32>::scan_from("42"), Ok((42, 2)));
    assert_match!(<i32>::scan_from("-312"), Ok((-312, 4)));
    assert_match!(<i32>::scan_from("1_234"), Ok((1, 1)));
}

parse_scanner! { impl<'a> for u8, regex UINTEGER_RE }
parse_scanner! { impl<'a> for u16, regex UINTEGER_RE }
parse_scanner! { impl<'a> for u32, regex UINTEGER_RE }
parse_scanner! { impl<'a> for u64, regex UINTEGER_RE }
parse_scanner! { impl<'a> for usize, regex UINTEGER_RE }

#[cfg(test)]
#[test]
fn test_scan_u32() {
    use ::ScanErrorKind as SEK;

    assert_match!(<u32>::scan_from(""), Err(SEK::Missing));
    assert_match!(<u32>::scan_from("-"), Err(SEK::Missing));
    assert_match!(<u32>::scan_from("+"), Err(SEK::Missing));
    assert_match!(<u32>::scan_from("x"), Err(SEK::Missing));
    assert_match!(<u32>::scan_from(" "), Err(SEK::Missing));
    assert_match!(<u32>::scan_from(" 0"), Err(SEK::Missing));
    assert_match!(<u32>::scan_from("0"), Ok((0, 1)));
    assert_match!(<u32>::scan_from("42"), Ok((42, 2)));
    assert_match!(<u32>::scan_from("-312"), Err(SEK::Missing));
    assert_match!(<u32>::scan_from("1_234"), Ok((1, 1)));
}

impl<'a> ScanFromStr<'a> for &'a str {
    type Output = &'a str;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        Word::scan_from(s)
    }
}

impl<'a> ScanFromStr<'a> for &'a [u8] {
    type Output = &'a [u8];
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        <&str>::scan_from(s).map(|(v, n)| (v.as_bytes(), n))
    }
}

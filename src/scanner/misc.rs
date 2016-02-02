/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Miscellaneous, abstract scanners.
*/
use std::marker::PhantomData;
use regex::Regex;
use strcursor::StrCursor;
use ::ScanError;
use ::input::ScanInput;
use super::{
    ScanFromStr, ScanSelfFromStr,
    ScanFromBinary, ScanFromOctal, ScanFromHex,
};
use super::util::StrUtil;

lazy_static! {
    static ref IDENT_RE: Regex = Regex::new(r"^(\p{XID_Start}|_)\p{XID_Continue}*").unwrap();
    static ref LINE_RE: Regex = Regex::new(r"^(.*?)(\n|\r\n|\r|$)").unwrap();
    static ref NONSPACE_RE: Regex = Regex::new(r"^\S+").unwrap();
    static ref NUMBER_RE: Regex = Regex::new(r"^\d+").unwrap();
    static ref SPACE_RE: Regex = Regex::new(r"^\s+").unwrap();
    static ref WORD_RE: Regex = Regex::new(r"^\w+").unwrap();
    static ref WORDISH_RE: Regex = Regex::new(r"^(\d+|\w+|\S)").unwrap();
}

/**
Scans the given `Output` type from its binary representation.
*/
pub struct Binary<Output>(PhantomData<Output>);

impl<'a, Output> ScanFromStr<'a> for Binary<Output>
where Output: ScanFromBinary<'a> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        Output::scan_from_binary(s)
    }
}

#[cfg(test)]
#[test]
fn test_binary() {
    assert_match!(Binary::<i32>::scan_from("0 1 2 x"), Ok((0b0, 1)));
    assert_match!(Binary::<i32>::scan_from("012x"), Ok((0b1, 2)));
    assert_match!(Binary::<i32>::scan_from("0b012x"), Ok((0b0, 1)));
    assert_match!(Binary::<i32>::scan_from("110010101110000b"), Ok((0x6570, 15)));
}

/**
Scans all remaining input into a string.

In most cases, you should use the `.. name` tail capture term to perform this task.  This scanner is provided as a way to do this in contexts where tail capture is not valid (because it normally wouldn't make any sense).
*/
pub struct Everything<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Everything<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        Ok((s.into(), s.len()))
    }
}

#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Everything<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        Ok((s.into(), s.len()))
    }
}

#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Everything<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        Ok((s.into(), s.len()))
    }
}

#[cfg(test)]
#[test]
fn test_everything() {
    // That's the scanner named `Everything`, not literally everything.
    assert_match!(Everything::<&str>::scan_from(""), Ok(("", 0)));
    assert_match!(Everything::<&str>::scan_from("で"), Ok(("で", 3)));
    assert_match!(Everything::<&str>::scan_from("うまいー　うまいー　ぼうぼうぼうぼう"), Ok(("うまいー　うまいー　ぼうぼうぼうぼう", 54)));
}

/**
Scans the given `Output` type from its hexadecimal representation.
*/
pub struct Hex<Output>(PhantomData<Output>);

impl<'a, Output> ScanFromStr<'a> for Hex<Output>
where Output: ScanFromHex<'a> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        Output::scan_from_hex(s)
    }
}

#[cfg(test)]
#[test]
fn test_hex() {
    assert_match!(Hex::<i32>::scan_from("0 1 2 x"), Ok((0x0, 1)));
    assert_match!(Hex::<i32>::scan_from("012x"), Ok((0x12, 3)));
    assert_match!(Hex::<i32>::scan_from("0x012x"), Ok((0x0, 1)));
    assert_match!(Hex::<i32>::scan_from("BadCafé"), Ok((0xbadcaf, 6)));
}

/**
Scans a single identifier into a string.

Specifically, this will match a single `XID_Start` character (or underscore) followed by zero or more `XID_Continue` characters.
*/
pub struct Ident<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Ident<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match IDENT_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            None => {
                // Err(ScanError::syntax("expected identifier"))
                Err(ScanError::syntax_no_message())
            },
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Ident<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match IDENT_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            None => {
                // Err(ScanError::syntax("expected identifier"))
                Err(ScanError::syntax_no_message())
            },
        }
    }
}

#[cfg(not(str_into_output_extra_broken))]
// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
impl<'a, Output> ScanFromStr<'a> for Ident<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match IDENT_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            None => {
                // Err(ScanError::syntax("expected identifier"))
                Err(ScanError::syntax_no_message())
            },
        }
    }
}

#[cfg(test)]
#[test]
fn test_ident() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(Ident::<&str>::scan_from(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Ident::<&str>::scan_from("a"), Ok(("a", 1)));
    assert_match!(Ident::<&str>::scan_from("two words "), Ok(("two", 3)));
    assert_match!(Ident::<&str>::scan_from("two_words "), Ok(("two_words", 9)));
    assert_match!(Ident::<&str>::scan_from("0123abc456 "), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Ident::<&str>::scan_from("_0123abc456 "), Ok(("_0123abc456", 11)));
    assert_match!(Ident::<&str>::scan_from("f(blah)"), Ok(("f", 1)));
}

/**
Explicitly infer the type of a scanner.

This is useful in cases where you want to only *partially* specify a scanner type, but the partial type cannot be inferred under normal circumstances.

For example, tuples allow their element types to scan to be abstract scanners; *e.g.* `(Word<String>, Hex<i32>)` will scan to `(String, i32)`.  However, this interferes with inferring the scanner type when you *partially* specify a tuple type.  If you attempt to store the result of scanning `(_, _)` into a `(String, i32)`, Rust cannot determine whether the *scanner* type should be `(String, Hex<i32>)`, or `(Word<String>, i32)`, or something else entirely.

This scanner, then, *requires* that the inner type scan to itself and *only* to itself.
*/
pub struct Inferred<T>(PhantomData<T>);

impl<'a, T> ScanFromStr<'a> for Inferred<T>
where T: ScanSelfFromStr<'a> {
    type Output = T;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        T::scan_from(s)
    }
}

/**
Scans everything up to the end of the current line, *or* the end of the input, whichever comes first.  The scanned result *does not* include the line terminator.

Note that this is effectively equivalent to the `Everything` matcher when used with `readln!`.
*/
pub struct Line<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Line<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        const EX_MSG: &'static str = "line scanning regex failed to match anything";
        let cap = LINE_RE.captures(s).expect(EX_MSG);
        let (_, b) = cap.pos(0).expect(EX_MSG);
        let (c, d) = cap.pos(1).expect(EX_MSG);
        Ok((s[c..d].into(), b))
    }
}

#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Line<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        const EX_MSG: &'static str = "line scanning regex failed to match anything";
        let cap = LINE_RE.captures(s).expect(EX_MSG);
        let (_, b) = cap.pos(0).expect(EX_MSG);
        let (c, d) = cap.pos(1).expect(EX_MSG);
        Ok((s[c..d].into(), b))
    }
}

#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Line<'a, Output> where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        const EX_MSG: &'static str = "line scanning regex failed to match anything";
        let cap = LINE_RE.captures(s).expect(EX_MSG);
        let (_, b) = cap.pos(0).expect(EX_MSG);
        let (c, d) = cap.pos(1).expect(EX_MSG);
        Ok((s[c..d].into(), b))
    }
}

#[cfg(test)]
#[test]
fn test_line() {
    assert_match!(Line::<&str>::scan_from(""), Ok(("", 0)));
    assert_match!(Line::<&str>::scan_from("abc def"), Ok(("abc def", 7)));
    assert_match!(Line::<&str>::scan_from("abc\ndef"), Ok(("abc", 4)));
    assert_match!(Line::<&str>::scan_from("abc\r\ndef"), Ok(("abc", 5)));
    assert_match!(Line::<&str>::scan_from("abc\rdef"), Ok(("abc", 4)));
}

/**
Scans a sequence of non-space characters into a string.

This *will not* match an empty sequence; there must be at least one non-space character for the scan to succeed.
*/
pub struct NonSpace<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for NonSpace<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NONSPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected at least one non-space character")),
            None => Err(ScanError::syntax_no_message())
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for NonSpace<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NONSPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected at least one non-space character")),
            None => Err(ScanError::syntax_no_message())
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for NonSpace<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NONSPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected at least one non-space character")),
            None => Err(ScanError::syntax_no_message())
        }
    }
}

#[cfg(test)]
#[test]
fn test_non_space() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(NonSpace::<&str>::scan_from(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(NonSpace::<&str>::scan_from(" abc "), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(NonSpace::<&str>::scan_from("abc "), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\t"), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\r"), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\n"), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\u{a0}"), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\u{2003}"), Ok(("abc", 3)));
    assert_match!(NonSpace::<&str>::scan_from("abc\u{200B}"), Ok(("abc\u{200b}", 6)));
    assert_match!(NonSpace::<&str>::scan_from("abc\u{3000}"), Ok(("abc", 3)));
}

/**
Scans a single number into a string.

Specifically, this will match a continuous run of decimal characters (*i.e.* /`\d+`/).

Note that this *includes* non-ASCII decimal characters, meaning it will scan numbers such as "42", "１７０１", and "𐒩０꘠᧑".
*/
pub struct Number<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Number<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NUMBER_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a number")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Number<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NUMBER_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a number")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Number<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match NUMBER_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a number")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

#[cfg(test)]
#[test]
fn test_number() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(Number::<&str>::scan_from(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Number::<&str>::scan_from("a"), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Number::<&str>::scan_from("0"), Ok(("0", 1)));
    assert_match!(Number::<&str>::scan_from("0x"), Ok(("0", 1)));
    assert_match!(Number::<&str>::scan_from("x0"), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Number::<&str>::scan_from("123 456 xyz"), Ok(("123", 3)));
    assert_match!(Number::<&str>::scan_from("123 456 xyz"), Ok(("123", 3)));
    assert_match!(Number::<&str>::scan_from("123４５６789 "), Ok(("123４５６789", 15)));
    assert_match!(Number::<&str>::scan_from("𐒩０꘠᧑ "), Ok(("𐒩０꘠᧑", 13)));
}

/**
Scans the given `Output` type from its octal representation.
*/
pub struct Octal<Output>(PhantomData<Output>);

impl<'a, Output> ScanFromStr<'a> for Octal<Output>
where Output: ScanFromOctal<'a> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        Output::scan_from_octal(s)
    }
}

#[cfg(test)]
#[test]
fn test_octal() {
    assert_match!(Octal::<i32>::scan_from("0 1 2 x"), Ok((0o0, 1)));
    assert_match!(Octal::<i32>::scan_from("012x"), Ok((0o12, 3)));
    assert_match!(Octal::<i32>::scan_from("0o012x"), Ok((0o0, 1)));
    assert_match!(Octal::<i32>::scan_from("7558"), Ok((0o755, 3)));
}

/**
An abstract scanner that scans a `(K, V)` value using the syntax `K: V`.

This scanner is designed to take advantage of three things:

1. Maps (*i.e.* associative containers) typically print themselves with the syntax `{key_0: value_0, key_1: value_1, ...}`.

2. Maps typically implement `Extend<(K, V)>`; that is, you can add new items by extending the map with a `(K, V)` tuple.

3. Repeating bindings can be scanned into any container that implements `Default` and `Extend`.

As such, this scanner allows one to parse a `Map` type like so:

```ignore
scan!(input; "{", [let kvs: KeyValuePair<K, V>],*: Map<_, _>, "}" => kvs)
```
*/
pub struct KeyValuePair<K, V>(PhantomData<(K, V)>);

impl<'a, K, V> ScanFromStr<'a> for KeyValuePair<K, V>
where K: ScanSelfFromStr<'a>, V: ScanSelfFromStr<'a> {
    type Output = (K, V);
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        scan!(s;
            (let k: K, ":", let v: V, ..tail) => ((k, v), s.subslice_offset_stable(tail).unwrap())
        )
    }
}

/**
Scans a quoted string.

Specifically, it scans the quoting format used by the `Debug` formatter for strings.

The scanned string has all escape sequences expanded to their values, and the surrounding quotes removed.
*/
pub enum QuotedString {}

impl<'a> ScanFromStr<'a> for QuotedString {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        let syn = |s| ScanError::syntax(s);

        let cur = StrCursor::new_at_start(s);
        let (cp, cur) = try!(cur.next_cp().ok_or(syn("expected quoted string")));
        match cp {
            '"' => (),
            _ => return Err(syn("expected `\"` for quoted string"))
        }

        let mut s = String::new();
        let mut cur = cur;
        loop {
            match cur.next_cp() {
                None => return Err(syn("unterminated quoted string")),
                Some(('\\', after)) => {
                    match after.slice_after().split_escape_default() {
                        Err(err) => return Err(ScanError::other(err).add_offset(after.byte_pos())),
                        Ok((cp, tail)) => {
                            // TODO: replace this
                            unsafe { cur.unsafe_set_at(tail); }
                            s.push(cp);
                        },
                    }
                },
                Some(('"', after)) => {
                    cur = after;
                    break;
                },
                Some((cp, after)) => {
                    cur = after;
                    s.push(cp);
                },
            }
        }

        Ok((s, cur.byte_pos()))
    }
}

#[cfg(test)]
#[test]
fn test_quoted_string() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;
    use self::QuotedString as QS;

    assert_match!(QS::scan_from(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(QS::scan_from("dummy xyz"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(QS::scan_from("'dummy' xyz"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(QS::scan_from("\"dummy\" xyz"),
        Ok((ref s, 7)) if s == "dummy");
    assert_match!(QS::scan_from("\"ab\\\"cd\" xyz"),
        Ok((ref s, 8)) if s == "ab\"cd");
    assert_match!(QS::scan_from("\"ab\\x41cd\" xyz"),
        Ok((ref s, 10)) if s == "abAcd");
    assert_match!(QS::scan_from("\"a\\'b\\u{5B57}c\\0d\" xyz"),
        Ok((ref s, 18)) if s == "a'b字c\0d");
}

/**
Scans a sequence of space characters into a string.

This *will not* match an empty sequence; there must be at least one space character for the scan to succeed.
*/
pub struct Space<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Space<'a, &'a str> {
    type Output = &'a str;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match SPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a space")),
            None => Err(ScanError::syntax_no_message()),
        }
    }

    fn wants_leading_junk_stripped() -> bool { false }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Space<'a, String> {
    type Output = String;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match SPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a space")),
            None => Err(ScanError::syntax_no_message()),
        }
    }

    fn wants_leading_junk_stripped() -> bool { false }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Space<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match SPACE_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a space")),
            None => Err(ScanError::syntax_no_message()),
        }
    }

    fn wants_leading_junk_stripped() -> bool { false }
}

#[cfg(test)]
#[test]
fn test_space() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(Space::<&str>::scan_from(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Space::<&str>::scan_from("a"), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Space::<&str>::scan_from("0"), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Space::<&str>::scan_from(" "), Ok((" ", 1)));
    assert_match!(Space::<&str>::scan_from("\t"), Ok(("\t", 1)));
    assert_match!(Space::<&str>::scan_from("\r"), Ok(("\r", 1)));
    assert_match!(Space::<&str>::scan_from("\n"), Ok(("\n", 1)));
    assert_match!(Space::<&str>::scan_from("\r\n"), Ok(("\r\n", 2)));
    assert_match!(Space::<&str>::scan_from("  \t \n \t\t "), Ok(("  \t \n \t\t ", 9)));
    assert_match!(Space::<&str>::scan_from("  \t \nx \t\t "), Ok(("  \t \n", 5)));
}

/**
Scans a single word into a string.

Specifically, this will match a continuous run of alphabetic, digit, punctuation, mark, and joining characters (*i.e.* /`\w+`/).
*/
pub struct Word<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Word<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match WORD_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Word<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match WORD_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Word<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        match WORD_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

#[cfg(test)]
#[test]
fn test_word() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;

    assert_match!(Word::<&str>::scan_from(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(Word::<&str>::scan_from("a"), Ok(("a", 1)));
    assert_match!(Word::<&str>::scan_from("0"), Ok(("0", 1)));
    assert_match!(Word::<&str>::scan_from("0x"), Ok(("0x", 2)));
    assert_match!(Word::<&str>::scan_from("x0"), Ok(("x0", 2)));
    assert_match!(Word::<&str>::scan_from("123 456 xyz"), Ok(("123", 3)));
    assert_match!(Word::<&str>::scan_from("123 456 xyz"), Ok(("123", 3)));
    assert_match!(Word::<&str>::scan_from("123４５６789 "), Ok(("123４５６789", 15)));
    assert_match!(Word::<&str>::scan_from("𐒩０꘠᧑ "), Ok(("𐒩０꘠᧑", 13)));
    assert_match!(Word::<&str>::scan_from("kumquat,bingo"), Ok(("kumquat", 7)));
    assert_match!(Word::<&str>::scan_from("mixed言葉كتابة "), Ok(("mixed言葉كتابة", 21)));
}

/**
Scans a single word-ish thing into a string.

Specifically, this will match a word (a continuous run of alphabetic, digit, punctuation, mark, and joining characters), a number (a continuous run of digits), or a single other non-whitespace character  (*i.e.* /`\w+|\d+|\S`/).
*/
pub struct Wordish<'a, Output=&'a str>(PhantomData<(&'a (), Output)>);

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Wordish<'a, &'a str> {
    type Output = &'a str;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        // TODO: This should be modified to grab an entire *grapheme cluster* in the event it can't find a word or number.
        match WORDISH_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word, number or some other character")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(str_into_output_extra_broken)]
impl<'a> ScanFromStr<'a> for Wordish<'a, String> {
    type Output = String;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        // TODO: This should be modified to grab an entire *grapheme cluster* in the event it can't find a word or number.
        match WORDISH_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word, number or some other character")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

// FIXME: Error message omitted due to https://github.com/rust-lang/rust/issues/26448.
#[cfg(not(str_into_output_extra_broken))]
impl<'a, Output> ScanFromStr<'a> for Wordish<'a, Output>
where &'a str: Into<Output> {
    type Output = Output;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        // TODO: This should be modified to grab an entire *grapheme cluster* in the event it can't find a word or number.
        match WORDISH_RE.find(s) {
            Some((a, b)) => {
                let word = &s[a..b];
                let tail = &s[b..];
                Ok((word.into(), s.subslice_offset_stable(tail).unwrap()))
            },
            // None => Err(ScanError::syntax("expected a word, number or some other character")),
            None => Err(ScanError::syntax_no_message()),
        }
    }
}

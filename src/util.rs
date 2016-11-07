/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Internal utilities.
*/
use std::error::Error;
use std::fmt::{self, Display};
use strcursor::StrCursor;

/**
String error message.

This exists because `Error` is not implemented for `&str` in Rust < 1.6.
*/
#[derive(Copy, Clone)]
pub struct MsgErr(pub &'static str);

impl fmt::Debug for MsgErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl Display for MsgErr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Error for MsgErr {
    fn description(&self) -> &str {
        self.0
    }
}

/**
Various string utility methods.
*/
pub trait StrUtil {
    /**
    Returns the byte offset of an inner slice relative to an enclosing outer slice.

    Named `*_stable` to avoid conflicting with the deprecated method in < 1.4.0.
    */
    fn subslice_offset_stable(&self, inner: &Self) -> Option<usize>;

    /**
    Extracts an escape sequence (sans leading backslash) from the start of this string, returning the unescaped code point, and the unconsumed input.
    */
    fn split_escape_default(&self) -> Result<(char, &Self), EscapeError>;
}

impl StrUtil for str {
    fn subslice_offset_stable(&self, inner: &str) -> Option<usize> {
        let self_beg = self.as_ptr() as usize;
        let inner = inner.as_ptr() as usize;
        if inner < self_beg || inner > self_beg.wrapping_add(self.len()) {
            None
        } else {
            Some(inner.wrapping_sub(self_beg))
        }
    }

    fn split_escape_default(&self) -> Result<(char, &Self), EscapeError> {
        use self::EscapeError::*;

        let cur = StrCursor::new_at_start(self);

        let (cp, cur) = try!(cur.next_cp().ok_or(LoneSlash));
        let is_x_esc = match cp {
            '"' => return Ok(('"', cur.slice_after())),
            '0' => return Ok(('\0', cur.slice_after())),
            '\'' => return Ok(('\'', cur.slice_after())),
            '\\' => return Ok(('\\', cur.slice_after())),
            'n' => return Ok(('\n', cur.slice_after())),
            'r' => return Ok(('\r', cur.slice_after())),
            'u' => false,
            'x' => true,
            cp => return Err(UnknownEscape(cp))
        };

        let s = cur.slice_after();
        let esc: fn(_) -> _ = if is_x_esc {
            match_hex_esc
        } else {
            match_uni_esc
        };
        let err = if is_x_esc { MalformedHex } else { MalformedUnicode };
        let (hex, tail) = try!(esc(s).ok_or(err));
        let hex = &s[(hex.0)..(hex.1)];
        let tail = &s[tail..];
        let usv = try!(u32::from_str_radix(hex, 16).map_err(|_| InvalidValue));
        if is_x_esc && usv > 0x7f {
            return Err(InvalidValue);
        }
        let cp = try!(::std::char::from_u32(usv).ok_or(InvalidValue));
        Ok((cp, tail))
    }
}

/**
Extension trait for Unicode tables.
*/
pub trait TableUtil<T: Ord> {
    /**
    Determines whether or not the given character is in the table.
    */
    fn span_table_contains(&self, e: &T) -> bool;
}

impl<T: Ord> TableUtil<T> for [(T, T)] {
    fn span_table_contains(&self, e: &T) -> bool {
        use std::cmp::Ordering::*;
        let len = self.len();

        let mut lo = 0;
        let mut hi = len;
        while lo < hi && hi <= len {
            let mid = lo + (hi - lo) / 2;
            let mid_e = &self[mid];
            match e.cmp(&mid_e.0) {
                Less => hi = mid,
                Equal => return true,
                Greater => {
                    match e.cmp(&mid_e.1) {
                        Less | Equal => return true,
                        Greater => lo = mid + 1,
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
#[test]
fn test_span_table_contains() {
    use ::unicode::general_category::Nd_table as Nd;

    // ('\u{30}', '\u{39}')
    assert_eq!(Nd.span_table_contains(&'/'), false);
    assert_eq!(Nd.span_table_contains(&'0'), true);
    assert_eq!(Nd.span_table_contains(&'1'), true);
    assert_eq!(Nd.span_table_contains(&'2'), true);
    assert_eq!(Nd.span_table_contains(&'3'), true);
    assert_eq!(Nd.span_table_contains(&'4'), true);
    assert_eq!(Nd.span_table_contains(&'5'), true);
    assert_eq!(Nd.span_table_contains(&'6'), true);
    assert_eq!(Nd.span_table_contains(&'7'), true);
    assert_eq!(Nd.span_table_contains(&'8'), true);
    assert_eq!(Nd.span_table_contains(&'9'), true);
    assert_eq!(Nd.span_table_contains(&':'), false);

    // ('\u{1090}', '\u{1099}')
    assert_eq!(Nd.span_table_contains(&'\u{108f}'), false);
    assert_eq!(Nd.span_table_contains(&'\u{1090}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{1099}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{109a}'), false);

    // ('\u{111d0}', '\u{111d9}')
    assert_eq!(Nd.span_table_contains(&'\u{111cf}'), false);
    assert_eq!(Nd.span_table_contains(&'\u{111d0}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{111d9}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{111da}'), false);

    // ('\u{1d7ce}', '\u{1d7ff}')
    assert_eq!(Nd.span_table_contains(&'\u{1d7cd}'), false);
    assert_eq!(Nd.span_table_contains(&'\u{1d7ce}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{1d7ff}'), true);
    assert_eq!(Nd.span_table_contains(&'\u{1d800}'), false);
}

/**
Indicates why unescaping a character from a string failed.
*/
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EscapeError {
    /// Backslash with nothing after it.
    LoneSlash,
    /// Backslash followed by unrecognised character.
    UnknownEscape(char),
    /// Malformed hex escape sequence.
    MalformedHex,
    /// Malformed unicode escape sequence.
    MalformedUnicode,
    /// Escape contained an invalid value.
    InvalidValue,
}

impl Display for EscapeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::EscapeError::*;
        match *self {
            LoneSlash => "backslash with nothing after it".fmt(fmt),
            UnknownEscape(cp) => write!(fmt, "unknown escape `\\{:?}`", cp),
            MalformedHex => "malformed hex escape".fmt(fmt),
            MalformedUnicode => "malformed Unicode escape".fmt(fmt),
            InvalidValue => "escape produced invalid code point value".fmt(fmt),
        }
    }
}

impl Error for EscapeError {
    fn description(&self) -> &str {
        use self::EscapeError::*;
        match *self {
            LoneSlash => "backslash with nothing after it",
            UnknownEscape(_) => "unknown escape",
            MalformedHex => "malformed hex escape",
            MalformedUnicode => "malformed Unicode escape",
            InvalidValue => "escape produced invalid code point value",
        }
    }
}

#[cfg(test)]
#[test]
fn test_subslice_offset() {
    let string = "a\nb\nc";
    let lines: Vec<&str> = string.lines().collect();

    assert_eq!(string.subslice_offset_stable(lines[0]), Some(0)); // &"a"
    assert_eq!(string.subslice_offset_stable(lines[1]), Some(2)); // &"b"
    assert_eq!(string.subslice_offset_stable(lines[2]), Some(4)); // &"c"
    assert_eq!(string[..4].subslice_offset_stable(lines[2]), Some(4));

    // Offset by 1 in case `other` is adjacent to `string` in memory.
    let other = "Xother";
    let other = &other[1..];
    assert_eq!(string.subslice_offset_stable(other), None);
}

#[cfg(test)]
#[test]
fn test_split_escape_default() {
    use self::EscapeError::*;

    assert_eq!("".split_escape_default(), Err(LoneSlash));
    assert_eq!("0bc".split_escape_default(), Ok(('\0', "bc")));
    assert_eq!("'bc".split_escape_default(), Ok(('\'', "bc")));
    assert_eq!("nbc".split_escape_default(), Ok(('\n', "bc")));
    assert_eq!("rbc".split_escape_default(), Ok(('\r', "bc")));
    assert_eq!("wbc".split_escape_default(), Err(UnknownEscape('w')));
    assert_eq!("x".split_escape_default(), Err(MalformedHex));
    assert_eq!("x6".split_escape_default(), Err(MalformedHex));
    assert_eq!("x61".split_escape_default(), Ok(('a', "")));
    assert_eq!("x61bc".split_escape_default(), Ok(('a', "bc")));
    assert_eq!("x7f".split_escape_default(), Ok(('\x7f', "")));
    assert_eq!("x80".split_escape_default(), Err(InvalidValue));
    assert_eq!("u".split_escape_default(), Err(MalformedUnicode));
    assert_eq!("u{".split_escape_default(), Err(MalformedUnicode));
    assert_eq!("u{6".split_escape_default(), Err(MalformedUnicode));
    assert_eq!("u{61".split_escape_default(), Err(MalformedUnicode));
    assert_eq!("u{61}".split_escape_default(), Ok(('a', "")));
    assert_eq!("u{61}bc".split_escape_default(), Ok(('a', "bc")));
    assert_eq!("u{7f}".split_escape_default(), Ok(('\x7f', "")));
    assert_eq!("u{80}".split_escape_default(), Ok(('\u{80}', "")));
    assert_eq!("u{2764}".split_escape_default(), Ok(('❤', "")));
    assert_eq!("u{110000}".split_escape_default(), Err(InvalidValue));
}

fn match_hex_esc(s: &str) -> Option<((usize, usize), usize)> {
    if s.bytes().take_while(|b| is_xdigit(*b)).take(2).count() == 2 {
        Some(((0, 2), 2))
    } else {
        None
    }
}

fn match_uni_esc(s: &str) -> Option<((usize, usize), usize)> {
    let mut bs = s.bytes().enumerate();
    match bs.next() {
        Some((_, b'{')) => (),
        _ => return None,
    }
    match bs.next() {
        Some((_, b)) if is_xdigit(b) => (),
        _ => return None,
    }
    while let Some((i, b)) = bs.next() {
        if is_xdigit(b) { /* do nothing */ }
        else if b == b'}' {
            return Some(((1, i), i+1));
        } else {
            return None;
        }
    }
    None
}

fn is_xdigit(b: u8) -> bool {
    match b {
        b'0'...b'9' | b'a'...b'f' | b'A'...b'F' => true,
        _ => false,
    }
}

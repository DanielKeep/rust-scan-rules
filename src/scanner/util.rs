/*!
Internal utilities for scanner implementations.
*/
use regex::Regex;
use strcursor::StrCursor;

lazy_static! {
    static ref HEX_ESC_RE: Regex = Regex::new(r"^([:xdigit:]{2})").unwrap();
    static ref UNI_ESC_RE: Regex = Regex::new(r"^\{([:xdigit:]+)\}").unwrap();
}

/**
Various string utility methods.
*/
pub trait StrUtil {
    /**
    Returns the byte offset of an inner slice relative to an enclosing outer slice.
    */
    fn subslice_offset(&self, inner: &Self) -> Option<usize>;

    /**
    Extracts an escape sequence (sans leading backslash) from the start of this string, returning the unescaped code point, and the unconsumed input.
    */
    fn split_escape_default(&self) -> Result<(char, &Self), EscapeError>;
}

impl StrUtil for str {
    fn subslice_offset(&self, inner: &str) -> Option<usize> {
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

        let re = if is_x_esc { &*HEX_ESC_RE } else { &*UNI_ESC_RE };
        let err = if is_x_esc { MalformedHex } else { MalformedUnicode };
        let cap = try!(re.captures(cur.slice_after()).ok_or(err));
        let hex = try!(cap.at(1).ok_or(err));
        let tail = &cur.slice_after()[try!(cap.pos(0).ok_or(err)).1 ..];
        let usv = try!(u32::from_str_radix(hex, 16).map_err(|_| InvalidValue));
        if is_x_esc && usv > 0x7f {
            return Err(InvalidValue);
        }
        let cp = try!(::std::char::from_u32(usv).ok_or(InvalidValue));
        Ok((cp, tail))
    }
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

#[cfg(test)]
#[test]
fn test_subslice_offset() {
    let string = "a\nb\nc";
    let lines: Vec<&str> = string.lines().collect();

    assert!(string.subslice_offset(lines[0]) == Some(0)); // &"a"
    assert!(string.subslice_offset(lines[1]) == Some(2)); // &"b"
    assert!(string.subslice_offset(lines[2]) == Some(4)); // &"c"
    assert!(string.subslice_offset("other!") == None);
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

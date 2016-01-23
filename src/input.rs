/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This module contains items related to input handling.

Note that this aspect of `scan-rules` is still under design and is very likely to change drastically in future.
*/
use std::ops::Deref;
use itertools::Itertools;
use regex::Regex;
use ::{ScanError, ScanErrorKind};

lazy_static! {
    /**
    This regex defines what the default literal matching code considers a "part" for the purposes of comparison.

    Specifically, it should be a contiguous sequence of letters *or* a single, non-whitespace character.
    */
    static ref LITERAL_PART_RE: Regex = Regex::new(r"(\w+|\S)").unwrap();
}

/**
This trait defines the interface to input values that can be scanned.
*/
pub trait ScanInput<'a>: Sized {
    /**
    Assert that the input has been exhausted, or that the current position is a valid place to "stop".
    */
    fn try_end(self) -> Result<(), (ScanError, Self)>;

    /**
    Scan a value from the current position.  The closure will be called with a string slice of all available input, and is expected to return *either* the scanned value, and the number of bytes of input consumed, *or* a reason why scanning failed.

    The input will have all leading whitespace removed, if applicable.
    */
    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    /**
    Performs the same task as [`try_scan`](#tymethod.try_scan), except that it *does not* perform whitespace stripping.
    */
    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    /**
    Match the provided literal term against the input.

    Implementations are free to interpret "match" as they please.
    */
    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError, Self)>;
}

/**
The basic input type for scanning.
*/
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Cursor<'a> {
    offset: usize,
    slice: &'a str,
}

impl<'a> Cursor<'a> {
    /**
    Construct a new `Cursor` with a specific `offset`.

    The `offset` is logically the number of bytes which have already been consumed from the original input; these already-consumed bytes *must not* be included in `slice`.
    */
    pub fn new_with_offset(slice: &'a str, offset: usize) -> Self {
        Cursor {
            offset: offset,
            slice: slice,
        }
    }

    /**
    Advance the cursor by the given number of bytes.
    */
    pub fn advance_by(self, bytes: usize) -> Self {
        Cursor {
            offset: self.offset + bytes,
            slice: &self.slice[bytes..],
        }
    }

    /**
    Access the wrapped string slice.
    */
    pub fn as_str(self) -> &'a str {
        self.slice
    }

    /**
    Returns the number of bytes of input that have been consumed by this `Cursor`.
    */
    pub fn offset(self) -> usize {
        self.offset
    }
}

impl<'a> Deref for Cursor<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

impl<'a> From<&'a str> for Cursor<'a> {
    fn from(v: &'a str) -> Self {
        Cursor::new_with_offset(v, 0)
    }
}

impl<'a> From<&'a String> for Cursor<'a> {
    fn from(v: &'a String) -> Self {
        Cursor::new_with_offset(v, 0)
    }
}

impl<'a> ScanInput<'a> for Cursor<'a> {
    fn try_end(self) -> Result<(), (ScanError, Self)> {
        if (skip_space(self.slice).0).len() == 0 {
            Ok(())
        } else {
            Err((ScanError::expected_end(self.offset()), self))
        }
    }

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        let (tmp, tmp_off) = skip_space(self.slice);
        match f(tmp) {
            Ok((out, off)) => Ok((out, self.advance_by(tmp_off + off))),
            Err(err) => Err((ScanError::new(self.advance_by(tmp_off).offset(), err), self))
        }
    }

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        match f(self.slice) {
            Ok((out, off)) => Ok((out, self.advance_by(off))),
            Err(err) => Err((ScanError::new(self.offset(), err), self))
        }
    }

    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError, Self)> {
        use itertools::EitherOrBoth::{Both, Left};
        let (tmp, tmp_off) = skip_space(self.slice);
        let tmp_cur = self.advance_by(tmp_off); // for errors
        let (lit, _) = skip_space(lit);
        let inp_parts = LITERAL_PART_RE.find_iter(tmp);
        let lit_parts = LITERAL_PART_RE.find_iter(lit);
        let mut last_pos = 0;
        for ilp in inp_parts.zip_longest(lit_parts) {
            let (i1, ip, lp) = match ilp {
                Both((i0, i1), (l0, l1)) => (i1, &tmp[i0..i1], &lit[l0..l1]),
                Left(_) => break,
                _ => return Err((ScanError::literal_mismatch(tmp_cur.offset()), self))
            };
            if ip != lp {
                return Err((ScanError::literal_mismatch(tmp_cur.offset()), self));
            }
            last_pos = i1;
        }
        Ok(self.advance_by(tmp_off + last_pos))
    }
}

/**
Skip all leading whitespace in a string, and return both the resulting slice and the number of bytes skipped.
*/
fn skip_space(s: &str) -> (&str, usize) {
    let off = s.char_indices()
        .take_while(|&(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    (&s[off..], off)
}

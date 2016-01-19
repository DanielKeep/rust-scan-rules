use std::ops::Deref;
use itertools::Itertools;
use regex::Regex;
use ::{ScanError, ScanErrorKind};

lazy_static! {
    static ref LITERAL_PART_RE: Regex = Regex::new(r"(\w+|\S)").unwrap();
}

pub trait ScanInput<'a>: Sized {
    fn try_end(self) -> Result<(), (ScanError<'a>, Self)>;

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError<'a>, Self)>;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Cursor<'a> {
    offset: usize,
    slice: &'a str,
}

impl<'a> Cursor<'a> {
    pub fn new_with_offset(slice: &'a str, offset: usize) -> Self {
        Cursor {
            offset: offset,
            slice: slice,
        }
    }

    pub fn advance_by(self, bytes: usize) -> Self {
        Cursor {
            offset: self.offset + bytes,
            slice: &self.slice[bytes..],
        }
    }

    pub fn as_str(self) -> &'a str {
        self.slice
    }

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
    fn try_end(self) -> Result<(), (ScanError<'a>, Self)> {
        if (skip_space(self.slice).0).len() == 0 {
            Ok(())
        } else {
            Err((ScanError::expected_end(self), self))
        }
    }

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        let (tmp, tmp_off) = skip_space(self.slice);
        match f(tmp) {
            Ok((out, off)) => Ok((out, self.advance_by(tmp_off + off))),
            Err(err) => Err((ScanError::new(self.advance_by(tmp_off), err), self))
        }
    }

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        match f(self.slice) {
            Ok((out, off)) => Ok((out, self.advance_by(off))),
            Err(err) => Err((ScanError::new(self, err), self))
        }
    }

    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError<'a>, Self)> {
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
                _ => return Err((ScanError::literal_mismatch(tmp_cur), self))
            };
            if ip != lp {
                return Err((ScanError::literal_mismatch(tmp_cur), self));
            }
            last_pos = i1;
        }
        Ok(self.advance_by(tmp_off + last_pos))
    }
}

fn skip_space(s: &str) -> (&str, usize) {
    let off = s.char_indices()
        .take_while(|&(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    (&s[off..], off)
}

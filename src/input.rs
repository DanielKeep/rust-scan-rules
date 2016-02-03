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

The short version is this:

* Values provided as input to the user-facing scanning macros must implement `IntoScanCursor`, which converts them into something that implements `ScanCursor`.

* The input provided to actual type scanners will be something that implements the `ScanInput` trait.

`IntoScanCursor` will be of interest if you are implementing a type which you want to be scannable.  `StrCursor` will be of interest if you want to construct a specialised cursor.  `ScanCursor` will be of interest if you are using a `^..cursor` pattern to capture a cursor.
*/
use std::borrow::Cow;
use std::marker::PhantomData;
use itertools::Itertools;
use regex::Regex;
use ::ScanError;

lazy_static! {
    /**
    This regex defines what the default literal matching code considers a "part" for the purposes of comparison.

    Specifically, it should be a contiguous sequence of letters *or* a single, non-whitespace character.
    */
    static ref LITERAL_PART_RE: Regex = Regex::new(r"(\w+|\S)").unwrap();
}

/**
Conversion into a `ScanCursor`.

This is a helper trait used to convert different values into a scannable cursor type.  Implement this if you want your type to be usable as input to one of the scanning macros.
*/
pub trait IntoScanCursor<'a>: Sized {
    /**
    The corresponding scannable cursor type.
    */
    type Output: 'a + ScanCursor<'a>;

    /**
    Convert this into a scannable cursor.
    */
    fn into_scan_cursor(self) -> Self::Output;
}

impl<'a, T> IntoScanCursor<'a> for T where T: 'a + ScanCursor<'a> {
    type Output = Self;
    fn into_scan_cursor(self) -> Self::Output {
        self
    }
}

impl<'a> IntoScanCursor<'a> for &'a str {
    type Output = StrCursor<'a>;
    fn into_scan_cursor(self) -> Self::Output {
        StrCursor::new(self)
    }
}

impl<'a> IntoScanCursor<'a> for &'a String {
    type Output = StrCursor<'a>;
    fn into_scan_cursor(self) -> Self::Output {
        StrCursor::new(self)
    }
}

impl<'a> IntoScanCursor<'a> for &'a Cow<'a, str> {
    type Output = StrCursor<'a>;
    fn into_scan_cursor(self) -> Self::Output {
        StrCursor::new(self)
    }
}

/**
This trait defines the interface to input values that can be scanned.
*/
pub trait ScanCursor<'a>: 'a + Sized + Clone {
    /**
    Corresponding scan input type.
    */
    type ScanInput: ScanInput<'a>;

    /**
    Assert that the input has been exhausted, or that the current position is a valid place to "stop".
    */
    fn try_end(self) -> Result<(), (ScanError, Self)>;

    /**
    Scan a value from the current position.  The closure will be called with all available input, and is expected to return *either* the scanned value, and the number of bytes of input consumed, *or* a reason why scanning failed.

    The input will have all leading whitespace removed, if applicable.
    */
    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(Self::ScanInput) -> Result<(Out, usize), ScanError>;

    /**
    Performs the same task as [`try_scan`](#tymethod.try_scan), except that it *does not* perform whitespace stripping.
    */
    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(Self::ScanInput) -> Result<(Out, usize), ScanError>;

    /**
    Match the provided literal term against the input.

    Implementations are free to interpret "match" as they please.
    */
    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError, Self)>;

    /**
    Returns the remaining input as a string slice.
    */
    fn as_str(self) -> &'a str;

    /**
    Returns the number of bytes consumed by this cursor since its creation.
    */
    fn offset(&self) -> usize;
}

/**
This trait is the interface scanners use to access the input being scanned.
*/
pub trait ScanInput<'a>: 'a + Sized + Clone {
    /**
    Corresponding cursor type.
    */
    type ScanCursor: ScanCursor<'a>;

    /**
    Marker type used to do string comparisons.
    */
    type StrCompare: StrCompare;

    /**
    Get the contents of the input as a string slice.
    */
    fn as_str(&self) -> &'a str;

    /**
    Create a new input from a subslice of *this* input's contents.

    This should be used to ensure that additional state and settings (such as the string comparison marker) are preserved.
    */
    fn from_subslice(&self, subslice: &'a str) -> Self;

    /**
    Turn the input into an independent cursor, suitable for feeding back into a user-facing scanning macro.
    */
    fn to_cursor(&self) -> Self::ScanCursor;
}

/**
Basic cursor implementation wrapping a string slice.

The `Cmp` parameter can be used to control the string comparison logic used.
*/
#[derive(Debug)]
pub struct StrCursor<'a, Cmp=ExactCompare>
where Cmp: StrCompare {
    offset: usize,
    slice: &'a str,
    _marker: PhantomData<Cmp>,
}

/*
These have to be spelled out to avoid erroneous constraints on the type parameters.
*/
impl<'a, Cmp> Copy for StrCursor<'a, Cmp>
where Cmp: StrCompare {}

impl<'a, Cmp> Clone for StrCursor<'a, Cmp>
where Cmp: StrCompare {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Cmp> StrCursor<'a, Cmp>
where Cmp: StrCompare {
    /**
    Construct a new `StrCursor` with a specific `offset`.

    The `offset` is logically the number of bytes which have already been consumed from the original input; these already-consumed bytes *must not* be included in `slice`.
    */
    pub fn new(slice: &'a str) -> Self {
        StrCursor {
            offset: 0,
            slice: slice,
            _marker: PhantomData,
        }
    }

    /**
    Advance the cursor by the given number of bytes.
    */
    fn advance_by(self, bytes: usize) -> Self {
        StrCursor {
            offset: self.offset + bytes,
            slice: &self.slice[bytes..],
            _marker: PhantomData,
        }
    }

    /**
    Returns the number of bytes of input that have been consumed by this `StrCursor`.
    */
    fn offset(self) -> usize {
        self.offset
    }
}

impl<'a, Cmp> ScanCursor<'a> for StrCursor<'a, Cmp>
where Cmp: StrCompare {
    type ScanInput = Self;

    fn try_end(self) -> Result<(), (ScanError, Self)> {
        if (skip_space(self.slice).0).len() == 0 {
            Ok(())
        } else {
            Err((ScanError::expected_end().add_offset(self.offset()), self))
        }
    }

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(Self::ScanInput) -> Result<(Out, usize), ScanError> {
        let (_, tmp_off) = skip_space(self.slice);
        let tmp = self.advance_by(tmp_off);
        match f(tmp) {
            Ok((out, off)) => Ok((out, tmp.advance_by(off))),
            Err(err) => Err((err.add_offset(tmp.offset()), self)),
        }
    }

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(Self::ScanInput) -> Result<(Out, usize), ScanError> {
        match f(self) {
            Ok((out, off)) => Ok((out, self.advance_by(off))),
            Err(err) => Err((err.add_offset(self.offset()), self)),
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
                _ => return Err((ScanError::literal_mismatch().add_offset(tmp_cur.offset()), self))
            };
            if !Cmp::compare(ip, lp) {
                return Err((ScanError::literal_mismatch().add_offset(tmp_cur.offset()), self));
            }
            last_pos = i1;
        }
        Ok(self.advance_by(tmp_off + last_pos))
    }

    fn as_str(self) -> &'a str {
        self.slice
    }

    fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a, Cmp> ScanInput<'a> for StrCursor<'a, Cmp>
where Cmp: StrCompare {
    type ScanCursor = Self;
    type StrCompare = Cmp;

    fn as_str(&self) -> &'a str {
        self.slice
    }

    fn from_subslice(&self, subslice: &'a str) -> Self {
        use ::util::StrUtil;
        let offset = self.as_str().subslice_offset_stable(subslice)
            .expect("called `StrCursor::from_subslice` with disjoint subslice");

        StrCursor {
            offset: self.offset + offset,
            slice: subslice,
            _marker: PhantomData,
        }
    }

    fn to_cursor(&self) -> Self::ScanCursor {
        /*
        Note that we strip the offset information here, essentially making this a *new* cursor, not just a copy of the existing one.
        */
        StrCursor::new(self.slice)
    }
}

/**
This implementation is provided to allow scanners to be used manually with a minimum of fuss.

It *only* supports direct, exact equality comparison.
*/
impl<'a> ScanInput<'a> for &'a str {
    type ScanCursor = StrCursor<'a>;
    type StrCompare = ExactCompare;

    fn as_str(&self) -> &'a str {
        *self
    }

    fn from_subslice(&self, subslice: &'a str) -> Self {
        subslice
    }

    fn to_cursor(&self) -> Self::ScanCursor {
        self.into_scan_cursor()
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

/**
Defines an interface for comparing two strings for equality.

This is used to allow `StrCursor` to be parametrised on different kinds of string comparisons: case-sensitive, case-insensitive, canonicalising, *etc.*
*/
pub trait StrCompare: 'static {
    /**
    Compare two strings and return `true` if they should be considered "equal".
    */
    fn compare(a: &str, b: &str) -> bool;
}

/**
Marker type used to do exact, byte-for-byte string comparisons.

This is likely the fastest kind of string comparison, and matches the default behaviour of the `==` operator on strings.
*/
#[derive(Debug)]
pub enum ExactCompare {}

impl StrCompare for ExactCompare {
    fn compare(a: &str, b: &str) -> bool {
        a == b
    }
}

/**
Marker type used to do ASCII case-insensitive string comparisons.

Note that this is *only correct* for pure, ASCII-only strings.  To get less incorrect case-insensitive comparisons, you will need to use a Unicode-aware comparison.

This exists because ASCII-only case conversions are easily understood and relatively fast.
*/
#[derive(Debug)]
pub enum IgnoreAsciiCase {}

impl StrCompare for IgnoreAsciiCase {
    fn compare(a: &str, b: &str) -> bool {
        use std::ascii::AsciiExt;
        a.eq_ignore_ascii_case(b)
    }
}

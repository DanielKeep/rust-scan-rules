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
use ::ScanError;

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
pub struct StrCursor<'a, Cmp=ExactCompare, Space=IgnoreSpace, Word=Wordish>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{
    offset: usize,
    slice: &'a str,
    _marker: PhantomData<(Cmp, Space, Word)>,
}

/*
These have to be spelled out to avoid erroneous constraints on the type parameters.
*/
impl<'a, Cmp, Space, Word>
Copy for StrCursor<'a, Cmp, Space, Word>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{}

impl<'a, Cmp, Space, Word>
Clone for StrCursor<'a, Cmp, Space, Word>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Cmp, Space, Word>
StrCursor<'a, Cmp, Space, Word>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{
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

impl<'a, Cmp, Space, Word>
ScanCursor<'a> for StrCursor<'a, Cmp, Space, Word>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{
    type ScanInput = Self;

    fn try_end(self) -> Result<(), (ScanError, Self)> {
        if Space::skip_space(self.slice) == self.slice.len() {
            Ok(())
        } else {
            Err((ScanError::expected_end().add_offset(self.offset()), self))
        }
    }

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError, Self)>
    where F: FnOnce(Self::ScanInput) -> Result<(Out, usize), ScanError> {
        let tmp_off = Space::skip_space(self.slice);
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
        let mut tmp_off = Space::skip_space(self.slice);
        let mut tmp = &self.slice[tmp_off..];
        let mut lit = lit;

        while lit.len() > 0 {
            // Match leading spaces.
            match Space::match_spaces(tmp, lit) {
                Ok((a, b)) => {
                    tmp = &tmp[a..];
                    tmp_off += a;
                    lit = &lit[b..];
                },
                Err(off) => {
                    return Err((
                        ScanError::literal_mismatch()
                            .add_offset(self.offset() + tmp_off + off),
                        self
                    ));
                },
            }

            if lit.len() == 0 { break; }

            // Pull out the leading wordish things.
            let lit_word = match Word::slice_word(lit) {
                Some(0) | None => panic!("literal {:?} begins with a non-space, non-word", lit),
                Some(b) => &lit[..b],
            };
            let tmp_word = match Word::slice_word(tmp) {
                Some(b) => &tmp[..b],
                None => return Err((
                    ScanError::literal_mismatch()
                        .add_offset(self.offset() + tmp_off),
                    self
                )),
            };

            if !Cmp::compare(tmp_word, lit_word) {
                return Err((
                    ScanError::literal_mismatch()
                        .add_offset(self.offset() + tmp_off),
                    self
                ));
            }

            tmp = &tmp[tmp_word.len()..];
            tmp_off += tmp_word.len();
            lit = &lit[lit_word.len()..];
        }

        Ok(self.advance_by(tmp_off))
    }

    fn as_str(self) -> &'a str {
        self.slice
    }

    fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a, Cmp, Space, Word>
ScanInput<'a> for StrCursor<'a, Cmp, Space, Word>
where
    Cmp: StrCompare,
    Space: SkipSpace,
    Word: SliceWord,
{
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
Defines an interface for skipping whitespace.
*/
pub trait SkipSpace: 'static {
    /**
    Given two strings, does the leading whitespace match?

    If so, how many leading bytes from each should be dropped?

    If not, after many bytes into `a` do they disagree?
    */
    fn match_spaces(a: &str, b: &str) -> Result<(usize, usize), usize>;

    /**
    Return the number of bytes of leading whitespace in `a` that should be skipped.
    */
    fn skip_space(a: &str) -> usize;
}

/**
Matches all whitespace *exactly*, and does not skip any.
*/
#[derive(Debug)]
pub enum ExactSpace {}

impl SkipSpace for ExactSpace {
    fn match_spaces(a: &str, b: &str) -> Result<(usize, usize), usize> {
        let mut acs = a.char_indices();
        let mut bcs = b.char_indices();
        let (mut last_ai, mut last_bi) = (0, 0);
        while let (Some((ai, ac)), Some((bi, bc))) = (acs.next(), bcs.next()) {
            if !ac.is_whitespace() {
                return Ok((ai, bi));
            } else if ac != bc {
                return Err(ai);
            } else {
                last_ai = ai + ac.len_utf8();
                last_bi = bi + ac.len_utf8();
            }
        }
        Ok((last_ai, last_bi))
    }

    fn skip_space(_: &str) -> usize {
        0
    }
}

#[cfg(test)]
#[test]
fn test_exact_space() {
    use self::ExactSpace as ES;

    assert_eq!(ES::match_spaces("", ""), Ok((0, 0)));
    assert_eq!(ES::match_spaces(" ", " "), Ok((1, 1)));
    assert_eq!(ES::match_spaces(" x", " x"), Ok((1, 1)));
    assert_eq!(ES::match_spaces(" ", " x"), Ok((1, 1)));
    assert_eq!(ES::match_spaces(" x", " "), Ok((1, 1)));
    assert_eq!(ES::match_spaces(" \t ", "   "), Err(1));
}

/**
Requires that whitespace in the pattern exists in the input, but the exact *kind* of space doesn't matter.
*/
#[derive(Debug)]
pub enum FuzzySpace {}

impl SkipSpace for FuzzySpace {
    fn match_spaces(inp: &str, pat: &str) -> Result<(usize, usize), usize> {
        let (_, a_off) = skip_space(inp);
        let (_, b_off) = skip_space(pat);

        match (a_off, b_off) {
            (0, 0) => Ok((0, 0)),
            (a, b) if a != 0 && b != 0 => Ok((a, b)),
            (_, _) => Err(0),
        }
    }

    fn skip_space(_: &str) -> usize {
        0
    }
}

#[cfg(test)]
#[test]
fn test_fuzzy_space() {
    use self::FuzzySpace as FS;

    assert_eq!(FS::match_spaces("x", "x"), Ok((0, 0)));
    assert_eq!(FS::match_spaces(" x", " x"), Ok((1, 1)));
    assert_eq!(FS::match_spaces("  x", " x"), Ok((2, 1)));
    assert_eq!(FS::match_spaces(" x", "  x"), Ok((1, 2)));
    assert_eq!(FS::match_spaces("\tx", " x"), Ok((1, 1)));
    assert_eq!(FS::match_spaces(" x", "\tx"), Ok((1, 1)));
    assert_eq!(FS::match_spaces("x", " x"), Err(0));
    assert_eq!(FS::match_spaces(" x", "x"), Err(0));
}

/**
Ignores all whitespace *other* than line breaks.
*/
#[derive(Debug)]
pub enum IgnoreNonLine {}

impl SkipSpace for IgnoreNonLine {
    fn match_spaces(a: &str, b: &str) -> Result<(usize, usize), usize> {
        let a_off = skip_space_non_line(a);
        let b_off = skip_space_non_line(b);
        Ok((a_off, b_off))
    }

    fn skip_space(s: &str) -> usize {
        skip_space_non_line(s)
    }
}

fn skip_space_non_line(s: &str) -> usize {
    s.char_indices()
        .take_while(|&(_, c)| c.is_whitespace()
            && c != '\r' && c != '\n')
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0)
}

/**
Ignores all whitespace entirely.
*/
#[derive(Debug)]
pub enum IgnoreSpace {}

impl SkipSpace for IgnoreSpace {
    fn match_spaces(a: &str, b: &str) -> Result<(usize, usize), usize> {
        let (_, a_off) = skip_space(a);
        let (_, b_off) = skip_space(b);
        Ok((a_off, b_off))
    }

    fn skip_space(s: &str) -> usize {
        s.char_indices()
            .take_while(|&(_, c)| c.is_whitespace())
            .map(|(i, c)| i + c.len_utf8())
            .last()
            .unwrap_or(0)
    }
}

/**
Defines an interface for slicing words out of input and literal text.
*/
pub trait SliceWord: 'static {
    /**
    If `s` starts with a word, how long is it?
    */
    fn slice_word(s: &str) -> Option<usize>;
}

/**
Treat any contiguous sequence of non-space characters (according to Unicode's definition of the `\s` regular expression class) as a word.
*/
#[derive(Debug)]
pub enum NonSpace {}

impl SliceWord for NonSpace {
    fn slice_word(s: &str) -> Option<usize> {
        slice_non_space(s)
    }
}

/**
Treat any contiguous sequence of "word" characters (according to Unicode's definition of the `\w` regular expression class) *or* any other single character as a word.
*/
#[derive(Debug)]
pub enum Wordish {}

impl SliceWord for Wordish {
    fn slice_word(s: &str) -> Option<usize> {
        slice_wordish(s)
    }
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
Marker type used to do case-insensitive string comparisons.

Note that this *does not* take any locale information into account.  It is only as correct as a call to `char::to_lowercase`.
*/
#[derive(Debug)]
pub enum IgnoreCase {}

impl StrCompare for IgnoreCase {
    fn compare(a: &str, b: &str) -> bool {
        let mut acs = a.chars().flat_map(char::to_lowercase);
        let mut bcs = b.chars().flat_map(char::to_lowercase);
        loop {
            match (acs.next(), bcs.next()) {
                (Some(a), Some(b)) if a == b => (),
                (None, None) => return true,
                _ => return false
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_ignore_case() {
    use self::IgnoreCase as IC;

    assert_eq!(IC::compare("hi", "hi"), true);
    assert_eq!(IC::compare("Hi", "hI"), true);
    assert_eq!(IC::compare("hI", "Hi"), true);
    assert_eq!(IC::compare("ẞß", "ßẞ"), true);
    assert_eq!(IC::compare("ßẞ", "ẞß"), true);
}

/**
Marker type used to do case-insensitive, normalized string comparisons.

Specifically, this type will compare strings based on the result of a NFD transform, followed by conversion to lower-case.

Note that this *does not* take any locale information into account.  It is only as correct as a call to `char::to_lowercase`.
*/
#[cfg(feature="unicode-normalization")]
#[derive(Debug)]
pub enum IgnoreCaseNormalized {}

#[cfg(feature="unicode-normalization")]
impl StrCompare for IgnoreCaseNormalized {
    fn compare(a: &str, b: &str) -> bool {
        use unicode_normalization::UnicodeNormalization;

        let mut acs = a.nfd().flat_map(char::to_lowercase);
        let mut bcs = b.nfd().flat_map(char::to_lowercase);
        loop {
            match (acs.next(), bcs.next()) {
                (Some(a), Some(b)) if a == b => (),
                (None, None) => return true,
                _ => return false
            }
        }
    }
}

#[cfg(feature="unicode-normalization")]
#[cfg(test)]
#[test]
fn test_ignore_case_normalized() {
    use self::IgnoreCaseNormalized as ICN;

    assert_eq!(ICN::compare("hi", "hi"), true);
    assert_eq!(ICN::compare("Hi", "hI"), true);
    assert_eq!(ICN::compare("hI", "Hi"), true);
    assert_eq!(ICN::compare("café", "cafe\u{301}"), true);
    assert_eq!(ICN::compare("cafe\u{301}", "café"), true);
    assert_eq!(ICN::compare("CafÉ", "CafE\u{301}"), true);
    assert_eq!(ICN::compare("CAFÉ", "cafe\u{301}"), true);
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

/**
Marker type used to do normalized string comparisons.

Specifically, this type will compare strings based on the result of a NFD transform.
*/
#[cfg(feature="unicode-normalization")]
#[derive(Debug)]
pub enum Normalized {}

#[cfg(feature="unicode-normalization")]
impl StrCompare for Normalized {
    fn compare(a: &str, b: &str) -> bool {
        use unicode_normalization::UnicodeNormalization;

        let mut acs = a.nfd();
        let mut bcs = b.nfd();
        loop {
            match (acs.next(), bcs.next()) {
                (Some(a), Some(b)) if a == b => (),
                (None, None) => return true,
                _ => return false
            }
        }
    }
}

#[cfg(feature="unicode-normalization")]
#[cfg(test)]
#[test]
fn test_normalized() {
    use self::Normalized as N;

    assert_eq!(N::compare("hi", "hi"), true);
    assert_eq!(N::compare("café", "cafe\u{301}"), true);
    assert_eq!(N::compare("cafe\u{301}", "café"), true);
}

fn slice_non_space(s: &str) -> Option<usize> {
    use ::util::TableUtil;
    use ::unicode::property::White_Space_table as WS;

    s.char_indices()
        .take_while(|&(_, c)| !WS.span_table_contains(&c))
        .map(|(i, c)| i + c.len_utf8())
        .last()
}

fn slice_wordish(s: &str) -> Option<usize> {
    use ::util::TableUtil;
    use ::unicode::regex::PERLW;

    let word_len = s.char_indices()
        .take_while(|&(_, c)| PERLW.span_table_contains(&c))
        .map(|(i, c)| i + c.len_utf8())
        .last();

    match word_len {
        Some(n) => Some(n),
        None => s.chars().next().map(|c| c.len_utf8()),
    }
}

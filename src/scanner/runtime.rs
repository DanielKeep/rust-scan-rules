/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Types and constructors for various runtime scanners.
*/
use std::marker::PhantomData;
use strcursor::StrCursor;
use ::ScanError;
use ::input::ScanInput;
use ::scanner::{ScanFromStr, ScanStr};

#[cfg(feature="regex")] use regex::Regex;

/**
Creates a runtime scanner that forces *exactly* `width` bytes to be consumed.

This is done in two steps: first, it truncates the input provided to the inner scanner to exactly `width` bytes.  Secondly, it verifies that the inner scanner consumed all of the truncated input.

See: [`exact_width_a`](fn.exact_width_a.html).
*/
pub fn exact_width<Then>(width: usize, then: Then) -> ExactWidth<Then> {
    ExactWidth(width, then)
}

/**
Creates a runtime scanner that forces *exactly* `width` bytes to be consumed by the static scanner `S`.

See: [`exact_width`](fn.exact_width.html).
*/
pub fn exact_width_a<S>(width: usize) -> ExactWidth<ScanA<S>> {
    exact_width(width, scan_a::<S>())
}

/**
Runtime scanner that forces *exactly* `width` bytes to be consumed.

See: [`exact_width`](fn.exact_width.html), [`exact_width_a`](fn.exact_width_a.html).
*/
pub struct ExactWidth<Then>(usize, Then);

impl<'a, Then> ScanStr<'a> for ExactWidth<Then>
where Then: ScanStr<'a> {
    type Output = Then::Output;

    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError> {
        let s_str = s.as_str();
        if s_str.len() < self.0 {
            return Err(ScanError::syntax("input not long enough"));
        }

        let sl = s.from_subslice(&s_str[..self.0]);

        match self.1.scan(sl) {
            Ok((_, n)) if n != self.0 => Err(ScanError::syntax("value did not consume enough characters")),
            Err(err) => Err(err),
            Ok((v, _)) => Ok((v, self.0))
        }
    }

    fn wants_leading_junk_stripped(&self) -> bool {
        self.1.wants_leading_junk_stripped()
    }
}

#[cfg(test)]
#[test]
fn test_exact_width() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;
    use ::scanner::Word;
    let scan = exact_width_a::<Word>;

    assert_match!(scan(2).scan(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("a"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("a b"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("ab"), Ok(("ab", 2)));
    assert_match!(scan(2).scan("abc"), Ok(("ab", 2)));
}

/**
Creates a runtime scanner that forces *at most* `width` bytes to be consumed.

This is done by truncating the input provided to the inner scanner to at most `width` bytes.

See: [`max_width_a`](fn.max_width_a.html).
*/
pub fn max_width<Then>(width: usize, then: Then) -> MaxWidth<Then> {
    MaxWidth(width, then)
}

/**
Creates a runtime scanner that forces *at most* `width` bytes to be consumed by the static scanner `S`.

See: [`max_width`](fn.max_width.html).
*/
pub fn max_width_a<S>(width: usize) -> MaxWidth<ScanA<S>> {
    max_width(width, scan_a::<S>())
}

/**
Runtime scanner that forces *at most* `width` bytes to be consumed.

See: [`max_width`](fn.max_width.html), [`max_width_a`](fn.max_width_a.html).
*/
pub struct MaxWidth<Then>(usize, Then);

impl<'a, Then> ScanStr<'a> for MaxWidth<Then>
where Then: ScanStr<'a> {
    type Output = Then::Output;

    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError> {
        let s_str = s.as_str();
        let len = ::std::cmp::min(s_str.len(), self.0);
        let stop = StrCursor::new_at_left_of_byte_pos(s_str, len);
        let sl = s.from_subslice(stop.slice_before());

        self.1.scan(sl)
    }

    fn wants_leading_junk_stripped(&self) -> bool {
        self.1.wants_leading_junk_stripped()
    }
}

#[cfg(test)]
#[test]
fn test_max_width() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;
    use ::scanner::Word;
    let scan = max_width_a::<Word>;

    assert_match!(scan(2).scan(""), Err(SE { kind: SEK::SyntaxNoMessage, .. }));
    assert_match!(scan(2).scan("a"), Ok(("a", 1)));
    assert_match!(scan(2).scan("a b"), Ok(("a", 1)));
    assert_match!(scan(2).scan("ab"), Ok(("ab", 2)));
    assert_match!(scan(2).scan("abc"), Ok(("ab", 2)));
}

/**
Creates a runtime scanner that forces *at least* `width` bytes to be consumed.

This is done by verifying the inner scanner consumed at least `width` bytes.

See: [`min_width_a`](fn.min_width_a.html).
*/
pub fn min_width<Then>(width: usize, then: Then) -> MinWidth<Then> {
    MinWidth(width, then)
}

/**
Creates a runtime scanner that forces *at least* `width` bytes to be consumed by the static scanner `S`.

See: [`min_width`](fn.min_width.html).
*/
pub fn min_width_a<S>(width: usize) -> MinWidth<ScanA<S>> {
    min_width(width, scan_a::<S>())
}

/**
Runtime scanner that forces *at least* `width` bytes to be consumed.

See: [`min_width`](fn.min_width.html), [`min_width_a`](fn.min_width_a.html).
*/
pub struct MinWidth<Then>(usize, Then);

impl<'a, Then> ScanStr<'a> for MinWidth<Then>
where Then: ScanStr<'a> {
    type Output = Then::Output;

    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError> {
        let s_str = s.as_str();
        if s_str.len() < self.0 {
            return Err(ScanError::syntax("expected more bytes to scan"));
        }
        match self.1.scan(s) {
            Ok((_, n)) if n < self.0 => Err(ScanError::syntax("scanned value too short")),
            other => other
        }
    }

    fn wants_leading_junk_stripped(&self) -> bool {
        self.1.wants_leading_junk_stripped()
    }
}

#[cfg(test)]
#[test]
fn test_min_width() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;
    use ::scanner::Word;
    let scan = min_width_a::<Word>;

    assert_match!(scan(2).scan(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("a"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("a b"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan(2).scan("ab"), Ok(("ab", 2)));
    assert_match!(scan(2).scan("abc"), Ok(("abc", 3)));
}

/**
Creates a runtime scanner that extracts a slice of the input using a regular expression, then scans the result using `Then`.

**Note**: requires the `regex` feature.

If the regular expression defines a group named `scan`, then it will extract the contents of that group.  Failing that, it will use the the first capturing group.  If there are no capturing groups, it will extract the entire match.

Irrespective of the amount of input provided by the regex scanner to the inner scanner, the regex scanner will only consume the portion that the inner scanner did.

Note that this scanner *does not* respect the case sensitivity of the input.

See: [`regex` crate](http://doc.rust-lang.org/regex/regex/index.html), [`re_a`](fn.re_a.html), [`re_str`](fn.re_str.html).
*/
#[cfg(feature="regex")]
pub fn re<Then>(s: &str, then: Then) -> ScanRegex<Then> {
    ScanRegex(Regex::new(s).unwrap(), then)
}

/**
Creates a runtime regex scanner that passes the matched input to a static scanner `S`.

**Note**: requires the `regex` feature.

See: [`re`](fn.re_a.html).
*/
#[cfg(feature="regex")]
pub fn re_a<S>(s: &str) -> ScanRegex<ScanA<S>> {
    re(s, scan_a::<S>())
}

/**
Creates a runtime regex scanner that yields the matched input as a string slice.

**Note**: requires the `regex` feature.

See: [`re`](fn.re_a.html).
*/
#[cfg(feature="regex")]
pub fn re_str(s: &str) -> ScanRegex<ScanA<::scanner::Everything<&str>>> {
    re_a::<::scanner::Everything<&str>>(s)
}

/**
Runtime scanner that slices the input based on a regular expression.

**Note**: requires the `regex` feature.

See: [`re`](../fn.re.html), [`re_a`](../fn.re_a.html), [`re_str`](../fn.re_str.html).
*/
#[cfg(feature="regex")]
pub struct ScanRegex<Then>(Regex, Then);

#[cfg(feature="regex")]
impl<'a, Then> ScanStr<'a> for ScanRegex<Then>
where Then: ScanStr<'a> {
    type Output = Then::Output;

    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError> {
        let s_str = s.as_str();
        let cap = match self.0.captures(s_str) {
            None => return Err(ScanError::syntax("no match for regular expression")),
            Some(cap) => cap,
        };

        let cover = match cap.pos(0) {
            None => return Err(ScanError::syntax("no match for regular expression")),
            Some(pos) => pos,
        };

        let sl = if let Some(sl) = cap.name("scan") {
            sl
        } else if let Some((a, b)) = cap.pos(1) {
            &s_str[a..b]
        } else {
            &s_str[cover.0 .. cover.1]
        };

        let sl = s.from_subslice(sl);

        match self.1.scan(sl) {
            Ok((v, _)) => Ok((v, cover.1)),
            Err(err) => Err(err),
        }
    }

    fn wants_leading_junk_stripped(&self) -> bool {
        self.1.wants_leading_junk_stripped()
    }
}

#[cfg(feature="regex")]
#[cfg(test)]
#[test]
fn test_re() {
    use ::ScanError as SE;
    use ::ScanErrorKind as SEK;
    let scan = re_str;

    assert_match!(scan("[a-z][0-9]").scan(""), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan("[a-z][0-9]").scan("a"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan("[a-z][0-9]").scan("a 0"), Err(SE { kind: SEK::Syntax(_), .. }));
    assert_match!(scan("[a-z][0-9]").scan("a0"), Ok(("a0", 2)));
    assert_match!(scan("[a-z][0-9]").scan("a0c"), Ok(("a0", 2)));
    assert_match!(scan("[a-z][0-9]").scan(" a0"), Ok(("a0", 3)));
}

/**
Returns a runtime scanner that delegates to a static scanner.
*/
pub fn scan_a<S>() -> ScanA<S> {
    ScanA(PhantomData)
}

/**
Runtime scanner that delegates to a static scanner.

See: [`scan_a`](../fn.scan_a.html).
*/
pub struct ScanA<S>(PhantomData<S>);

impl<'a, S> ScanStr<'a> for ScanA<S>
where S: ScanFromStr<'a> {
    type Output = S::Output;

    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError> {
        <S as ScanFromStr<'a>>::scan_from(s)
    }

    fn wants_leading_junk_stripped(&self) -> bool {
        <S as ScanFromStr<'a>>::wants_leading_junk_stripped()
    }
}

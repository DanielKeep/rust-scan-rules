//! This module contains items that need to be publicly exposed for macros, but we don't want to expose as part of the stable, public interface.
//!
//! Nothing** in this module is subject to semver restrictions.  Use of *any* symbol in this module from *outside* this crate is a Bad Idea, and it will be your own damn fault when your code breaks.
//!
//! You will get no sympathy, only laughter.
//!
#![doc(hidden)]
use ScanError;

/**
Remove a single trailing line terminator from `s`.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
pub fn strip_line_term(s: &str) -> &str {
    if s.ends_with("\r\n") {
        &s[0..s.len() - 2]
    } else if s.ends_with("\n") {
        &s[0..s.len() - 1]
    } else if s.ends_with("\r") {
        &s[0..s.len() - 1]
    } else {
        s
    }
}

/**
Compute the offset of `b`, which must be a subslice of `a`.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
pub fn subslice_offset(a: &str, b: &str) -> Option<usize> {
    use util::StrUtil;
    a.subslice_offset_stable(b)
}

/**
Dispatch to a runtime scanner.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
pub fn try_scan_runtime<'a, C, S>(cur: C, scan: &mut S) -> Result<(S::Output, C), (ScanError, C)>
    where C: ::input::ScanCursor<'a>,
          S: ::scanner::ScanStr<'a>
{
    if scan.wants_leading_junk_stripped() {
        cur.try_scan(|s| scan.scan(s))
    } else {
        cur.try_scan_raw(|s| scan.scan(s))
    }
}

/**
Dispatch to a static abstract scanner.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
pub fn try_scan_static<'a, C, S>(cur: C) -> Result<(S::Output, C), (ScanError, C)>
    where C: ::input::ScanCursor<'a>,
          S: ::scanner::ScanFromStr<'a>
{
    if S::wants_leading_junk_stripped() {
        cur.try_scan(S::scan_from)
    } else {
        cur.try_scan_raw(S::scan_from)
    }
}

/**
Dispatch to a static self scanner.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
pub fn try_scan_static_self<'a, C, S>(cur: C) -> Result<(S, C), (ScanError, C)>
    where C: ::input::ScanCursor<'a>,
          S: ::scanner::ScanSelfFromStr<'a>
{
    if S::wants_leading_junk_stripped() {
        cur.try_scan(S::scan_self_from)
    } else {
        cur.try_scan_raw(S::scan_self_from)
    }
}

/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This module defines various abstract scanners that can be used to scan other types with particular properties, or under custom parsing rules.

It is also where implementations for existing standard and external types are kept, though these do not appear in the documentation.
*/
pub use self::misc::{
    Everything, NonSpace,
    Ident, Line, Number, Word, Wordish,
    KeyValuePair, QuotedString,
    Binary, Octal, Hex,
};

#[macro_use] mod macros;

pub mod util;

mod lang;
mod misc;
mod std;

use ::ScanErrorKind;

/**
This trait defines the interface to a type which can be scanned.

The exact syntax scanned is entirely arbitrary, though there are some rules of thumb that implementations should *generally* stick to:

* Do not ignore leading whitespace.
* Do not eagerly consume trailing whitespace, unless it is legitimately part of the scanned syntax.

In addition, if you are implementing scanning directly for the result type (*i.e.* `Output = Self`), prefer parsing *only* the result of the type's `Debug` implementation.  This ensures that there is a degree of round-tripping between `format!` and `scan!`.

If a type has multiple legitimate parsing forms, consider defining those alternate forms on abstract scanner types (*i.e.* `Output != Self`) instead.

See: [`ScanSelfFromStr`](trait.ScanSelfFromStr.html).
*/
pub trait ScanFromStr<'a>: Sized {
    /**
    The type that the implementation scans into.  This *does not* have to be the same as the implementing type, although it typically *will* be.

    See: [`ScanSelfFromStr::scan_self_from`](trait.ScanSelfFromStr.html#method.scan_self_from).
    */
    type Output;

    /**
    Perform a scan on the given input.

    Implementations must return *either* the scanned value, and the number of bytes consumed from the input, *or* a reason why scanning failed.
    */
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind>;
}

/**
This is a convenience trait automatically implemented for all scanners which result in themselves (*i.e.* `ScanFromStr::Output = Self`).

This exists to aid type inference.

See: [`ScanFromStr`](trait.ScanFromStr.html).
*/
pub trait ScanSelfFromStr<'a>: ScanFromStr<'a, Output=Self> {
    /**
    Perform a scan on the given input.

    See: [`ScanFromStr::scan_from`](trait.ScanFromStr.html#tymethod.scan_from).
    */
    fn scan_self_from(s: &'a str) -> Result<(Self, usize), ScanErrorKind> {
        Self::scan_from(s)
    }
}

impl<'a, T> ScanSelfFromStr<'a> for T where T: ScanFromStr<'a, Output=T> {}

/**
This trait defines scanning a type from a binary representation.

This should be implemented to match implementations of `std::fmt::Binary`.
*/
pub trait ScanFromBinary<'a>: Sized {
    /**
    Perform a scan on the given input.

    See: [`ScanFromStr::scan_from`](trait.ScanFromStr.html#tymethod.scan_from).
    */
    fn scan_from_binary(s: &'a str) -> Result<(Self, usize), ScanErrorKind>;
}

/**
This trait defines scanning a type from an octal representation.

This should be implemented to match implementations of `std::fmt::Octal`.
*/
pub trait ScanFromOctal<'a>: Sized {
    /**
    Perform a scan on the given input.

    See: [`ScanFromStr::scan_from`](trait.ScanFromStr.html#tymethod.scan_from).
    */
    fn scan_from_octal(s: &'a str) -> Result<(Self, usize), ScanErrorKind>;
}

/**
This trait defines scanning a type from a hexadecimal representation.

This should be implemented to match implementations of `std::fmt::LowerHex` and `std::fmt::UpperHex`.
*/
pub trait ScanFromHex<'a>: Sized {
    /**
    Perform a scan on the given input.

    See: [`ScanFromStr::scan_from`](trait.ScanFromStr.html#tymethod.scan_from).
    */
    fn scan_from_hex(s: &'a str) -> Result<(Self, usize), ScanErrorKind>;
}

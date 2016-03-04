/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This module defines various scanners that can be used to extract values from input text.

## Kinds of Scanner

Scanners can be classified as "static self scanners", "static abstract scanners", and "runtime abstract scanners".

* "Static self scanners" are types which implement the `ScanFromStr` trait and output an instance of themselves.  For example, if you scan using the `i32` type, you get an `i32` result.  These are implemented for types which have an obvious "default" scanning syntax.

  As a consequence of outputting an instance of themselves, they *also* automatically implement the `ScanSelfFromStr` trait.

* "Static abstract scanners" are types which implement the `ScanFromStr` trait and output an instance of *some other* type.  For example, if you scan using the `Word` type, you get a `&str` or `String` result.  These are implemented for cases where different rules are desireable, such as scanning particular *subsets* of a type (see `Word`, `Number`, `NonSpace`), or non-default encodings (see `Binary`, `Octal`, `Hex`).

* "Runtime abstract scanners" implement the `ScanStr` trait and serve the same overall function as static abstract scanners, except that the scanner *itself* must be constructed.  In other words, static scanners are types, runtime scanners are *values*.  This makes them a little less straightforward to use, but they are *significantly* more flexible.  They can be parameterised at runtime, to perform arbitrary manipulations of both the text input and scanned values (see `max_width`, `re_str`).

## Bridging Between Static and Runtime Scanners

A scanner of interest is `ScanA<Type>`.  This is a runtime scanner which takes a *static* scanner as a type parameter.  This allows you to use a static scanner in a context where a runtime scanner is needed.

For example, these two bindings are equivalent in terms of behaviour:

```ignore
    // Scan a u32.
    let _: u32
    let _ <| scan_a::<u32>()
```

## Creating Runtime Scanners

Runtime scanners are typically constructed using functions, rather than dealing with the implementing type itself.  For example, to get an instance of the `ExactWidth` runtime scanner, you would call either the `exact_width` or `exact_width_a` functions.

The reason for two functions is that most runtime scanners accept a *second* runtime scanner for the purposes of chaining.  This allows several transformations to be applied outside-in.  For example, you can combine runtime scanners together like so:

```ignore
    // Scan a word of between 2 and 5 bytes.
    let _ <| min_width(2, max_width(5, scan_a::<Word>()))
```

Functions ending in `_a` are a shorthand for the common case of wrapping a runtime scanner around a static scanner.  For example, the following two patterns are equivalent:

```ignore
    // Scan a u32 that has, at most, four digits.
    let _ <| max_width(4, scan_a::<u32>())
    let _ <| max_width_a::<u32>(4)
```
*/
/*
It is also where implementations for existing standard and external types are kept, though these do not appear in the documentation.
*/
pub use self::misc::{
    Everything, HorSpace, Newline, NonSpace, Space,
    Ident, Line, Number, Word, Wordish,
    Inferred, KeyValuePair, QuotedString,
    Binary, Octal, Hex,
};

#[doc(inline)] pub use self::runtime::{
    exact_width, exact_width_a,
    max_width, max_width_a,
    min_width, min_width_a,
    scan_a,
};

#[cfg(feature="regex")]
#[doc(inline)]
pub use self::runtime::{re, re_a, re_str};

#[cfg(feature="nightly-pattern")]
#[doc(inline)]
pub use self::runtime::{until_pat, until_pat_a, until_pat_str};

#[macro_use] mod macros;

pub mod runtime;

mod lang;
mod misc;
mod std;

use ::ScanError;
use ::input::ScanInput;

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
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError>;

    /**
    Indicates whether or not the scanner wants its input to have leading "junk", such as whitespace, stripped.

    The default implementation returns `true`, which is almost *always* the correct answer.  You should only implement this explicitly (and return `false`) if you are implementing a scanner for which leading whitespace is important.
    */
    fn wants_leading_junk_stripped() -> bool { true }
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
    fn scan_self_from<I: ScanInput<'a>>(s: I) -> Result<(Self, usize), ScanError> {
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
    fn scan_from_binary<I: ScanInput<'a>>(s: I) -> Result<(Self, usize), ScanError>;
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
    fn scan_from_octal<I: ScanInput<'a>>(s: I) -> Result<(Self, usize), ScanError>;
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
    fn scan_from_hex<I: ScanInput<'a>>(s: I) -> Result<(Self, usize), ScanError>;
}

/**
This trait defines the interface for runtime scanners.

Runtime scanners must be created before they can be used, but this allows their behaviour to be modified at runtime.
*/
pub trait ScanStr<'a>: Sized {
    /**
    The type that the implementation scans into.
    */
    type Output;

    /**
    Perform a scan on the given input.

    See: [`ScanFromStr::scan_from`](trait.ScanFromStr.html#tymethod.scan_from).
    */
    fn scan<I: ScanInput<'a>>(&mut self, s: I) -> Result<(Self::Output, usize), ScanError>;

    /**
    Indicates whether or not the scanner wants its input to have leading "junk", such as whitespace, stripped.

    There is no default implementation of this for runtime scanners, because almost all runtime scanners forward on to some *other* scanner, and it is *that* scanner that should typically decide what to do.

    Thus, in most cases, your implementation of this method should simply defer to the *next* scanner.

    See: [`ScanFromStr::wants_leading_junk_stripped`](trait.ScanFromStr.html#tymethod.wants_leading_junk_stripped).
    */
    fn wants_leading_junk_stripped(&self) -> bool;
}

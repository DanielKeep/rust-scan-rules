/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Defines error types used by the crate.
*/
use std::error::Error;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

/**
Represents an error that occurred during scanning.

Depending on what happened, it could represent an actual scanning failure, a problem with the pattern, an underlying IO failure, or something else entirely.
*/
#[derive(Debug)]
pub struct ScanError {
    /**
    The rough cursor position at which this error occurred.  This will typically be the position the input cursor was at when it began trying to scan a particular literal or value.
    */
    pub at: ScanErrorAt,

    /**
    The kind of error that occurred.
    */
    pub kind: ScanErrorKind,

    /**
    Dummy private field to prevent exhaustive deconstruction.
    */
    _priv: (),
}

impl ScanError {
    /**
    Construct a new `ScanError`.
    */
    pub fn new(at: usize, kind: ScanErrorKind) -> Self {
        ScanError {
            at: ScanErrorAt { bytes: at },
            kind: kind,
            _priv: (),
        }
    }

    /**
    Shorthand for constructing an `ExpectedEnd` error.
    */
    pub fn expected_end(at: usize) -> Self {
        Self::new(at, ScanErrorKind::ExpectedEnd)
    }

    /**
    Shorthand for constructing an `Io` error.
    */
    pub fn io(err: io::Error) -> Self {
        Self::new(0, ScanErrorKind::Io(err))
    }

    /**
    Shorthand for constructing a `LiteralMismatch` error.
    */
    pub fn literal_mismatch(at: usize) -> Self {
        Self::new(at, ScanErrorKind::LiteralMismatch)
    }

    /**
    Shorthand for constructing a `Syntax` error.
    */
    pub fn syntax(at: usize, desc: &'static str) -> Self {
        Self::new(at, ScanErrorKind::Syntax(desc))
    }

    /**
    Shorthand for constructing an `Other` error.
    */
    pub fn other<E: Into<Box<Error>>>(at: usize, err: E) -> Self {
        Self::new(at, ScanErrorKind::from_other(err))
    }

    /**
    Compare two `ScanError`s, and return the one which occurred the furthest into the input cursor.
    */
    pub fn furthest_along(self, other: Self) -> Self {
        if self.at.offset() >= other.at.offset() {
            self
        } else {
            other
        }
    }
}

impl<'a> fmt::Display for ScanError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!("scan error: ".fmt(fmt));
        try!(self.kind.fmt(fmt));
        try!(", at offset: ".fmt(fmt));
        try!(self.at.offset().fmt(fmt));
        Ok(())
    }
}

impl Error for ScanError {
    fn cause(&self) -> Option<&Error> {
        self.kind.cause()
    }

    fn description(&self) -> &str {
        self.kind.description()
    }
}

/**
Represents the position at which an error occurred.
*/
/*
This exists because I'm still considering including the input which generated the error, for the sake of nice error messages.

I'm not using `Cursor`, because I don't want errors tied to a specific input wrapper.
*/
#[derive(Debug)]
pub struct ScanErrorAt {
    /// Offset in bytes.
    bytes: usize,
}

impl ScanErrorAt {
    /**
    Return the offset from the start of input that an error occurred at, in bytes.
    */
    pub fn offset(&self) -> usize {
        self.bytes
    }
}

/**
Indicates the kind of error that occurred during scanning.
*/
#[derive(Debug)]
pub enum ScanErrorKind {
    /// Failed to match a literal pattern term.
    LiteralMismatch,

    /// General syntax error.
    Syntax(&'static str),

    /**
    General syntax error.

    Due to [Rust issue #26448](https://github.com/rust-lang/rust/issues/26448), some scanners which want to return a `Syntax` error *cannot*.
    */
    SyntaxNoMessage,

    /// Expected end-of-input.
    ExpectedEnd,

    /// Floating point parsing failed.
    Float(ParseFloatError),

    /// Integer parsing failed.
    Int(ParseIntError),

    /// An IO error occurred.
    Io(io::Error),

    /// Some other error occurred.
    Other(Box<Error>),

    /// Hidden variant to prevent exhaustive matching.
    #[doc(hidden)]
    __DoNotMatch,
}

impl ScanErrorKind {
    /**
    Construct an `Other` error from some generic error value.
    */
    pub fn from_other<E: Into<Box<Error>>>(err: E) -> Self {
        ScanErrorKind::Other(err.into())
    }
}

impl fmt::Display for ScanErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch => "did not match literal".fmt(fmt),
            Syntax(desc) => {
                try!("syntax error: ".fmt(fmt));
                try!(desc.fmt(fmt));
                Ok(())
            },
            SyntaxNoMessage => "unknown syntax error".fmt(fmt),
            ExpectedEnd => "expected end of input".fmt(fmt),
            Float(ref err) => err.fmt(fmt),
            Int(ref err) => err.fmt(fmt),
            Io(ref err) => err.fmt(fmt),
            Other(ref err) => err.fmt(fmt),
            __DoNotMatch => panic!("do not use ScanErrorKind::__DoNotMatch!"),
        }
    }
}

impl Error for ScanErrorKind {
    fn cause(&self) -> Option<&Error> {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch 
            | Syntax(_)
            | SyntaxNoMessage
            | ExpectedEnd
            => None,
            Float(ref err) => err.cause(),
            Int(ref err) => err.cause(),
            Io(ref err) => err.cause(),
            Other(ref err) => err.cause(),
            __DoNotMatch => panic!("do not use ScanErrorKind::__DoNotMatch!"),
        }
    }

    fn description(&self) -> &str {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch => "did not match literal",
            Syntax(_) => "syntax error",
            SyntaxNoMessage => "unknown syntax error",
            ExpectedEnd => "expected end of input",
            Float(ref err) => err.description(),
            Int(ref err) => err.description(),
            Io(ref err) => err.description(),
            Other(ref err) => err.description(),
            __DoNotMatch => panic!("do not use ScanErrorKind::__DoNotMatch!"),
        }
    }
}

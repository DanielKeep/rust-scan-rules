/*!
Defines error types used by the crate.
*/
use std::error::Error;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use input::Cursor;

/**
Represents an error that occurred during scanning.

Depending on what happened, it could represent an actual scanning failure, a problem with the pattern, an underlying IO failure, or something else entirely.
*/
#[derive(Debug)]
pub struct ScanError<'a> {
    /**
    The rough cursor position at which this error occurred.  This will typically be the position the input cursor was at when it began trying to scan a particular literal or value.
    */
    pub at: Cursor<'a>,

    /**
    The kind of error that occurred.
    */
    pub kind: ScanErrorKind,

    /**
    Dummy private field to prevent exhaustive deconstruction.
    */
    _priv: (),
}

impl<'a> ScanError<'a> {
    /**
    Construct a new `ScanError`.
    */
    pub fn new(at: Cursor<'a>, kind: ScanErrorKind) -> Self {
        ScanError {
            at: at,
            kind: kind,
            _priv: (),
        }
    }

    /**
    Shorthand for constructing an `ExpectedEnd` error.
    */
    pub fn expected_end(at: Cursor<'a>) -> Self {
        Self::new(at, ScanErrorKind::ExpectedEnd)
    }

    /**
    Shorthand for constructing an `Io` error.
    */
    pub fn io(err: io::Error) -> Self {
        Self::new(Cursor::new_with_offset("", 0), ScanErrorKind::Io(err))
    }

    /**
    Shorthand for constructing a `LiteralMismatch` error.
    */
    pub fn literal_mismatch(at: Cursor<'a>) -> Self {
        Self::new(at, ScanErrorKind::LiteralMismatch)
    }

    /**
    Shorthand for constructing a `Syntax` error.
    */
    pub fn syntax(at: Cursor<'a>, desc: &'static str) -> Self {
        Self::new(at, ScanErrorKind::Syntax(desc))
    }

    /**
    Shorthand for constructing an `Other` error.
    */
    pub fn other<E: Into<Box<Error>>>(at: Cursor<'a>, err: E) -> Self {
        Self::new(at, ScanErrorKind::from_other(err))
    }

    /**
    Compare two `ScanError`s, and return the one which occurred the furthest into the input cursor.
    */
    pub fn furthest_along(self, other: Self) -> Self {
        if self.at.as_bytes().as_ptr() >= other.at.as_bytes().as_ptr() {
            self
        } else {
            other
        }
    }

    /**
    Replace the borrowed components of the `ScanError` with `'static` dummy values, allowing the error to escape beyond the lifetime of the original input data.
    */
    pub fn into_static(self) -> ScanError<'static> {
        ScanError {
            at: Cursor::new_with_offset("", self.at.offset()),
            kind: self.kind,
            _priv: (),
        }
    }
}

impl<'a> fmt::Display for ScanError<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!("scan error: ".fmt(fmt));
        try!(self.kind.fmt(fmt));
        try!(", at offset: ".fmt(fmt));
        fmt::Debug::fmt(&self.at.offset(), fmt)
    }
}

impl<'a> Error for ScanError<'a> {
    fn cause(&self) -> Option<&Error> {
        self.kind.cause()
    }

    fn description(&self) -> &str {
        self.kind.description()
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

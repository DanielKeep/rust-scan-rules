use std::error::Error;
use std::fmt;
use std::io;
use ::Cursor;

#[derive(Debug)]
pub struct ScanError<'a> {
    pub at: Cursor<'a>,
    pub kind: ScanErrorKind,
    _priv: (),
}

impl<'a> ScanError<'a> {
    pub fn new(at: Cursor<'a>, kind: ScanErrorKind) -> Self {
        ScanError {
            at: at,
            kind: kind,
            _priv: (),
        }
    }

    pub fn expected_end(at: Cursor<'a>) -> Self {
        Self::new(at, ScanErrorKind::ExpectedEnd)
    }

    pub fn io(err: io::Error) -> Self {
        Self::new(Cursor::new_with_offset("", 0), ScanErrorKind::Io(err))
    }

    pub fn literal_mismatch(at: Cursor<'a>) -> Self {
        Self::new(at, ScanErrorKind::LiteralMismatch)
    }

    pub fn missing(at: Cursor<'a>) -> Self {
        Self::new(at, ScanErrorKind::Missing)
    }

    pub fn other<E: Into<Box<Error>>>(at: Cursor<'a>, err: E) -> Self {
        Self::new(at, ScanErrorKind::from_other(err))
    }

    pub fn furthest_along(self, other: Self) -> Self {
        if self.at.as_bytes().as_ptr() >= other.at.as_bytes().as_ptr() {
            self
        } else {
            other
        }
    }

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

#[derive(Debug)]
pub enum ScanErrorKind {
    LiteralMismatch,
    Missing,
    ExpectedEnd,
    Io(io::Error),
    Other(Box<Error>),
}

impl ScanErrorKind {
    pub fn from_other<E: Into<Box<Error>>>(err: E) -> Self {
        ScanErrorKind::Other(err.into())
    }
}

impl fmt::Display for ScanErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch => "did not match literal".fmt(fmt),
            Missing => "missing scannable input".fmt(fmt),
            ExpectedEnd => "expected end of input".fmt(fmt),
            Io(ref err) => err.fmt(fmt),
            Other(ref err) => err.fmt(fmt),
        }
    }
}

impl Error for ScanErrorKind {
    fn cause(&self) -> Option<&Error> {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch => None,
            Missing => None,
            ExpectedEnd => None,
            Io(ref err) => err.cause(),
            Other(ref err) => err.cause(),
        }
    }

    fn description(&self) -> &str {
        use self::ScanErrorKind::*;
        match *self {
            LiteralMismatch => "did not match literal",
            Missing => "missing scannable input",
            ExpectedEnd => "expected end of input",
            Io(ref err) => err.description(),
            Other(ref err) => err.description(),
        }
    }
}

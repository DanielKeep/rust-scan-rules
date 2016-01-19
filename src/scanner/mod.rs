pub use self::misc::Word;

#[macro_use] mod macros;

pub mod util;

mod lang;
mod misc;
mod std;

use ::ScanErrorKind;

pub trait ScanFromStr<'a>: Sized {
    type Output;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind>;
}

pub trait ScanSelfFromStr<'a>: ScanFromStr<'a, Output=Self> {
    fn scan_self_from(s: &'a str) -> Result<(Self, usize), ScanErrorKind> {
        Self::scan_from(s)
    }
}

impl<'a, T> ScanSelfFromStr<'a> for T where T: ScanFromStr<'a, Output=T> {}

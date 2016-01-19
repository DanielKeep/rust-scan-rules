use regex::Regex;
use ::ScanErrorKind;

lazy_static! {
    static ref INTEGER_RE: Regex = Regex::new(r"^\d+").unwrap();
    static ref WORD_RE: Regex = Regex::new(r"^(\w+|\S)").unwrap();
}

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

impl<'a> ScanFromStr<'a> for &'a str {
    type Output = &'a str;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        Word::scan_from(s)
    }
}

impl<'a> ScanFromStr<'a> for i32 {
    type Output = Self;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        let (w, end) = try!(INTEGER_RE.find(s)
            .map(|(_, b)| (&s[..b], b))
            .ok_or(ScanErrorKind::Missing));
        w.parse()
            .map(|v| (v, end))
            .map_err(ScanErrorKind::from_other)
    }
}

impl<'a> ScanFromStr<'a> for String {
    type Output = String;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        <&str>::scan_from(s).map(|(v, n)| (v.to_owned(), n))
    }
}

pub enum Word {}

impl<'a> ScanFromStr<'a> for Word {
    type Output = &'a str;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        WORD_RE.find(s)
            .map(|(_, b)| (&s[..b], b))
            .ok_or(ScanErrorKind::Missing)
    }
}

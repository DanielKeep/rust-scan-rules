use regex::Regex;
use ::ScanErrorKind;
use super::ScanFromStr;
use super::misc::Word;

lazy_static! {
    static ref INTEGER_RE: Regex = Regex::new(r"^\d+").unwrap();
}

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

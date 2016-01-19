use regex::Regex;
use ::ScanErrorKind;
use super::ScanFromStr;

lazy_static! {
    static ref WORD_RE: Regex = Regex::new(r"^(\w+|\S)").unwrap();
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

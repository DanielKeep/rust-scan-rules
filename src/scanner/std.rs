use ::ScanErrorKind;
use super::ScanFromStr;

impl<'a> ScanFromStr<'a> for String {
    type Output = String;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        <&str>::scan_from(s).map(|(v, n)| (v.to_owned(), n))
    }
}

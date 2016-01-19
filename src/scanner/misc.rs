use ::ScanErrorKind;
use super::ScanFromStr;
use super::util::StrUtil;

pub enum Word {}

impl<'a> ScanFromStr<'a> for Word {
    type Output = &'a str;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        match s.split_word() {
            Some((word, tail)) => Ok((word, s.subslice_offset(tail).unwrap())),
            None => Err(ScanErrorKind::Missing),
        }
    }
}

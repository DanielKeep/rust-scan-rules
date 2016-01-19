use std::marker::PhantomData;
use ::ScanErrorKind;
use super::{ScanFromStr, ScanSelfFromStr};
use super::util::StrUtil;

pub struct KeyValuePair<K, V>(PhantomData<(K, V)>);

impl<'a, K, V> ScanFromStr<'a> for KeyValuePair<K, V>
where K: ScanSelfFromStr<'a>, V: ScanSelfFromStr<'a> {
    type Output = (K, V);
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        scan!(s;
            (let k: K, ":", let v: V, ..tail) => ((k, v), s.subslice_offset(tail).unwrap())
        ).map_err(|e| e.kind)
    }
}

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

/*!
Miscellaneous, abstract scanners.
*/
use std::marker::PhantomData;
use ::ScanErrorKind;
use super::{ScanFromStr, ScanSelfFromStr};
use super::util::StrUtil;

/**
An abstract scanner that scans a `(K, V)` value using the syntax `K: V`.

This scanner is designed to take advantage of three things:

1. Maps (*i.e.* associative containers) typically print themselves with the syntax `{key_0: value_0, key_1: value_1, ...}`.

2. Maps typically implement `Extend<(K, V)>`; that is, you can add new items by extending the map with a `(K, V)` tuple.

3. Repeating bindings can be scanned into any container that implements `Default` and `Extend`.

As such, this scanner allows one to parse a `Map` type like so:

```ignore
scan!(input; "{", [let kvs: KeyValuePair<K, V>],*: Map<_, _>, "}" => kvs)
```
*/
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

/**
Scans a single word into a string.

TODO: be more specific.
*/
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

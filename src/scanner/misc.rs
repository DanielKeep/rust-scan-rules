/*!
Miscellaneous, abstract scanners.
*/
use std::marker::PhantomData;
use strcursor::StrCursor;
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
Scans a quoted string.
*/
pub enum QuotedString {}

impl<'a> ScanFromStr<'a> for QuotedString {
    type Output = String;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        // TODO: Stop being lazy.
        use ::ScanErrorKind::Missing;

        let cur = StrCursor::new_at_start(s);
        let (cp, cur) = try!(cur.next_cp().ok_or(Missing));
        match cp {
            '"' => (),
            _ => return Err(Missing)
        }

        let mut s = String::new();
        let mut cur = cur;
        loop {
            match cur.next_cp() {
                None => return Err(Missing),
                Some(('\\', after)) => {
                    match after.slice_after().split_escape_default() {
                        Err(_) => return Err(Missing),
                        Ok((cp, tail)) => {
                            // TODO: replace this
                            unsafe { cur.unsafe_set_at(tail); }
                            s.push(cp);
                        },
                    }
                },
                Some(('"', after)) => {
                    cur = after;
                    break;
                },
                Some((cp, after)) => {
                    cur = after;
                    s.push(cp);
                },
            }
        }

        Ok((s, cur.byte_pos()))
    }
}

#[cfg(test)]
#[test]
fn test_quoted_string() {
    use ::ScanErrorKind as SEK;
    use self::QuotedString as QS;

    assert_match!(QS::scan_from(""), Err(SEK::Missing));
    assert_match!(QS::scan_from("dummy xyz"), Err(SEK::Missing));
    assert_match!(QS::scan_from("'dummy' xyz"), Err(SEK::Missing));
    assert_match!(QS::scan_from("\"dummy\" xyz"),
        Ok((ref s, 7)) if s == "dummy");
    assert_match!(QS::scan_from("\"ab\\\"cd\" xyz"),
        Ok((ref s, 8)) if s == "ab\"cd");
    assert_match!(QS::scan_from("\"ab\\x41cd\" xyz"),
        Ok((ref s, 10)) if s == "abAcd");
    assert_match!(QS::scan_from("\"a\\'b\\u{5B57}c\\0d\" xyz"),
        Ok((ref s, 18)) if s == "a'b字c\0d");
}

/**
Scans a single word into a string.

TODO: be more specific.
*/
pub struct Word<'a, T=&'a str>(PhantomData<(&'a (), T)>);

impl<'a, T> ScanFromStr<'a> for Word<'a, T> where &'a str: Into<T> {
    type Output = T;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        match s.split_word() {
            Some((word, tail)) => Ok((word.into(), s.subslice_offset(tail).unwrap())),
            None => Err(ScanErrorKind::Missing),
        }
    }
}

#[cfg(test)]
macro_rules! assert_match {
    ($e:expr, $p:pat) => {
        match $e {
            $p => (),
            e => panic!("assertion failed: `(left match right)` (left: `{:?}`, right: `{:?}`)",
                e, stringify!($p))
        }
    };

    ($e:expr, $p:pat if $cond:expr) => {
        match $e {
            $p if $cond => (),
            e => panic!("assertion failed: `(left match right)` (left: `{:?}`, right: `{:?}`)",
                e, stringify!($p if $cond))
        }
    };
}

macro_rules! parse_scanner {
    (@as_item $i:item) => {$i};

    (impl<$lt:tt> for $ty:ty, from $scanner:ty) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from(s: &$lt str) -> ::std::result::Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    use ::std::result::Result::{Ok, Err};
                    use ::std::str::FromStr;
                    match <$scanner as $crate::scanner::ScanFromStr>::scan_from(s) {
                        Err(err) => Err(err),
                        Ok((v, n)) => match <Self as FromStr>::from_str(v) {
                            Err(_) => Err($crate::ScanErrorKind::Missing),
                            Ok(v) => Ok((v, n)),
                        },
                    }
                }
            }
        }
    };

    (impl<$lt:tt> for $ty:ty, regex $regex:expr) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from(s: &$lt str) -> Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use ::std::str::FromStr;
                    use ::regex::Regex;
                    use $crate::ScanErrorKind;

                    let (w, end) = try!(
                        Option::ok_or(
                            Option::map(
                                Regex::find(&$regex, s),
                                |(a, b)| (&s[a..b], b)
                            ),
                            ScanErrorKind::Missing
                        )
                    );

                    Result::map_err(
                        Result::map(
                            <Self as FromStr>::from_str(w),
                            |v| (v, end)
                        ),
                        ScanErrorKind::from_other
                    )
                }
            }
        }
    };
}
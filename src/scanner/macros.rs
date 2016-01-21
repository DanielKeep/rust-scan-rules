/*!
Macros specific to the scanner implementations.
*/

/**
Like `assert_eq!`, except the RHS is a pattern (optionally with guard).
*/
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

/**
Define a scanner implementation based on a few common cases:

* `impl<'a> for Ty, from OtherTy`: run the scanner for `OtherTy`, passing the result through `FromStr`.

* `impl<'a> for Ty, regex r"..."`: use the provided regex to extract part of the input, passing the resulting slice through `FromStr`.
*/
macro_rules! parse_scanner {
    (@as_item $i:item) => {$i};

    (impl<$lt:tt> for $ty:ty, from $scanner:ty, err wrap $kind:ident) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from(s: &$lt str) -> ::std::result::Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    use ::std::result::Result::{Ok, Err};
                    use ::std::str::FromStr;
                    match <$scanner as $crate::scanner::ScanFromStr>::scan_from(s) {
                        Err(_) => Err($crate::ScanErrorKind::Syntax($msg)),
                        Ok((v, n)) => match <Self as FromStr>::from_str(v) {
                            Err(err) => Err($crate::ScanErrorKind::$kind(err)),
                            Ok(v) => Ok((v, n)),
                        },
                    }
                }
            }
        }
    };

    (impl<$lt:tt> for $ty:ty, from $scanner:ty, err desc $msg:expr) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from(s: &$lt str) -> ::std::result::Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    use ::std::result::Result::{Ok, Err};
                    use ::std::str::FromStr;
                    match <$scanner as $crate::scanner::ScanFromStr>::scan_from(s) {
                        Err(_) => Err($crate::ScanErrorKind::Syntax($msg)),
                        Ok((v, n)) => match <Self as FromStr>::from_str(v) {
                            Err(_) => Err($crate::ScanErrorKind::Syntax($msg)),
                            Ok(v) => Ok((v, n)),
                        },
                    }
                }
            }
        }
    };

    (
        impl<$lt:tt> for $ty:ty,
        regex $regex:expr,
        regex err $re_err:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            impl<$lt> for $ty,
                regex $regex,
                regex err $re_err,
                map |m| <$ty as ::std::str::FromStr>::from_str(m),
                err map $err
        }
    };

    (
        impl<$lt:tt> for $ty:ty,
        regex $regex:expr,
        regex err $re_err:expr,
        map |$s:ident| $map:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from(s: &$lt str) -> Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use ::regex::Regex;
                    use $crate::ScanErrorKind;

                    let ($s, end) = try!(
                        Option::ok_or(
                            Option::map(
                                Regex::find(&$regex, s),
                                |(a, b)| (&s[a..b], b)
                            ),
                            ScanErrorKind::Syntax($re_err)
                        )
                    );

                    Result::map_err(
                        Result::map(
                            $map,
                            |v| (v, end)
                        ),
                        $err
                    )
                }
            }
        }
    };

    (
        impl<$lt:tt> $tr_name:ident::$tr_meth:ident for $ty:ty,
        regex $regex:expr,
        regex err $re_err:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            impl<$lt> $tr_name::$tr_meth for $ty,
                regex $regex,
                regex err $re_err,
                map |m| <$ty as ::std::str::FromStr>::from_str(m),
                err map $err
        }
    };

    (
        impl<$lt:tt> $tr_name:ident::$tr_meth:ident for $ty:ty,
        regex $regex:expr,
        regex err $re_err:expr,
        map $map:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::$tr_name<$lt> for $ty {
                fn $tr_meth(s: &$lt str) -> Result<(Self, usize), $crate::ScanErrorKind> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use ::regex::Regex;
                    use $crate::ScanErrorKind;

                    let (w, end) = try!(
                        Option::ok_or(
                            Option::map(
                                Regex::find(&$regex, s),
                                |(a, b)| (&s[a..b], b)
                            ),
                            ScanErrorKind::Syntax($re_err)
                        )
                    );

                    Result::map_err(
                        Result::map(
                            ($map)(w),
                            |v| (v, end)
                        ),
                        $err
                    )
                }
            }
        }
    };
}

/**
Define a scanner implementation using `scan!` rules.

The result of the rules should be `(Output, &str)` where the `&str` is the unconsumed tail.
*/
macro_rules! scanner {
    (@as_item $i:item) => {$i};

    (
        impl<$lt:tt $(, $ty_params:ident)*> ScanFromStr for $ty:ty { $($patterns:tt)* }
    ) => {
        scanner! { impl<$lt $(, $ty_params)*> ScanFromStr for $ty where {} { $($patterns)* } }
    };

    (
        impl<$lt:tt $(, $ty_params:ident)*> ScanFromStr for $ty:ty where {$($clauses:tt)*} { $($patterns:tt)* }
    ) => {
        scanner! {
            @as_item
            impl<$lt $(, $ty_params)*> $crate::scanner::ScanFromStr<$lt> for $ty
            where
                $($ty_params: $crate::scanner::ScanFromStr<$lt, Output=$ty_params>,)*
                $($clauses)*
            {
                type Output = Self;

                fn scan_from(s: &$lt str) -> Result<(Self::Output, usize), $crate::ScanErrorKind> {
                    match scan! { s; $($patterns)* } {
                        Ok((v, tail)) => {
                            let off = ::std::option::Option::expect($crate::subslice_offset(s, tail), "scanner returned tail that wasn't part of the original input");
                            Ok((v, off))
                        },
                        Err(err) => Err(err.kind),
                    }
                }
            }
        }
    };
}

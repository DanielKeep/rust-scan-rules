/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
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
Determines whether an expression matches a pattern.
*/
macro_rules! matches {
    (@as_expr $e:expr) => { $e };

    ($e:expr, $($pat:tt)+) => {
        matches!(
            @as_expr
            match $e {
                $($pat)* => true,
                _ => false,
            }
        )
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
                fn scan_from<I: $crate::input::ScanInput<$lt>>(s: I) -> ::std::result::Result<(Self::Output, usize), $crate::ScanError> {
                    use ::std::result::Result::{Ok, Err};
                    use ::std::str::FromStr;
                    let s = s.as_str();
                    match <$scanner as $crate::scanner::ScanFromStr>::scan_from(s) {
                        Err(_) => Err($crate::ScanError::syntax($msg)),
                        Ok((v, n)) => match <Self as FromStr>::from_str(v) {
                            Err(err) => Err($crate::ScanError::new(0, $crate::ScanErrorKind::$kind(err))),
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
                fn scan_from<I: $crate::input::ScanInput<$lt>>(s: I) -> ::std::result::Result<(Self::Output, usize), $crate::ScanError> {
                    use ::std::result::Result::{Ok, Err};
                    use ::std::str::FromStr;
                    match <$scanner as $crate::scanner::ScanFromStr>::scan_from(s) {
                        Err(_) => Err($crate::ScanError::syntax($msg)),
                        Ok((v, n)) => match <Self as FromStr>::from_str(v) {
                            Err(_) => Err($crate::ScanError::new(0, $crate::ScanErrorKind::Syntax($msg))),
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
                fn scan_from<I: $crate::input::ScanInput<$lt>>(s: I) -> Result<(Self::Output, usize), $crate::ScanError> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use ::regex::Regex;
                    use $crate::ScanError;

                    let s = s.as_str();
                    let ($s, end) = try!(
                        Option::ok_or(
                            Option::map(
                                Regex::find(&$regex, s),
                                |(a, b)| (&s[a..b], b)
                            ),
                            ScanError::syntax($re_err)
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
        impl<$lt:tt> for $ty:ty,
        matcher $matcher:expr,
        matcher err $ma_err:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            impl<$lt> for $ty,
                matcher $matcher,
                matcher err $ma_err,
                map |m| <$ty as ::std::str::FromStr>::from_str(m),
                err map $err
        }
    };

    (
        impl<$lt:tt> for $ty:ty,
        matcher $matcher:expr,
        matcher err $ma_err:expr,
        map |$s:ident| $map:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::ScanFromStr<$lt> for $ty {
                type Output = Self;
                fn scan_from<I: $crate::input::ScanInput<$lt>>(s: I) -> Result<(Self::Output, usize), $crate::ScanError> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use $crate::ScanError;

                    let s = s.as_str();
                    let ($s, end) = try!(
                        Option::ok_or(
                            Option::map(
                                $matcher(s),
                                |((a, b), c)| (&s[a..b], c)
                            ),
                            ScanError::syntax($ma_err)
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
                fn $tr_meth<I: $crate::input::ScanInput<$lt>>(s: I) -> Result<(Self, usize), $crate::ScanError> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use ::regex::Regex;
                    use $crate::ScanError;

                    let s_str = s.as_str();
                    let (w, end) = try!(
                        Option::ok_or(
                            Option::map(
                                Regex::find(&$regex, s_str),
                                |(a, b)| (&s_str[a..b], b)
                            ),
                            ScanError::syntax($re_err)
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

    (
        impl<$lt:tt> $tr_name:ident::$tr_meth:ident for $ty:ty,
        matcher $matcher:expr,
        matcher err $ma_err:expr,
        map $map:expr,
        err map $err:expr
    ) => {
        parse_scanner! {
            @as_item
            impl<$lt> $crate::scanner::$tr_name<$lt> for $ty {
                fn $tr_meth<I: $crate::input::ScanInput<$lt>>(s: I) -> Result<(Self, usize), $crate::ScanError> {
                    use ::std::option::Option;
                    use ::std::result::Result;
                    use $crate::ScanError;

                    let s_str = s.as_str();
                    let (w, end) = try!(
                        Option::ok_or(
                            Option::map(
                                $matcher(s_str),
                                |((a, b), c)| (&s_str[a..b], c)
                            ),
                            ScanError::syntax($ma_err)
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
        impl<$lt:tt $(, $ty_params:ident)*> ScanFromStr for $ty:ty => $dest:ident { $($patterns:tt)* }
    ) => {
        scanner! { impl<$lt $(, $ty_params)*> ScanFromStr for $ty => $dest, where {} { $($patterns)* } }
    };

    (
        impl<$lt:tt $(, $ty_params:ident)*> ScanFromStr for $ty:ty => $dest:ident, where {$($clauses:tt)*} { $($patterns:tt)* }
    ) => {
        scanner! {
            @as_item
            impl<$lt $(, $ty_params)*> $crate::scanner::ScanFromStr<$lt> for $ty
            where
                $($ty_params: $crate::scanner::ScanFromStr<$lt, Output=$ty_params>,)*
                $($clauses)*
            {
                type Output = $dest<$(<$ty_params as $crate::scanner::ScanFromStr<$lt>>::Output,)*>;

                fn scan_from<I: $crate::input::ScanInput<$lt>>(s: I) -> Result<(Self::Output, usize), $crate::ScanError> {
                    match scan! { s.to_cursor(); $($patterns)* } {
                        Ok((v, tail)) => {
                            let off = ::std::option::Option::expect($crate::internal::subslice_offset(s.as_str(), tail), "scanner returned tail that wasn't part of the original input");
                            Ok((v, off))
                        },
                        Err(err) => Err(err),
                    }
                }
            }
        }
    };
}

/**
Returns the contents of an `Option`, or returns `None` from the current function.
*/
macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None,
        }
    };
}

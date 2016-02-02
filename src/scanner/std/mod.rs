/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Scanner implementations for standard library (and other "official" crates) types.
*/
mod collections;
mod net;

use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use ::ScanError;
use ::input::ScanInput;
use ::scanner::ScanFromStr;
use ::scanner::util::StrUtil;

macro_rules! impl_tuple {
    () => {};

    ($head:ident $($tail:ident)*) => {
        impl<'a, $head $(, $tail)*> ::scanner::ScanFromStr<'a> for ($head, $($tail,)*)
        where $head: ::scanner::ScanFromStr<'a>, $($tail: ::scanner::ScanFromStr<'a>,)* {
            type Output = (<$head as ::scanner::ScanFromStr<'a>>::Output, $(<$tail as ::scanner::ScanFromStr<'a>>::Output,)*);
            fn scan_from<I: $crate::input::ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ::ScanError> {
                #![allow(non_snake_case)]
                use ::scanner::util::StrUtil;
                let s = s.as_str();
                scan!(s;
                    ("(", let $head: $head, $(",", let $tail: $tail,)* [","]?, ")", ..tail)
                    => (($head, $($tail,)*), s.subslice_offset_stable(tail).unwrap())
                )
            }
        }

        impl_tuple! { $($tail)* }
    };
}

#[cfg(not(feature="tuples-16"))]
mod impl_tuples {
    impl_tuple! { T0 T1 T2 T3 }
}

#[cfg(feature="tuples-16")]
mod impl_tuples {
    impl_tuple! { T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 }
}

impl<'a> ScanFromStr<'a> for () {
    type Output = Self;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        scan!(s; ("(", ")", ..tail) => ((), s.subslice_offset_stable(tail).unwrap()))
    }
}

macro_rules! impl_array {
    (@as_item $i:item) => { $i };
    (@replace.expr $_tt:tt $sub:expr) => { $sub };

    () => {};

    ($len:tt $e0:ident $($ns:tt $es:ident)*) => {
        impl_array! {
            @as_item
            impl<'a, T> ::scanner::ScanFromStr<'a> for [T; $len] where T: ::scanner::ScanFromStr<'a> {
                type Output = [T::Output; $len];
                fn scan_from<I: $crate::input::ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ::ScanError> {
                    use ::scanner::util::StrUtil;
                    let s = s.as_str();
                    scan!(s;
                        ("[", let $e0: T, $(",", let $es: T,)* [","]?, "]", ..tail)
                        => ([$e0, $($es,)*], s.subslice_offset_stable(tail).unwrap())
                    )
                }
            }
        }

        impl_array! { $($ns $es)* }
    };
}

#[cfg(not(feature="arrays-32"))]
mod impl_arrays {
    impl_array! {
        8 e8 7 e7 6 e6 5 e5 4 e4 3 e3 2 e2 1 e1
    }
}

#[cfg(feature="arrays-32")]
mod impl_arrays {
    impl_array! {
        32 e32 31 e31
        30 e30 29 e29 28 e28 27 e27 26 e26 25 e25 24 e24 23 e23 22 e22 21 e21
        20 e20 19 e19 18 e18 17 e17 16 e16 15 e15 14 e14 13 e13 12 e12 11 e11
        10 e10 9 e9 8 e8 7 e7 6 e6 5 e5 4 e4 3 e3 2 e2 1 e1
    }
}

impl<'a, T> ScanFromStr<'a> for [T; 0] {
    type Output = Self;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let s = s.as_str();
        scan!(s; ("[", "]", ..tail) => ([], s.subslice_offset_stable(tail).unwrap()))
    }
}

impl<'a, T> ScanFromStr<'a> for Option<T> where T: ScanFromStr<'a> {
    type Output = Option<T::Output>;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        scan!( s.to_cursor();
            ("Some", "(", let v: T, ")", ..tail) => (Some(v), tail),
            ("None", ..tail) => (None, tail),
        ).map(|(v, t)| (v, s.as_str().subslice_offset_stable(t).unwrap()))
    }
}

impl<'a, T, E> ScanFromStr<'a> for Result<T, E>
where T: ScanFromStr<'a>, E: ScanFromStr<'a> {
    type Output = Result<T::Output, E::Output>;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        scan!( s.to_cursor();
            ("Some", "(", let v: T, ")", ..tail) => (Ok(v), tail),
            ("Err", "(", let v: E, ")", ..tail) => (Err(v), tail),
        ).map(|(v, t)| (v, s.as_str().subslice_offset_stable(t).unwrap()))
    }
}

impl<'a> ScanFromStr<'a> for String {
    type Output = Self;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        ::scanner::QuotedString::scan_from(s)
    }
}

scanner! { impl<'a, T> ScanFromStr for Range<T> => Range {
    (let a: T, "..", let b: T, ..tail) => (a..b, tail)
}}

scanner! { impl<'a, T> ScanFromStr for RangeFrom<T> => RangeFrom {
    (let a: T, "..", ..tail) => (a.., tail)
}}

scanner! { impl<'a, T> ScanFromStr for RangeTo<T> => RangeTo {
    ("..", let b: T, ..tail) => (..b, tail)
}}

impl<'a> ScanFromStr<'a> for RangeFull {
    type Output = Self;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        use ::input::ScanCursor;
        scan! { s.to_cursor();
            ("..", ^..tail) => (.., tail.offset())
        }
    }
}

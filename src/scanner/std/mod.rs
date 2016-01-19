mod collections;
mod net;

use ::ScanErrorKind;
use ::scanner::{ScanFromStr, ScanSelfFromStr};
use ::scanner::util::StrUtil;

impl<'a> ScanFromStr<'a> for String {
    type Output = String;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        <&str>::scan_from(s).map(|(v, n)| (v.to_owned(), n))
    }
}

macro_rules! impl_tuple {
    () => {};

    ($head:ident $($tail:ident)*) => {
        impl<'a, $head $(, $tail)*> ScanFromStr<'a> for ($head, $($tail,)*)
        where $head: ScanSelfFromStr<'a>, $($tail: ScanSelfFromStr<'a>,)* {
            type Output = Self;
            fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
                #![allow(non_snake_case)]
                scan!(s;
                    ("(", let $head, $(",", let $tail,)* [","]?, ")", ..tail)
                    => (($head, $($tail,)*), s.subslice_offset(tail).unwrap())
                ).map_err(|e| e.kind)
            }
        }

        impl_tuple! { $($tail)* }
    };
}

impl_tuple! { T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 }

impl<'a> ScanFromStr<'a> for () {
    type Output = Self;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        scan!(s; ("(", ")", ..tail) => ((), s.subslice_offset(tail).unwrap())).map_err(|e| e.kind)
    }
}

macro_rules! impl_array {
    (@as_item $i:item) => { $i };
    (@replace.expr $_tt:tt $sub:expr) => { $sub };

    () => {};

    ($len:tt $e0:ident $($ns:tt $es:ident)*) => {
        impl_array! {
            @as_item
            impl<'a, T> ScanFromStr<'a> for [T; $len] where T: ScanSelfFromStr<'a> {
                type Output = Self;
                fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
                    scan!(s;
                        ("[", let $e0, $(",", let $es,)* [","]?, "]", ..tail)
                        => ([$e0, $($es,)*], s.subslice_offset(tail).unwrap())
                    ).map_err(|e| e.kind)
                }
            }
        }

        impl_array! { $($ns $es)* }
    };
}

impl_array! {
    32 e32 31 e31
    30 e30 29 e29 28 e28 27 e27 26 e26 25 e25 24 e24 23 e23 22 e22 21 e21
    20 e20 19 e19 18 e18 17 e17 16 e16 15 e15 14 e14 13 e13 12 e12 11 e11
    10 e10 9 e9 8 e8 7 e7 6 e6 5 e5 4 e4 3 e3 2 e2 1 e1
}

impl<'a, T> ScanFromStr<'a> for [T; 0] {
    type Output = Self;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        scan!(s; ("[", "]", ..tail) => ([], s.subslice_offset(tail).unwrap())).map_err(|e| e.kind)
    }
}

impl<'a, T> ScanFromStr<'a> for Option<T> where T: ScanSelfFromStr<'a> {
    type Output = Self;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        scan!( s;
            ("Some", "(", let v, ")", ..tail) => (Some(v), tail),
            ("None", ..tail) => (None, tail),
        ).map(|(v, t)| (v, s.subslice_offset(t).unwrap()))
            .map_err(|e| e.kind)
    }
}

impl<'a, T, E> ScanFromStr<'a> for Result<T, E>
where T: ScanSelfFromStr<'a>, E: ScanSelfFromStr<'a> {
    type Output = Self;
    fn scan_from(s: &'a str) -> Result<(Self::Output, usize), ScanErrorKind> {
        scan!( s;
            ("Some", "(", let v, ")", ..tail) => (Ok(v), tail),
            ("Err", "(", let v, ")", ..tail) => (Err(v), tail),
        ).map(|(v, t)| (v, s.subslice_offset(t).unwrap()))
            .map_err(|e| e.kind)
    }
}

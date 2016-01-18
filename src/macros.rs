#[macro_export]
macro_rules! scan {
    ($input:expr; $($pattern:tt)*) => {
        quickscan_impl!(@scan ($input); $($pattern)*)
    };
}

#[macro_export]
macro_rules! quickscan_impl {
    /*

    # `@scan` - parse scan pattern.

    */

    /*
    ## Termination rule.
    */
    (@scan ($cur:expr); => $body:expr) => {
        {
            match $crate::ScanInput::try_end($cur) {
                Ok(()) => Ok($body),
                Err((err, _)) => Err(err)
            }
        }
    };

    /*
    ## Tail capture.
    */
    (@scan ($cur:expr); .. _ => $($tail:tt)*) => {
        {
            match $crate::ScanInput::try_scan_raw($cur, |s| Ok::<_, $crate::ScanErrorKind>((s, s.len()))) {
                Ok((_, new_cur)) => quickscan_impl!(@scan (new_cur); => $($tail)*),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); ..$name:ident => $($tail:tt)*) => {
        {
            match $crate::ScanInput::try_scan_raw($cur, |s| Ok::<_, $crate::ScanErrorKind>((s, s.len()))) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); => $($tail)*),
                Err((err, _)) => Err(err)
            }
        }
    };

    /*
    ## Value capture.
    */
    (@scan ($cur:expr); let $name:ident, $($tail:tt)*) => {
        {
            match $crate::ScanInput::try_scan($cur, $crate::ScanSelfFromStr::scan_self_from) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); $($tail)*),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); let $name:ident => $($tail:tt)*) => {
        quickscan_impl!(@scan ($cur); let $name, => $($tail)*)
    };

    (@scan ($cur:expr); let $name:ident: $t:ty, $($tail:tt)*) => {
        {
            match $crate::ScanInput::try_scan($cur, <$t as $crate::ScanFromStr>::scan_from) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); $($tail)*),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); let $name:ident: $t:ty => $($tail:tt)*) => {
        quickscan_impl!(@scan ($cur); let $name: $t, => $($tail)*)
    };

    /*
    ## Literal match.
    */
    (@scan ($cur:expr); $lit:expr, $($tail:tt)*) => {
        match $crate::ScanInput::try_match_literal($cur, $lit) {
            Ok(new_cur) => quickscan_impl!(@scan (new_cur); $($tail)*),
            Err((err, _)) => Err(err)
        }
    };

    (@scan ($cur:expr); $lit:expr => $($tail:tt)*) => {
        quickscan_impl!(@scan ($cur); $lit, => $($tail)*)
    };

    // /*
    // # `@err` - Process error for return to caller.
    // */
    // (@err $err:expr) => {
    //     Err(Box::<::std::error::Error>::from($err))
    // };
}

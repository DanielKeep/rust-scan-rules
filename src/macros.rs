#[macro_export]
macro_rules! scan {
    ($input:expr;
        ($($head_pattern:tt)*) => $head_body:expr
        $(, ($($tail_patterns:tt)*) => $tail_bodies:expr)* $(,)*
    ) => {
        {
            let cur: $crate::Cursor = ::std::convert::Into::into($input);

            let result = quickscan_impl!(@scan (cur.clone()); ($($head_pattern)*,) => $head_body);

            $(
                let result = match result {
                    Ok(v) => Ok(v),
                    Err(last_err) => match quickscan_impl!(@scan (cur.clone()); ($($tail_patterns)*,) => $tail_bodies) {
                        Ok(v) => Ok(v),
                        Err(new_err) => Err(last_err.furthest_along(new_err))
                    }
                };
            )*

            result
        }
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
    (@scan ($cur:expr); () => $body:expr) => {
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
    (@scan ($cur:expr); (.._,) => $body:expr) => {
        {
            match $crate::ScanInput::try_scan_raw($cur, |s| Ok::<_, $crate::ScanErrorKind>((s, s.len()))) {
                Ok((_, new_cur)) => quickscan_impl!(@scan (new_cur); () => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (..$name:ident,) => $body:expr) => {
        {
            match $crate::ScanInput::try_scan_raw($cur, |s| Ok::<_, $crate::ScanErrorKind>((s, s.len()))) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); () => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    /*
    ## Anchor capture.
    */
    (@scan ($cur:expr); (^..$name:ident,) => $body:expr) => {
        {
            let $name = $cur;
            Ok($body)
        }
    };

    /*
    ## Value capture.
    */
    (@scan ($cur:expr); (let _: $t:ty, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::ScanInput::try_scan($cur, <$t as $crate::ScanFromStr>::scan_from) {
                Ok((_, new_cur)) => quickscan_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let $name:ident, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::ScanInput::try_scan($cur, $crate::ScanSelfFromStr::scan_self_from) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let $name:ident: $t:ty, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::ScanInput::try_scan($cur, <$t as $crate::ScanFromStr>::scan_from) {
                Ok(($name, new_cur)) => quickscan_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    /*
    ## Repeating entry.
    */
    /*
    ### No separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*]?, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {0, Some(1)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]*, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {0, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]+, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {1, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{,$max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {0, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$n:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {$n, Some($n)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$min:expr,}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {$min, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$min:expr, $max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (), {$min, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    /*
    ### Comma separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*],?, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {0, Some(1)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],*, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {0, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],+, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {1, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{,$max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {0, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$n:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {$n, Some($n)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$min:expr,}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {$min, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$min:expr, $max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], (","), {$min, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    /*
    ### Sub-pattern separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)?, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, Some(1)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)*, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)+, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {1, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){,$max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$n:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$n, Some($n)}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$min:expr,}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$min, None}, Vec<_>; ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$min:expr, $max:expr}, $($tail:tt)*) => $body:expr) => {
        quickscan_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$min, Some($max)}, Vec<_>; ($($tail)*) => $body)
    };

    /*
    ## Literal match.
    */
    (@scan ($cur:expr); ($lit:expr, $($tail:tt)*) => $body:expr) => {
        match $crate::ScanInput::try_match_literal($cur, $lit) {
            Ok(new_cur) => quickscan_impl!(@scan (new_cur); ($($tail)*) => $body),
            Err((err, _)) => Err(err)
        }
    };

    /*

    # `@repeat` - Repetition expansion.

    */
    (@repeat ($cur:expr),
        [$($pat:tt)*], ($($sep:tt)*), {$min:expr, $max:expr}, $col_ty:ty;
        $($tail:tt)*
    ) => {
        {
            let mut cur = $cur;
            let mut repeats: usize = 0;
            let min: usize = $min;
            let max: ::std::option::Option<usize> = $max;
            quickscan_impl!(@with_bindings ($($pat)*), then: quickscan_impl!(@repeat.define_cols $col_ty,););
            quickscan_impl!(@with_bindings ($($sep)*), then: quickscan_impl!(@repeat.define_cols $col_ty,););

            match (min, max) {
                (a, Some(b)) if a > b => panic!("assertion failed: `(min <= max)` (min: `{:?}`, max: `{:?}`)", a, b),
                _ => ()
            }

            let mut break_err: Option<$crate::ScanError> = None;
            let mut break_after_sep;

            loop {
                // Doing this here prevents an "does not need to be mut" warning.
                break_after_sep = false;

                match max {
                    ::std::option::Option::Some(max) if max == repeats => break,
                    _ => ()
                }

                quickscan_impl!(@if_empty.expr ($($sep)*) {
                    () // Do nothing.
                } else {
                    if repeats > 0 {
                        match quickscan_impl!(@scan (cur);
                            ($($sep)*, ^..after,) => {
                                cur = after;
                                quickscan_impl!(@with_bindings ($($sep)*), then: quickscan_impl!(@repeat.tuple))
                            }
                        ) {
                            ::std::result::Result::Ok(elems) => {
                                // See below about black-holing.
                                let _ = elems.0;
                                quickscan_impl!(@with_bindings ($($sep)*), then: quickscan_impl!(@repeat.push elems,););
                            },
                            ::std::result::Result::Err(err) => {
                                break_err = Some(err);
                                break;
                            }
                        }
                    }
                });

                match quickscan_impl!(@scan (cur);
                    ($($pat)*, ^..after,) => {
                        cur = after;
                        quickscan_impl!(@with_bindings ($($pat)*), then: quickscan_impl!(@repeat.tuple))
                    }
                ) {
                    ::std::result::Result::Ok(elems) => {
                        // Black-hole the first element to stop Rust from complaining when there are no captures.
                        let _ = elems.0;
                        quickscan_impl!(@with_bindings ($($pat)*), then: quickscan_impl!(@repeat.push elems,););
                        repeats += 1;
                    },
                    ::std::result::Result::Err(err) => {
                        quickscan_impl!(@if_empty.expr ($($sep)*) {
                            () // Do nothing
                        } else {
                            break_after_sep = true
                        });
                        break_err = Some(err);
                        break;
                    }
                }
            }

            if repeats < min || break_after_sep {
                Err(break_err.unwrap())
            } else {
                quickscan_impl!(@scan (cur); $($tail)*)
            }
        }
    };

    /*
    ## `.define_cols`

    Define the collections that repeating variables will be collected into.
    */
    (@repeat.define_cols $col_ty:ty, $(($names:ident, $_idxs:expr),)*) => {
        $(
            let mut $names: $col_ty = ::std::default::Default::default();
        )*
    };

    /*
    ## `.tuple`

    Define a tuple expression that contains the names of the repeating variables.

    The first element is *always* `()` so we can explicitly drop it to avoid unused variable warnings.
    */
    (@repeat.tuple $(($names:ident, $_idxs:expr),)*) => {
        ((), $($names,)*)
    };

    /*
    ## `.push`

    Push captured values into their respective collections.
    */
    (@repeat.push $elems:expr, $(($names:ident, $idxs:tt),)*) => {
        $(
            ::std::iter::Extend::extend(
                &mut $names,
                ::std::iter::once(quickscan_impl!(@as_expr $elems.$idxs))
            )
        )*
    };

    /*

    # `@with_bindings` - Extract all binding names from pattern.

    **Note**: The first element of the tuple will be a `()` which we can explicitly drop to avoid unused variable warnings.  As such, the index counter starts at `1`, not `0`.

    */
    (@with_bindings ($($pat:tt)*), then: $cb_name:ident!$cb_arg:tt) => {
        quickscan_impl!(@with_bindings.step 1, (), ($cb_name $cb_arg); $($pat)*,)
    };

    (@with_bindings ($($pat:tt)*), then: $cb_name:ident!$cb_arg:tt;) => {
        quickscan_impl!(@with_bindings.step 1, (), ($cb_name $cb_arg;); $($pat)*,)
    };

    /*
    ## `.step`

    Step over the next part of the pattern.  If it has a binding, extract it and increment `$i`.

    If there's nothing left in the input, invoke the callback.

    **Note**: tail and anchor captures aren't valid inside repeats.
    */
    (@with_bindings.step
        $_i:expr,
        ($($names:tt)*),
        ($cb_name:ident ($($cb_args:tt)*)); $(,)*
    ) => {
        quickscan_impl!(@as_expr $cb_name!($($cb_args)* $($names)*))
    };

    (@with_bindings.step
        $_i:expr,
        ($($names:tt)*),
        ($cb_name:ident ($($cb_args:tt)*);); $(,)*
    ) => {
        quickscan_impl!(@as_stmt $cb_name!($($cb_args)* $($names)*))
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; let _: $_ty:ty, $($tail:tt)*) => {
        quickscan_impl!(@with_bindings.step $i, $names, $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, ($($names:tt)*), $cb:tt; let $name:ident, $($tail:tt)*) => {
        quickscan_impl!(@with_bindings.inc $i, ($($names)* ($name, $i),), $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, ($($names:tt)*), $cb:tt; let $name:ident: $_ty:ty, $($tail:tt)*) => {
        quickscan_impl!(@with_bindings.inc $i, ($($names)* ($name, $i),), $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]*, $($tail:tt)*) => {
        quickscan_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; $_lit:expr, $($tail:tt)*) => {
        quickscan_impl!(@with_bindings.step $i, $names, $cb; $($tail)*)
    };

    /*
    ## `.inc`

    Increment the index counter.  Because `macro_rules!` is stupid, this is *very* limited in how many identifiers can be transitively within a repeating pattern.
    */
    (@with_bindings.inc  1, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  2, $($tail)*) };
    (@with_bindings.inc  2, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  3, $($tail)*) };
    (@with_bindings.inc  3, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  4, $($tail)*) };
    (@with_bindings.inc  4, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  5, $($tail)*) };
    (@with_bindings.inc  5, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  6, $($tail)*) };
    (@with_bindings.inc  6, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  7, $($tail)*) };
    (@with_bindings.inc  7, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  8, $($tail)*) };
    (@with_bindings.inc  8, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step  9, $($tail)*) };
    (@with_bindings.inc  9, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 10, $($tail)*) };
    (@with_bindings.inc 10, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 11, $($tail)*) };
    (@with_bindings.inc 11, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 12, $($tail)*) };
    (@with_bindings.inc 12, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 13, $($tail)*) };
    (@with_bindings.inc 13, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 14, $($tail)*) };
    (@with_bindings.inc 14, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 15, $($tail)*) };
    (@with_bindings.inc 15, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 16, $($tail)*) };
    (@with_bindings.inc 16, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 17, $($tail)*) };
    (@with_bindings.inc 17, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 18, $($tail)*) };
    (@with_bindings.inc 18, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 19, $($tail)*) };
    (@with_bindings.inc 19, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 20, $($tail)*) };
    (@with_bindings.inc 20, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 21, $($tail)*) };
    (@with_bindings.inc 21, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 22, $($tail)*) };
    (@with_bindings.inc 22, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 23, $($tail)*) };
    (@with_bindings.inc 23, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 24, $($tail)*) };
    (@with_bindings.inc 24, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 25, $($tail)*) };
    (@with_bindings.inc 25, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 26, $($tail)*) };
    (@with_bindings.inc 26, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 27, $($tail)*) };
    (@with_bindings.inc 27, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 28, $($tail)*) };
    (@with_bindings.inc 28, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 29, $($tail)*) };
    (@with_bindings.inc 29, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 30, $($tail)*) };
    (@with_bindings.inc 30, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 31, $($tail)*) };
    (@with_bindings.inc 31, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 32, $($tail)*) };
    (@with_bindings.inc 32, $($tail:tt)*) => { quickscan_impl!(@with_bindings.step 33, $($tail)*) };

    /*

    # Miscellaneous

    */
    (@as_expr $e:expr) => {$e};
    (@as_stmt $s:stmt) => {$s};

    (@if_empty.expr () {$($th:tt)*} else {$($_el:tt)*}) => {
        quickscan_impl!(@as_expr $($th)*)
    };

    (@if_empty.expr ($($_tts:tt)+) {$($_th:tt)*} else {$($el:tt)*}) => {
        quickscan_impl!(@as_expr $($el)*)
    };

    // /*
    // # `@err` - Process error for return to caller.
    // */
    // (@err $err:expr) => {
    //     Err(Box::<::std::error::Error>::from($err))
    // };
}

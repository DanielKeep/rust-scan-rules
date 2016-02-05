/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This module contains the public and crate-wide macros.
*/

/**
Reads a line of text from standard input, then scans it using the provided rules.  The result of the `readln!` invocation is the type of the rule bodies; just as with `match`, all bodies must agree on their result type.

Note that this macro automatically flushes standard output.  As a result, if you use this macro *while* you are holding a lock on standard output, your program will deadlock.

If you wish to read from standard input whilst manually locking standard output, you should use `scan!` directly.

This macro *cannot* be used to capture slices of the input; all captured values must be owned.

See also: [Pattern Syntax](index.html#pattern-syntax), [`try_readln!`](macro.try_readln!.html).

# Panics

Panics if an error is encountered while reading from standard input, or if all rules fail to match.
*/
#[macro_export]
macro_rules! readln {
    ($($rules:tt)*) => {
        match ::std::io::Write::flush(&mut ::std::io::stdout()) {
            Err(err) => panic!("{:?}", err),
            Ok(()) => {
                let mut line = ::std::string::String::new();
                match ::std::io::Stdin::read_line(&::std::io::stdin(), &mut line) {
                    Err(err) => panic!("{:?}", err),
                    Ok(_) => {
                        let line = $crate::internal::strip_line_term(&line);
                        match scan!(line; $($rules)*) {
                            Err(err) => panic!("{:?}", err),
                            Ok(v) => v,
                        }
                    },
                }
            },
        }
    };
}

/**
Reads a line of text from standard input, then scans it using the provided rules.  The result of the `try_readln!` invocation is a `Result<T, ScanError>`, where `T` is the type of the rule bodies; just as with `match`, all bodies must agree on their result type.

See also: [Pattern Syntax](index.html#pattern-syntax), [`readln!`](macro.readln!.html).
*/
#[macro_export]
macro_rules! try_readln {
    ($($rules:tt)*) => {
        match ::std::io::Write::flush(&mut ::std::io::stdout()) {
            Err(err) => Err($crate::ScanError::io(err)),
            Ok(()) => {
                let mut line = ::std::string::String::new();
                match ::std::io::Stdin::read_line(&::std::io::stdin(), &mut line) {
                    Err(err) => Err($crate::ScanError::io(err)),
                    Ok(_) => {
                        let line = $crate::internal::strip_line_term(&line);
                        scan!(line; $($rules)*)
                    },
                }
            },
        }
    };
}

/**
Scans the provided input, using the specified pattern.  All values are bound directly to local variables.

Note that this macro only supports a *single* pattern.

See also: [Pattern Syntax](index.html#pattern-syntax), [`scan!`](macro.scan!.html).

## Examples

```rust
# #[macro_use] extern crate scan_rules;
# use scan_rules::scanner::Word;
# fn main() {
let input = "10¥, うまい棒";
let_scan!(input; (let cost: u32, "¥,", let product: Word));
println!("One {} costs {}¥.", product, cost);
# }
```

## Panics

Panics if the pattern fails to match.
*/
#[macro_export]
macro_rules! let_scan {
    ($input:expr; ($($pattern:tt)*)) => {
        scan_rules_impl!(@with_bindings ($($pattern)*),
            then: scan_rules_impl!(@let_bindings.panic $input, ($($pattern)*),);)
    };
}

/**
Scans the provided input, using the specified rules.  The result is a `Result<T, ScanError>` where `T` is the type of the rule bodies; just as with `match`, all bodies must agree on their result type.

The input may be any value which implements `IntoScanCursor`, which includes `&str`, `String`, and `Cow<str>`.

See also: [Pattern Syntax](index.html#pattern-syntax).
*/
#[macro_export]
macro_rules! scan {
    ($input:expr;
        $(($($patterns:tt)*) => $bodies:expr),+
    ) => {
        scan!($input; $(($($patterns)*) => $bodies,)+)
    };

    ($input:expr;
        ($($head_pattern:tt)*) => $head_body:expr
        , $(($($tail_patterns:tt)*) => $tail_bodies:expr,)*
    ) => {
        {
            let cur = $crate::input::IntoScanCursor::into_scan_cursor($input);

            let result = scan_rules_impl!(@scan (cur.clone()); ($($head_pattern)*,) => $head_body);

            $(
                let result = match result {
                    Ok(v) => Ok(v),
                    Err(last_err) => match scan_rules_impl!(@scan (cur.clone()); ($($tail_patterns)*,) => $tail_bodies) {
                        Ok(v) => Ok(v),
                        Err(new_err) => Err(last_err.furthest_along(new_err))
                    }
                };
            )*

            result
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! scan_rules_impl {
    /*

    # `@scan` - parse scan pattern.

    */

    /*
    ## Termination rule.
    */
    (@scan ($cur:expr); () => $body:expr) => {
        {
            match $crate::input::ScanCursor::try_end($cur) {
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
            match $crate::input::ScanCursor::try_scan_raw(
                $cur,
                |s| {
                    let s = $crate::input::ScanInput::as_str(&s);
                    Ok::<_, $crate::ScanError>((s, s.len()))
                }
            ) {
                Ok((_, new_cur)) => scan_rules_impl!(@scan (new_cur); () => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (..$name:ident,) => $body:expr) => {
        {
            match $crate::input::ScanCursor::try_scan_raw(
                $cur,
                |s| {
                    let s = $crate::input::ScanInput::as_str(&s);
                    Ok::<_, $crate::ScanError>((s, s.len()))
                }
            ) {
                Ok(($name, new_cur)) => scan_rules_impl!(@scan (new_cur); () => $body),
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
            match $crate::internal::try_scan_static::<_, $t>($cur) {
                Ok((_, new_cur)) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let _ <| $s:expr, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::internal::try_scan_runtime($cur, &mut $s) {
                Ok((_, new_cur)) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let $name:ident, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::internal::try_scan_static_self($cur) {
                Ok(($name, new_cur)) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let $name:ident: $t:ty, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::internal::try_scan_static::<_, $t>($cur) {
                Ok(($name, new_cur)) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    (@scan ($cur:expr); (let $name:ident <| $s:expr, $($tail:tt)*) => $body:expr) => {
        {
            match $crate::internal::try_scan_runtime($cur, &mut $s) {
                Ok(($name, new_cur)) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
                Err((err, _)) => Err(err)
            }
        }
    };

    /*
    ## Repeating entry.

    This is a *tremendous* discomfort in the posterior.  Without alternation, the only way to get the desired syntax is to exhaustively *list* the various combinations, recursing into another invocation to normalise everything.

    It's a small miracle that the ascription syntax works, though I daresay any user who accidentally types `[...]*: T: U` is going to be *very* confused.

    The next few sections are divided first by separator, then by repetition count control.
    */
    /*
    ### No separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*]? $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {0, Some(1)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]* $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {0, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]+ $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {1, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{,$max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {0, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$n:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {$n, Some($n)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$min:expr,} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {$min, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]{$min:expr, $max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (), {$min, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    /*
    ### Comma separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*],? $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {0, Some(1)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],* $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {0, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],+ $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {1, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{,$max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {0, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$n:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {$n, Some($n)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$min:expr,} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {$min, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*],{$min:expr, $max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], (","), {$min, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    /*
    ### Sub-pattern separator.
    */
    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)? $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, Some(1)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)* $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*)+ $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {1, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){,$max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {0, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$n:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$n, Some($n)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$min:expr,} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$min, None}, ($($col_ty)*); ($($tail)*) => $body)
    };

    (@scan ($cur:expr); ([$($pat:tt)*]($($sep:tt)*){$min:expr, $max:expr} $(: $col_ty:ty)*, $($tail:tt)*) => $body:expr) => {
        scan_rules_impl!(@repeat ($cur), [$($pat)*], ($($sep)*), {$min, Some($max)}, ($($col_ty)*); ($($tail)*) => $body)
    };

    /*
    ## Literal match.
    */
    (@scan ($cur:expr); ($lit:expr, $($tail:tt)*) => $body:expr) => {
        match $crate::input::ScanCursor::try_match_literal($cur, $lit) {
            Ok(new_cur) => scan_rules_impl!(@scan (new_cur); ($($tail)*) => $body),
            Err((err, _)) => Err(err)
        }
    };

    /*

    # `@repeat` - Repetition expansion.

    The first step here is to handle a missing `$col_ty` by replacing it with `Vec<_>`.  We delegate to `.with_col_ty` to handle the rest.

    */
    (@repeat ($cur:expr),
        [$($pat:tt)*], ($($sep:tt)*), {$min:expr, $max:expr}, ();
        $($tail:tt)*
    ) => {
        scan_rules_impl!(@repeat.with_col_ty ($cur), [$($pat)*], ($($sep)*), {$min, $max}, Vec<_>; $($tail)*)
    };

    (@repeat ($cur:expr),
        [$($pat:tt)*], ($($sep:tt)*), {$min:expr, $max:expr}, ($col_ty:ty);
        $($tail:tt)*
    ) => {
        scan_rules_impl!(@repeat.with_col_ty ($cur), [$($pat)*], ($($sep)*), {$min, $max}, $col_ty; $($tail)*)
    };

    /*
    ## `.with_col_ty`

    This handles the bulk of the repetition expansion.  The only somewhat obtuse part is how captures are handled: we have to define a collection to hold every value captured in both the repeating and separator sub-patterns.

    This will go rather *poorly* if someone is silly enough to use the same name more than once... but then, that's a bad idea in general.
    */
    (@repeat.with_col_ty ($cur:expr),
        [$($pat:tt)*], ($($sep:tt)*), {$min:expr, $max:expr}, $col_ty:ty;
        $($tail:tt)*
    ) => {
        {
            let mut cur = $cur;
            let mut repeats: usize = 0;
            let min: usize = $min;
            let max: ::std::option::Option<usize> = $max;
            scan_rules_impl!(@with_bindings ($($pat)*), then: scan_rules_impl!(@repeat.define_cols $col_ty,););
            scan_rules_impl!(@with_bindings ($($sep)*), then: scan_rules_impl!(@repeat.define_cols $col_ty,););

            match (min, max) {
                (a, Some(b)) if a > b => panic!("assertion failed: `(min <= max)` (min: `{:?}`, max: `{:?}`)", a, b),
                _ => ()
            }

            // If we broke out of the loop due to a scanning error, what was it?
            let mut break_err: Option<$crate::ScanError> = None;

            // Did we break due to a scanning error *after* having successfully scanned a separator?
            let mut break_after_sep: bool;

            loop {
                // Doing this here prevents an "does not need to be mut" warning.
                break_after_sep = false;

                match max {
                    ::std::option::Option::Some(max) if max == repeats => break,
                    _ => ()
                }

                // Handle the separator pattern, if there is one.
                scan_rules_impl!(@if_empty.expr ($($sep)*) {
                    () // Do nothing.
                } else {
                    if repeats > 0 {
                        match scan_rules_impl!(@scan (cur.clone());
                            ($($sep)*, ^..after,) => {
                                cur = after;
                                scan_rules_impl!(@with_bindings ($($sep)*), then: scan_rules_impl!(@repeat.tuple))
                            }
                        ) {
                            ::std::result::Result::Ok(elems) => {
                                // See below about black-holing.
                                let _ = elems.0;
                                scan_rules_impl!(@with_bindings ($($sep)*), then: scan_rules_impl!(@repeat.push elems,););
                            },
                            ::std::result::Result::Err(err) => {
                                break_err = Some(err);
                                break;
                            }
                        }
                    }
                });

                // Scan the repeating pattern.
                match scan_rules_impl!(@scan (cur.clone());
                    ($($pat)*, ^..after,) => {
                        cur = after;
                        scan_rules_impl!(@with_bindings ($($pat)*), then: scan_rules_impl!(@repeat.tuple))
                    }
                ) {
                    ::std::result::Result::Ok(elems) => {
                        // Black-hole the first element to stop Rust from complaining when there are no captures.
                        let _ = elems.0;
                        scan_rules_impl!(@with_bindings ($($pat)*), then: scan_rules_impl!(@repeat.push elems,););
                        repeats += 1;
                    },
                    ::std::result::Result::Err(err) => {
                        scan_rules_impl!(@if_empty.expr ($($sep)*) {
                            () // Do nothing
                        } else {
                            break_after_sep = repeats > 0
                        });
                        break_err = Some(err);
                        break;
                    }
                }
            }

            if repeats < min || break_after_sep {
                // Evaluate to the last error because *either* we didn't get enough elements, *or* because we found a separator that wasn't followed by a match.
                Err(break_err.unwrap())
            } else {
                scan_rules_impl!(@scan (cur); $($tail)*)
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
                ::std::iter::once(scan_rules_impl!(@as_expr $elems.$idxs))
            )
        )*
    };

    /*

    # `@let_bindings`

    This is a callback designed to continue from `@with_bindings`.  It takes the list of binding names, and defines local variables for them, and sets up the pattern body to return them.

    */

    (@let_bindings.panic $input:expr, $pattern:tt, $(($ns:ident, $_is:tt),)*) => {
        scan_rules_impl!(
            @as_stmt
            let ($($ns,)*) = match $input {
                input => match scan!(&input[..]; $pattern => ($($ns,)*)) {
                    Ok(vs) => vs,
                    Err(err) => panic!("error while scanning `{:?}`: {}", input, err)
                }
            }
        );
    };

    /*

    # `@with_bindings` - Extract all binding names from pattern.

    The callback will be invoked with `(a, 1), (x, 2), (vvv, 3), ...,` appended to the argument.  This will be a list of every binding name in the pattern in lexical order, plus a matching ordinal.

    **Note**: The first element of the tuple will be a `()` which we can explicitly drop to avoid unused variable warnings.  As such, the index counter starts at `1`, not `0`.

    **Note**: tail and anchor captures aren't valid inside repeats, so they are *not* handled by this macro.

    */
    (@with_bindings ($($pat:tt)*), then: $cb_name:ident!$cb_arg:tt) => {
        scan_rules_impl!(@with_bindings.step 1, (), ($cb_name $cb_arg); $($pat)*,)
    };

    (@with_bindings ($($pat:tt)*), then: $cb_name:ident!$cb_arg:tt;) => {
        scan_rules_impl!(@with_bindings.step 1, (), ($cb_name $cb_arg;); $($pat)*,)
    };

    /*
    ## `.step`

    Step over the next part of the pattern.  If it has a binding, extract it and increment `$i`.

    If there's nothing left in the input, invoke the callback.
    */
    (@with_bindings.step
        $_i:expr,
        ($($names:tt)*),
        ($cb_name:ident ($($cb_args:tt)*)); $(,)*
    ) => {
        scan_rules_impl!(@as_expr $cb_name!($($cb_args)* $($names)*))
    };

    (@with_bindings.step
        $_i:expr,
        ($($names:tt)*),
        ($cb_name:ident ($($cb_args:tt)*);); $(,)*
    ) => {
        scan_rules_impl!(@as_stmt $cb_name!($($cb_args)* $($names)*))
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; let _: $_ty:ty, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; let _ <| $_s:expr, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, ($($names:tt)*), $cb:tt; let $name:ident, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.inc $i, ($($names)* ($name, $i),), $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, ($($names:tt)*), $cb:tt; let $name:ident: $_ty:ty, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.inc $i, ($($names)* ($name, $i),), $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, ($($names:tt)*), $cb:tt; let $name:ident <| $_s:expr, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.inc $i, ($($names)* ($name, $i),), $cb; $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]? $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]* $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]+ $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]{$($_bounds:tt)*} $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*],? $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*],* $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*],+ $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*],{$($_bounds:tt)*} $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]($($sep:tt)*)? $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($sep)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]($($sep:tt)*)* $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($sep)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]($($sep:tt)*)+ $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($sep)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; [$($pat:tt)*]($($sep:tt)*){$($_bounds:tt)*} $(: $col_ty:ty)*, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($pat)*, $($sep)*, $($tail)*)
    };

    (@with_bindings.step $i:tt, $names:tt, $cb:tt; $_lit:expr, $($tail:tt)*) => {
        scan_rules_impl!(@with_bindings.step $i, $names, $cb; $($tail)*)
    };

    /*
    ## `.inc`

    Increment the index counter.  Because `macro_rules!` is stupid, this is *very* limited in how many identifiers can be transitively within a repeating pattern.
    */
    (@with_bindings.inc  1, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  2, $($tail)*) };
    (@with_bindings.inc  2, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  3, $($tail)*) };
    (@with_bindings.inc  3, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  4, $($tail)*) };
    (@with_bindings.inc  4, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  5, $($tail)*) };
    (@with_bindings.inc  5, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  6, $($tail)*) };
    (@with_bindings.inc  6, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  7, $($tail)*) };
    (@with_bindings.inc  7, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  8, $($tail)*) };
    (@with_bindings.inc  8, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step  9, $($tail)*) };
    (@with_bindings.inc  9, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 10, $($tail)*) };
    (@with_bindings.inc 10, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 11, $($tail)*) };
    (@with_bindings.inc 11, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 12, $($tail)*) };
    (@with_bindings.inc 12, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 13, $($tail)*) };
    (@with_bindings.inc 13, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 14, $($tail)*) };
    (@with_bindings.inc 14, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 15, $($tail)*) };
    (@with_bindings.inc 15, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 16, $($tail)*) };
    (@with_bindings.inc 16, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 17, $($tail)*) };
    (@with_bindings.inc 17, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 18, $($tail)*) };
    (@with_bindings.inc 18, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 19, $($tail)*) };
    (@with_bindings.inc 19, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 20, $($tail)*) };
    (@with_bindings.inc 20, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 21, $($tail)*) };
    (@with_bindings.inc 21, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 22, $($tail)*) };
    (@with_bindings.inc 22, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 23, $($tail)*) };
    (@with_bindings.inc 23, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 24, $($tail)*) };
    (@with_bindings.inc 24, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 25, $($tail)*) };
    (@with_bindings.inc 25, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 26, $($tail)*) };
    (@with_bindings.inc 26, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 27, $($tail)*) };
    (@with_bindings.inc 27, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 28, $($tail)*) };
    (@with_bindings.inc 28, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 29, $($tail)*) };
    (@with_bindings.inc 29, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 30, $($tail)*) };
    (@with_bindings.inc 30, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 31, $($tail)*) };
    (@with_bindings.inc 31, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 32, $($tail)*) };
    (@with_bindings.inc 32, $($tail:tt)*) => { scan_rules_impl!(@with_bindings.step 33, $($tail)*) };

    /*

    # Miscellaneous

    */
    (@as_expr $e:expr) => {$e};
    (@as_stmt $s:stmt) => {$s};

    (@if_empty.expr () {$($th:tt)*} else {$($_el:tt)*}) => {
        scan_rules_impl!(@as_expr $($th)*)
    };

    (@if_empty.expr ($($_tts:tt)+) {$($_th:tt)*} else {$($el:tt)*}) => {
        scan_rules_impl!(@as_expr $($el)*)
    };
}

/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!

This crate provides some macros for quickly parsing values out of text.  Roughly speaking, it does the inverse of the `print!`/`format!` macros; or, in other words, a similar job to `scanf` from C.

The macros of interest are:

* [`readln!`](macro.readln!.html) - reads and scans a line from standard input.
* [`try_readln!`](macro.try_readln!.html) - like `readln!`, except it returns a `Result` instead of panicking.
* [`scan!`](macro.scan!.html) - scans the provided string.

Plus a convenience macro:

* [`let_scan!`](macro.let_scan!.html) - scans a string and binds captured values directly to local variables.  Only supports *one* pattern and panics if it doesn't match.

If you are interested in implementing support for your own types, see the [`ScanFromStr`](scanner/trait.ScanFromStr.html) and [`ScanStr`](scanner/trait.ScanStr.html) traits.

The provided scanners can be found in the [`scanner`](scanner/index.html) module.

<style type="text/css">
.link-block { font-family: "Fira Sans"; }
.link-block > p { display: inline-block; }
.link-block > p > strong { font-weight: 500; margin-right: 1em; }
.link-block > ul { display: inline-block; padding: 0; list-style: none; }
.link-block > ul > li {
  font-size: 0.8em;
  background-color: #eee;
  border: 1px solid #ccc;
  padding: 0.3em;
  display: inline-block;
}
</style>
<span></span><div class="link-block">

**Links**

* [Latest Release](https://crates.io/crates/scan-rules/)
* [Latest Docs](https://danielkeep.github.io/rust-scan-rules/doc/scan_rules/index.html)
* [Repository](https://github.com/DanielKeep/rust-scan-rules)

<span></span></div>

## Compatibility

v0.0.4 was tested against `rustc` versions 1.6.0, 1.7.0-beta.1, and nightly 2016-01-20.

* `rustc` versions prior to 1.7 will have only concrete implementations of `ScanFromStr` for the `Everything`, `Ident`, `Line`, `NonSpace`, `Number`, `Word`, and `Wordish` scanners for `&str` and `String` output types.  1.7 and higher will have generic implementations for all output types such that `&str: Into<Output>`.

## Features

The following optional features are available:

* `arrays-32`: implement scanning for arrays of up to 32 elements.  The default is up to 8 elements.

* `tuples-16`: implement scanning for tuples of up to 16 elements.  The default is up to 4 elements.

## Important Notes

* There are no default scanners for `&str` or `String`; if you want a string, you should pick an appropriate abstract scanner from the [`scanner`](scanner/index.html) module.

* The macros in this crate are extremely complex.  Moderately complex usage can exhaust the standard macro recursion limit.  If this happens, you can raise the limit (from its default of 64) by adding the following attribute to your crate's root module:

  `#![recursion_limit="128"]`

## Quick Examples

Here is a simple CLI program that asks the user their name and age.  You can run this using `cargo run --example ask_age`.

```ignore
#[macro_use] extern crate scan_rules;

use scan_rules::scanner::Word;

fn main() {
    print!("What's your name? ");
    let name: String = readln! { (let name: Word<String>) => name };
    //                           ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^ rule
    //                                                       ^~~^ body
    //                           ^~~~~~~~~~~~~~~~~~~~~~~^ pattern
    //                            ^~~~~~~~~~~~~~~~~~~~~^ variable binding

    print!("Hi, {}.  How old are you? ", name);
    readln! {
        (let age) => {
    //   ^~~~~~^ implicitly typed variable binding
            let age: i32 = age;
            println!("{} years old, huh?  Neat.", age);
        },
        (..other) => println!("`{}` doesn't *look* like a number...", other),
    //   ^~~~~~^ bind to any input "left over"
    }
}
```

This example shows how to parse one of several different syntaxes.  You can run this using `cargo run --example scan_data`.

```ignore
#[macro_use] extern crate scan_rules;

use std::collections::BTreeSet;

// `Word` is an "abstract" scanner; rather than scanning itself, it scans some
// *other* type using custom rules.  In this case, it scans a word into a
// string slice.  You can use `Word<String>` to get an owned string.
use scan_rules::scanner::Word;

#[derive(Debug)]
enum Data {
    Vector(i32, i32, i32),
    Truthy(bool),
    Words(Vec<String>),
    Lucky(BTreeSet<i32>),
    Other(String),
}

fn main() {
    print!("Enter some data: ");
    let data = readln! {
        ("<", let x, ",", let y, ",", let z, ">") => Data::Vector(x, y, z),
    //      ^ pattern terms are comma-separated
    //   ^~^ literal text match

        // Rules are tried top-to-bottom, stopping as soon as one matches.
        (let b) => Data::Truthy(b),
        ("yes") => Data::Truthy(true),
        ("no") => Data::Truthy(false),

        ("words:", [ let words: Word<String> ],+) => Data::Words(words),
    //             ^~~~~~~~~~~~~~~~~~~~~~~~~~~~^ repetition pattern
    //                                         ^ one or more matches
    //                                        ^ matches must be comma-separated

        ("lucky numbers:", [ let ns: i32 ]*: BTreeSet<_>) => Data::Lucky(ns),
    //          collect into specific type ^~~~~~~~~~~~^
    //                                    ^ zero or more (you might be unlucky!)
    //                                      (no separator this time)

        // Rather than scanning a sequence of values and collecting them into
        // a `BTreeSet`, we can instead scan the `BTreeSet` *directly*.  This
        // scans the syntax `BTreeSet` uses when printed using `{:?}`:
        // `{1, 5, 13, ...}`.
        ("lucky numbers:", let ns) => Data::Lucky(ns),

        (..other) => Data::Other(String::from(other))
    };
    println!("data: {:?}", data);
}
```

This example demonstrates using runtime scanners and the `let_scan!` convenience macro.  You can run this using `cargo run --example runtime_scanners`.

```ignore
#[macro_use] extern crate scan_rules;

use scan_rules::scanner::{
    NonSpace, Number, Word,             // static scanners
    max_width_a, exact_width_a, re_str, // runtime scanners
};

fn main() {
    // Adapted example from <http://en.cppreference.com/w/cpp/io/c/fscanf>.
    let inp = "25 54.32E-1 Thompson 56789 0123 56ß水";

    // `let_scan!` avoids the need for indentation and braces, but only supports
    // a single pattern, and panics if anything goes wrong.
    let_scan!(inp; (
        let i: i32, let x: f32, let str1 <| max_width_a::<NonSpace>(9),
    //               use runtime scanner ^~~~~~~~~~~~~~~~~~~~~~~~~~~~^
    //          limit maximum width of a... ^~~~~~~~~~^
    //                      ...static NonSpace scanner... ^~~~~~~^
    //                                                      9 bytes ^
        let j <| exact_width_a::<i32>(2), let y: f32, let _: Number,
    //        ^~~~~~~~~~~~~~~~~~~~~~~~~^ scan an i32 with exactly 2 digits
        let str2 <| re_str(r"^[0-9]{1,3}"), let warr: Word
    //           ^~~~~~~~~~~~~~~~~~~~~~~~^ scan using a regular expression
    ));

    println!(
        "Converted fields:\n\
            i = {i:?}\n\
            x = {x:?}\n\
            str1 = {str1:?}\n\
            j = {j:?}\n\
            y = {y:?}\n\
            str2 = {str2:?}\n\
            warr = {warr:?}",
        i=i, j=j, x=x, y=y,
        str1=str1, str2=str2, warr=warr);
}
```

## Rule Syntax

Scanning rules are written as one or more arms like so:

```ignore
scan! { input_expression;
    ( pattern ) => body,
    ( pattern ) => body,
    ...
    ( pattern ) => body,
}
```

Note that the trailing comma on the last rule is optional.

Rules are checked top-to-bottom, stopping at the first that matches.

Patterns (explained under ["Pattern Syntax"](#pattern-syntax)) must be enclosed in parentheses.  If a pattern matches the provided input, the corresponding body is evaluated.

### Pattern Syntax

A scanning pattern is made up of one or more pattern terms, separated by commas.  The following terms are supported:

* *strings* - any expression that evaluates to a string will be used as a literal match on the input.  Exactly *how* this match is done depends on the kind of input, but the default is to do a case-sensitive match of whole words, individual non-letter characters, and to ignore all whitespace.

  *E.g.* `"Two words"`, `"..."` (counts as three "words"), `&format!("{} {}", "Two", "words")`.

* `let` *name* \[ `:` *type* ] - scans a value out of the input text, and binds it to *name*.  If *type* is omitted, it will be inferred.

  *E.g.* `let x`, `let n: i32`, `let words: Vec<_>`, `let _: &str` (scans and discards a value).

* `let` *name* `<|` *expression* - scans a value out of the input text and binds it to *name*, using the value of *expression* to perform the scan.  The expression must evaluate to something that implements the `ScanStr` trait.

  *E.g.* `let n <| scan_a::<i32>()` (same as above example for `n`), `let three_digits <| max_width_a::<u32>()` (scan a three-digit `u32`).

* `..` *name* - binds the remaining, unscanned input as a string to *name*.  This can *only* appear as the final term in a top-level pattern.

* `[` *pattern* `]` \[ *(nothing)* | `,` | `(` *seperator pattern* `)` ] ( `?` | `*` | `+` | `{` *range* `}` ) \[ ":" *collection type* ] - scans *pattern* repeatedly.

  The first (mandatory) part of the term specifies the *pattern* that should be repeatedly scanned.

  The second (optional) part of the term controls if (and what) repeats are separated by.  `,` is provided as a short-cut to an obvious common case; it is equivalent to writing `(",")`.  Otherwise, you may write any arbitrary *separator pattern* as the separator, including variable bindings and more repetitions.

  The third (mandatory) part of the term specifies how many times *pattern* should be scanned.  The available options are:

  * `?` - match zero or one times.
  * `*` - match zero or more times.
  * `+` - match one or more times.
  * `{n}` - match exactly *n* times.
  * `{a,}` - match at least *a* times.
  * `{,b}` - match at most *b* times.
  * `{a, b}` - match at least *a* times, and at most *b* times.

  The fourth (optional) part of the term specifies what type of collection scanned values should be added to.  Note that the type specified here applies to *all* values captured by this repetition.  As such, you typically want to use a partially inferred type such as `BTreeSet<_>`.  If omitted, it defaults to `Vec<_>`.

  *E.g.* `[ let nums: i32 ],+`, `[ "pretty" ]*, "please"`.

*/
#![forbid(missing_docs)]
#![recursion_limit="128"]
#[macro_use] extern crate lazy_static;
extern crate itertools;
extern crate regex;
extern crate strcursor;

#[macro_use] mod macros;

pub use error::{ScanError, ScanErrorKind};

mod error;
pub mod input;
pub mod scanner;

/**
Remove a single trailing line terminator from `s`.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
#[doc(hidden)]
pub fn strip_line_term(s: &str) -> &str {
    if s.ends_with("\r\n") {
        &s[0..s.len()-2]
    } else if s.ends_with("\n") {
        &s[0..s.len()-1]
    } else if s.ends_with("\r") {
        &s[0..s.len()-1]
    } else {
        s
    }
}

/**
Compute the offset of `b`, which must be a subslice of `a`.

This is publicly exposed for the sake of macros and **is not** considered a stable part of the public API.
*/
#[doc(hidden)]
pub fn subslice_offset(a: &str, b: &str) -> Option<usize> {
    use scanner::util::StrUtil;
    a.subslice_offset(b)
}

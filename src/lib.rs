/*!

This crate provides some macros for quickly parsing values out of text.  Roughly speaking, it does the inverse of the `print!`/`format!` macros; or, in other words, a similar job to `scanf` from C.

The macros of interest are:

* [`readln!`](macro.readln!.html) - reads and scans a line from standard input.
* [`try_readln!`](macro.try_readln!.html) - like `readln!`, except it returns a `Result` instead of panicking.
* [`scan!`](macro.scan!.html) - scans the provided string.

If you are interesting in implementing support for your own types, see the [`ScanFromStr`](scanner/trait.ScanFromStr.html) trait.

## Features

The following optional features are available:

* `arrays-32`: implement scanning for arrays of up to 32 elements.  The default is up to 8 elements.

* `tuples-16`: implement scanning for tuples of up to 16 elements.  The default is up to 4 elements.

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

## Pattern Syntax

A scanning pattern is made up of one or more pattern terms, separated by commas.  The following terms are supported:

* *strings* - any expression that evaluates to a string will be used as a literal match on the input.  Exactly *how* this match is done depends on the kind of input, but the default is to do a case-sensitive match of whole words, individual non-letter characters, and to ignore all whitespace.

  *E.g.* `"Two words"`, `"..."` (counts as three "words"), `&format!("{} {}", "Two", "words")`.

* `let` *name* \[ `:` *type* ] - scans a value out of the input text, and binds it to *name*.  If *type* is omitted, it will be inferred.

  *E.g.* `let x`, `let n: i32`, `let words: Vec<_>`, `let _: &str` (scans and discards a value).

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

## Things What Need Mentioning

* *Rule* syntax.
* Scanners and things like `Word` being abstract.
* Cursor(s).
* More examples.
* `#![recursion_limit]`

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

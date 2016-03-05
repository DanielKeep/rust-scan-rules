# `scan-rules`

This crate provides some macros for quickly parsing values out of text.  Roughly speaking, it does the inverse of the `print!`/`format!` macros; or, in other words, a similar job to `scanf` from C.

The macros of interest are:

* `readln!` - reads and scans a line from standard input.
* `try_readln!` - like `readln!`, except it returns a `Result` instead of panicking.
* `scan!` - scans the provided string.

Plus a convenience macro:

* `let_scan!` - scans a string and binds captured values directly to local variables.  Only supports *one* pattern and panics if it doesn't match.

If you are interested in implementing support for your own types, see the `ScanFromStr` trait.

The available abstract scanners can be found in the `scanner` module.

**Links**

* [Latest Release](https://crates.io/crates/scan-rules/)
* [Latest Docs](https://danielkeep.github.io/rust-scan-rules/doc/scan_rules/index.html)
* [Repository](https://github.com/DanielKeep/rust-scan-rules)

## Compatibility

v0.1.2 was tested against `rustc` versions 1.3.0-1.6.0, 1.8.0-beta.1, and nightly 2016-03-04.

* `rustc` < 1.7 will have only concrete implementations of `ScanFromStr` for the `Everything`, `Ident`, `Line`, `NonSpace`, `Number`, `Word`, and `Wordish` scanners for `&str` and `String` output types.  1.7 and higher will have generic implementations for all output types such that `&str: Into<Output>`.

* `rustc` < 1.5 will not support scanning the `SocketAddrV4` and `SocketAddrV6` types, due to missing `FromStr` implementations.

* `rustc` < 1.4 will not support scanning the `Ipv4Addr`, `Ipv6Addr` or `SocketAddr` types, due to their `FromStr` implementations producing errors that do not implement `Error`.

* `rustc` < 1.3 is explicitly not supported, due to limitations in the macro syntax.

## Quick Examples

Here is a simple CLI program that asks the user their name and age.  You can run this using `cargo run --example ask_age`.

```rust
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

```rust
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

```rust
//! **NOTE**: requires the `regex` feature.
#[macro_use] extern crate scan_rules;

fn main() {
    use scan_rules::scanner::{
        NonSpace, Number, Word,             // static scanners
        max_width_a, exact_width_a, re_str, // runtime scanners
    };

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

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

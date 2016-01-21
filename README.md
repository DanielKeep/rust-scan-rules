# `scan-rules`

This crate provides some macros for quickly parsing values out of text.  Roughly speaking, it does the inverse of the `print!`/`format!` macros; or, in other words, a similar job to `scanf` from C.

The macros of interest are:

* `readln!` - reads and scans a line from standard input.
* `try_readln!` - like `readln!`, except it returns a `Result` instead of panicking.
* `scan!` - scans the provided string.

If you are interested in implementing support for your own types, see the `ScanFromStr` trait.

The available abstract scanners can be found in the `scanner` module.

**Links**

* [Latest Release](https://crates.io/crates/scan-rules/)
* [Latest Docs](https://danielkeep.github.io/rust-scan-rules/doc/scan_rules/index.html)
* [Repository](https://github.com/DanielKeep/rust-scan-rules)

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

## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

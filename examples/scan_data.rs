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

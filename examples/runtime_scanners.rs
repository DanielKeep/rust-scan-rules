/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
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

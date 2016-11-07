/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
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

    question_three();
}

// Only works in Rust 1.10 and up.
#[cfg(macro_inter_stmt_binding_visibility)]
fn question_three() {
    print!("Ok.  What... is your favourite colour? (R, G, B): ");
    let_readln!(let r: f32, ",", let g: f32, ",", let b: f32);
    //          ^~~~^            ^~~~^            ^~~~^
    // Scans and binds three variables without nesting scope.
    // Panics if *anything* goes wrong.
    if !(g < r && g < b && b >= r * 0.25 && b <= r * 0.75) {
        println!("Purple's better.");
    } else {
        println!("Good choice!");
    }
}

#[cfg(not(macro_inter_stmt_binding_visibility))]
fn question_three() {}

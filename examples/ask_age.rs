#[macro_use] extern crate scan_rules;

fn main() {
    print!("What's your name? ");
    let name: String = readln! { (let name) => name };
    //                           ^~~~~~~~~~~~~~~~~^ rule
    //                                         ^~~^ body
    //                           ^~~~~~~~~^ pattern
    //                            ^~~~~~~^ variable binding

    print!("Hi, {}.  How old are you? ", name);
    readln! {
        (let age: i32) => println!("{} years old, huh?  Neat.", age),
    //   ^~~~~~~~~~~^ explicitly typed variable binding
        (..other) => println!("`{}` doesn't *look* like a number...", other),
    //   ^~~~~~^ bind to any input "left over"
    }
}

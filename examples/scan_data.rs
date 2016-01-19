#[macro_use] extern crate scan_rules;

#[derive(Debug)]
enum Data {
    Vector(i32, i32, i32),
    Truthy(bool),
    Words(Vec<String>),
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

        ("words:", [ let words ],+) => Data::Words(words),
    //             ^~~~~~~~~~~~~~^ repetition pattern
    //                           ^ one or more matches
    //                          ^ matches must be comma-separated

        (..other) => Data::Other(String::from(other))
    };
    println!("data: {:?}", data);
}

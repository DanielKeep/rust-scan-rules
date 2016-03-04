/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#[macro_use] extern crate scan_rules;
#[macro_use] mod util;

use scan_rules::scanner::{HorSpace, Newline, Space, max_width_a};

#[test]
fn test_scan_space() {
    let inp = "  \t \n x\r\n y z \t\r ";

    assert_match!(
        scan!(inp; (let a: Space, "x", let b: Space, "y z", let c: Space, ..tail) => (a, b, c, tail)),
        Ok(("  \t \n ", "\r\n ", " \t\r ", ""))
    );

    assert_match!(
        scan!(inp; (let a: Space, "x", let b: Space, "y", "z", let c: Space, ..tail) => (a, b, c, tail)),
        Ok(("  \t \n ", "\r\n ", " \t\r ", ""))
    );

    assert_match!(
        scan!(inp; (let a: Space, "x", let b: Space, "y", let c: Space, "z", let d: Space, ..tail) => (a, b, c, d, tail)),
        Ok(("  \t \n ", "\r\n ", " ", " \t\r ", ""))
    );

    assert_match!(
        scan!(inp; (let a <| max_width_a::<Space>(3), let b: Space, "x", ..tail) => (a, b, tail)),
        Ok(("  \t", " \n ", "\r\n y z \t\r "))
    );

    assert_match!(
        scan!(inp; (
            let a: HorSpace, let b: Newline, let c: HorSpace, "x",
            let d: Newline, "y",
            let e: Space, "z",
            let f: Space
        ) => (a, b, c, d, e, f)),
        Ok(("  \t ", "\n", " ", "\r\n", " ", " \t\r "))
    );
}

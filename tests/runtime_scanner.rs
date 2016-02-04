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

use scan_rules::scanner::{
    Octal, Hex,
    exact_width_a, max_width_a, min_width_a,
};

#[test]
fn test_runtime_scanner() {
    assert_match!(
        scan!("0123456789"; (
                let a <| exact_width_a::<i32>(3),
                let b <| max_width_a::<Hex<i32>>(2),
                let c <| min_width_a::<Octal<i32>>(2),
                ..d
            ) => (a, b, c, d)),
        Ok((12, 0x34, 0o567, "89"))
    );
}

#[cfg(feature="regex")]
#[test]
fn test_runtime_scanner_re() {
    use scan_rules::scanner::re_str;
    assert_match!(
        scan!("0123456789"; (
                let a <| re_str(r"[0-9]{3}[:xdigit:]{0,2}[0-7]{2,}"),
                let b <| re_str(r"9")
            ) => (a, b)),
        Ok(("01234567", "9"))
    );
}

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

use scan_rules::input::{self, StrCursor};

#[test]
fn test_non_space() {
    type Cursor<'a> = StrCursor<'a, input::ExactCompare, input::IgnoreSpace, input::NonSpace>;

    let inp = "a a2c 1b3 a,c*e a2,d+f-8\u{200b}abc";

    assert_match!(
        scan!(inp;
            ("a", "a2c", "1b3", "a", ",", "c", "*", "e",
                "a2", ",", "d", "+", "f", "-", "8\u{200b}abc") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("a", "a2c", "1b3", "a,c*e", "a2,d+f-8\u{200b}abc") => ()),
        Ok(())
    );
}

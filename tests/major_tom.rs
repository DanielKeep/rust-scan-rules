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

use scan_rules::{ScanError, ScanErrorKind};
use scan_rules::input::ScanCursor;
use scan_rules::scanner::Word;

#[test]
fn test_tom() {
    let inp = "Hi  , my name  is \t Major Tom! I was born in 1969.";

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word) => name),
        Err(ScanError { ref at, kind: ScanErrorKind::ExpectedEnd, .. }) if at.offset() == 29
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, "! I was born in 1947.") => name),
        Err(ScanError { ref at, kind: ScanErrorKind::LiteralMismatch, .. }) if at.offset() == 29
    );

    assert_match!(
        scan!(inp; ("hi, my name is major", let name: Word, "! i was born in 1969.") => name),
        Err(ScanError { ref at, kind: ScanErrorKind::LiteralMismatch, .. }) if at.offset() == 0
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, "! I was born in 1969.") => name),
        Ok("Tom")
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, .._) => name),
        Ok("Tom")
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, ..tail) => (name, tail)),
        Ok(("Tom", "! I was born in 1969."))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, ^..tail) => (name, tail.as_str())),
        Ok(("Tom", "! I was born in 1969."))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, "! I was born in", let year, ".") => {
            let year: i32 = year;
            (name, year)
        }),
        Ok(("Tom", 1969))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: Word, "! I was born in", let year: i32, ".") => (name, year)),
        Ok(("Tom", 1969))
    );
}

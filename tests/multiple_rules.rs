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

use scan_rules::ScanError as SE;
use scan_rules::ScanErrorKind as SEK;
use scan_rules::scanner::Word;

#[test]
fn test_multiple_rules() {
    assert_match!(parse(""),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 0);
    assert_match!(parse("wazza: chazza"),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 0);
    assert_match!(parse("line: x y z"),
        Ok(Parsed::Line(" x y z")));
    assert_match!(parse("word: x"),
        Ok(Parsed::Word("x")));
    assert_match!(parse("word: x y z"),
        Err(SE { ref at, kind: SEK::ExpectedEnd, .. }) if at.offset() == 7);
    assert_match!(parse("i32: 42"),
        Ok(Parsed::I32(42)));
    assert_match!(parse("i32: 42.0"),
        Err(SE { ref at, kind: SEK::ExpectedEnd, .. }) if at.offset() == 7);
}

#[derive(Debug)]
enum Parsed<'a> {
    Line(&'a str),
    Word(&'a str),
    I32(i32),
}

fn parse(s: &str) -> Result<Parsed, SE> {
    scan! { s;
        ("line:", ..v) => Parsed::Line(v),
        ("word:", let v: Word) => Parsed::Word(v),
        ("i32:", let v) => Parsed::I32(v),
    }
}

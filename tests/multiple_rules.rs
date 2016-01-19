#[macro_use] extern crate quickscan;
#[macro_use] mod util;

use quickscan::ScanError as SE;
use quickscan::ScanErrorKind as SEK;

#[test]
fn test_multiple_rules() {
    assert_match!(parse(""),
        Err(SE { ref at, kind: SEK::LiteralMismatch }) if at.offset() == 0);
    assert_match!(parse("wazza: chazza"),
        Err(SE { ref at, kind: SEK::LiteralMismatch }) if at.offset() == 0);
    assert_match!(parse("line: x y z"),
        Ok(Parsed::Line(" x y z")));
    assert_match!(parse("word: x"),
        Ok(Parsed::Word("x")));
    assert_match!(parse("word: x y z"),
        Err(SE { ref at, kind: SEK::ExpectedEnd }) if at.offset() == 7);
    assert_match!(parse("i32: 42"),
        Ok(Parsed::I32(42)));
    assert_match!(parse("i32: 42.0"),
        Err(SE { ref at, kind: SEK::ExpectedEnd }) if at.offset() == 7);
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
        ("word:", let v) => Parsed::Word(v),
        ("i32:", let v) => Parsed::I32(v),
    }
}

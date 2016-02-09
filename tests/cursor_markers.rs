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
fn test_exact_space() {
    use scan_rules::ScanError as SE;
    use scan_rules::ScanErrorKind as SEK;

    type Cursor<'a> = StrCursor<'a, input::ExactCompare, input::ExactSpace, input::Wordish>;

    let inp = "one ,two \tbuckle\nmy  shoe ";

    assert_match!(
        scan!(inp;
            ("one", ",", "two", "buckle", "my", "shoe") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one", ",", "two", "buckle", "my", "shoe") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 3
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one", " ", ",", "two", " \t", "buckle",
                "\n", "my", "  ", "shoe", " ") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one", " ", ",", "two", " \t", "buckle",
                "\n", "my", "  ", "shoe") => ()),
        Err(SE { ref at, kind: SEK::ExpectedEnd, .. }) if at.offset() == 25
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one", " ", ",", "two", " ", "\t", "buckle",
                "\n", "my", " ", " ", "shoe", " ") => ()),
        Ok(())
    );
}

#[test]
fn test_fuzzy_space() {
    use scan_rules::ScanError as SE;
    use scan_rules::ScanErrorKind as SEK;

    type Cursor<'a> = StrCursor<'a, input::ExactCompare, input::FuzzySpace, input::Wordish>;

    let inp = "one ,two \tbuckle\nmy  shoe ";

    assert_match!(
        scan!(inp;
            ("one", ",", "two", "buckle", "my", "shoe") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one", ",", "two", "buckle", "my", "shoe") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 3
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one ,two \tbuckle\nmy  shoe ") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one\t,two\n buckle\rmy\t shoe\n") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one , two \tbuckle\nmy  shoe ") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 5
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("one ,two \tbuckle\nmy  shoe") => ()),
        Err(SE { ref at, kind: SEK::ExpectedEnd, .. }) if at.offset() == 25
    );
}

#[test]
fn test_ignore_non_line() {
    type Cursor<'a> = StrCursor<'a, input::ExactCompare, input::IgnoreNonLine, input::Wordish>;

    let inp = "0 1 2\n 3 4 5 \n6 7 8";

    assert_match!(
        scan!(inp;
            ([[let xss: i32]+]+) => xss),
        Ok(ref xss) if *xss == vec![vec![0, 1, 2, 3, 4, 5, 6, 7, 8]]
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ([[let xss: i32]+]("\n")+) => xss),
        Ok(ref xss) if *xss == vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]]
    );
}

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

#[cfg(feature="unicode-normalization")]
#[test]
fn test_normalized() {
    use scan_rules::ScanError as SE;
    use scan_rules::ScanErrorKind as SEK;

    type Cursor<'a> = StrCursor<'a, input::Normalized, input::IgnoreSpace, input::Wordish>;

    let inp = "café bäbe";

    assert_match!(
        scan!(inp;
            ("café bäbe") => ()),
        Ok(())
    );

    assert_match!(
        scan!(inp;
            ("café bäbe") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 0
    );

    assert_match!(
        scan!(inp;
            ("café bäbe") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 6
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("café bäbe") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("café bäbe") => ()),
        Ok(())
    );

    assert_match!(
        scan!(Cursor::new(inp);
            ("café bäbe") => ()),
        Ok(())
    );
}

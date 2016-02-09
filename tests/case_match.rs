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
use scan_rules::input::{StrCursor, ExactCompare, IgnoreCase, IgnoreAsciiCase};

#[test]
fn test_case_match() {
    let inp = "UPPERCASE lowercase mIxeDcAsE TitleCase";

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCaSE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 0
    );

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCASE", "lowerCase", "mIxeDcAsE", "TitleCase") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 10
    );

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCASE", "lowercase", "mIxEdcAsE", "TitleCase") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 20
    );

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitLecAse") => ()),
        Err(SE { ref at, kind: SEK::LiteralMismatch, .. }) if at.offset() == 30
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCaSE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCASE", "lowerCase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxEdcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitLecAse") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCaSE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCASE", "lowerCase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxEdcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitLecAse") => ()),
        Ok(())
    );
}

/**
Make sure the "official" API style for new code works.
*/
#[test]
fn test_case_match_stable_api() {
    let inp = "UPPERCASE lowercase mIxeDcAsE TitleCase";

    assert_match!(
        scan!(StrCursor::<ExactCompare>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );

    assert_match!(
        scan!(StrCursor::<IgnoreAsciiCase>::new(inp);
            ("UPPERCASE", "lowercase", "mIxeDcAsE", "TitleCase") => ()),
        Ok(())
    );
}

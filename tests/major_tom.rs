#[macro_use] extern crate quickscan;
#[macro_use] mod util;

use quickscan::{ScanError, ScanErrorKind};

#[test]
fn test_tom() {
    let inp = "Hi  , my name  is \t Major Tom! I was born in 1969.";

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str) => name),
        Err(ScanError { ref at, kind: ScanErrorKind::ExpectedEnd }) if at.offset() == 29
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, "! I was born in 1947.") => name),
        Err(ScanError { ref at, kind: ScanErrorKind::LiteralMismatch }) if at.offset() == 29
    );

    assert_match!(
        scan!(inp; ("hi, my name is major", let name: &str, "! i was born in 1969.") => name),
        Err(ScanError { ref at, kind: ScanErrorKind::LiteralMismatch }) if at.offset() == 0
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, "! I was born in 1969.") => name),
        Ok("Tom")
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, .._) => name),
        Ok("Tom")
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, ..tail) => (name, tail)),
        Ok(("Tom", "! I was born in 1969."))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, ^..tail) => (name, tail.as_str())),
        Ok(("Tom", "! I was born in 1969."))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name, ..tail) => {
            let name: &str = name;
            (name, tail)
        }),
        Ok(("Tom", "! I was born in 1969."))
    );

    assert_match!(
        scan!(inp; ("Hi, my name is Major", let name: &str, "! I was born in", let year: i32, ".") => (name, year)),
        Ok(("Tom", 1969))
    );
}
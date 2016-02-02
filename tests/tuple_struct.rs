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

use scan_rules::scanner::{Inferred, Word};

#[test]
fn test_tuple_struct_borrowed() {
    #[derive(Debug)]
    struct A<'a>(&'a str, Option<u64>);

    let inp = "A(word, None)";

    assert_match!(
        scan!(inp;
            ("A", "(", let a: Word, ",", let b, ")")
            => A(a, b)
        ),
        Ok(A("word", None))
    );

    assert_match!(
        scan!(inp;
            ("A", let a: (Word, Option<u64>))
            => A(a.0, a.1)
        ),
        Ok(A("word", None))
    );

    assert_match!(
        scan!(inp;
            ("A", let a: (Word, Inferred<Option<u64>>))
            => A(a.0, a.1)
        ),
        Ok(A("word", None))
    );

    assert_match!(
        scan!(inp;
            ("A", let a: (Word, Inferred<_>))
            => A(a.0, a.1)
        ),
        Ok(A("word", None))
    );
}

#[test]
fn test_tuple_struct_owned() {
    #[derive(Debug)]
    struct A(String, Option<u64>);

    let inp = "A(\"word\", None)";

    assert_match!(
        scan!(inp;
            ("A", "(", let a, ",", let b, ")")
            => A(a, b)
        ),
        Ok(A(ref s, None)) if s == "word"
    );

    assert_match!(
        scan!(inp;
            ("A", let a: (String, Option<u64>))
            => A(a.0, a.1)
        ),
        Ok(A(ref s, None)) if s == "word"
    );

    assert_match!(
        scan!(inp;
            ("A", let a: Inferred<(String, Option<u64>)>)
            => A(a.0, a.1)
        ),
        Ok(A(ref s, None)) if s == "word"
    );

    assert_match!(
        scan!(inp;
            ("A", let a: Inferred<(_, _)>)
            => A(a.0, a.1)
        ),
        Ok(A(ref s, None)) if s == "word"
    );
}

#[macro_use] extern crate scan_rules;
#[macro_use] mod util;

use scan_rules::ScanError as SE;
use scan_rules::ScanErrorKind as SEK;

#[test]
fn test_repeating() {
    assert_match!(
        scan!("[]"; ("[", [ let ns: i32 ]?, "]") => ns),
        Ok(ref ns) if *ns == vec![]
    );

    assert_match!(
        scan!("[]"; ("[", [ let ns: i32 ]*, "]") => ns),
        Ok(ref ns) if *ns == vec![]
    );

    assert_match!(
        scan!("[]"; ("[", [ let ns: i32 ]+, "]") => ns),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 1
    );

    assert_match!(
        scan!("[0]"; ("[", [ let ns: i32 ]?, "]") => ns),
        Ok(ref ns) if *ns == vec![0]
    );

    assert_match!(
        scan!("[0]"; ("[", [ let ns: i32 ]*, "]") => ns),
        Ok(ref ns) if *ns == vec![0]
    );

    assert_match!(
        scan!("[0]"; ("[", [ let ns: i32 ]+, "]") => ns),
        Ok(ref ns) if *ns == vec![0]
    );

    assert_match!(
        scan!("[0 1]"; ("[", [ let ns: i32 ]?, "]") => ns),
        Err(SE { ref at, kind: SEK::LiteralMismatch }) if at.offset() == 3
    );

    assert_match!(
        scan!("[0 1]"; ("[", [ let ns: i32 ]*, "]") => ns),
        Ok(ref ns) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("[0 1]"; ("[", [ let ns: i32 ]+, "]") => ns),
        Ok(ref ns) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("[0 1 2 3]"; ("[", [ let _: i32 ]*, "]") => ()),
        Ok(())
    );

    assert_match!(
        scan!("[0 1 2 3]"; ("[", [ let ns: i32 ]*, "]") => ns),
        Ok(ref ns) if *ns == vec![0, 1, 2, 3]
    );

    assert_match!(
        scan!("[0 1 2 3]"; ("[", [ let xs: i32, let ys: i32 ]*, "]") => (xs, ys)),
        Ok((ref xs, ref ys)) if &**xs == [0, 2] && &**ys == [1, 3]
    );

    assert_match!(
        scan!("[[0 1] [2 3]]"; ("[", ["[", [let nss: i32]*, "]"]*, "]") => nss),
        Ok(ref ns) if *ns == vec![vec![0, 1], vec![2, 3]]
    );

    assert_match!(
        scan!("[0 [1 2] 3 [4 5]]"; ("[", [let xs: i32, "[", [let yss: i32]*, "]"]*, "]") => (xs, yss)),
        Ok((ref xs, ref yss)) if *xs == vec![0, 3] && *yss == vec![vec![1, 2], vec![4, 5]]
    );

    assert_match!(
        scan!("0"; ([ let ns: i32 ]{2}, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 1
    );

    assert_match!(
        scan!("0 1"; ([ let ns: i32 ]{2}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1 2"; ([ let ns: i32 ]{2}, ..tail) => (ns, tail)),
        Ok((ref ns, " 2")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0"; ([ let ns: i32 ]{2,}, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 1
    );

    assert_match!(
        scan!("0 1"; ([ let ns: i32 ]{2,}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1 2"; ([ let ns: i32 ]{2,}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0"; ([ let ns: i32 ]{2, 3}, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 1
    );

    assert_match!(
        scan!("0 1"; ([ let ns: i32 ]{2, 3}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1 2"; ([ let ns: i32 ]{2, 3}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0 1 2 3"; ([ let ns: i32 ]{2, 3}, ..tail) => (ns, tail)),
        Ok((ref ns, " 3")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0"; ([ let ns: i32 ]{,3}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0]
    );

    assert_match!(
        scan!("0 1"; ([ let ns: i32 ]{,3}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1 2"; ([ let ns: i32 ]{,3}, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0 1 2 3"; ([ let ns: i32 ]{,3}, ..tail) => (ns, tail)),
        Ok((ref ns, " 3")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!(""; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![]
    );

    assert_match!(
        scan!("0"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0]
    );

    assert_match!(
        scan!("0,"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 2
    );

    assert_match!(
        scan!("0, 1, 2, 3"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1, 2, 3]
    );

    assert_match!(
        scan!("0, 1, 2 3"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, " 3")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0, 1 2, 3"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, " 2, 3")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1, 2, 3"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Ok((ref ns, " 1, 2, 3")) if *ns == vec![0]
    );

    assert_match!(
        scan!("0, 1, 2, 3,"; ([ let ns: i32 ],*, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 11
    );

    assert_match!(
        scan!("0 and 1 and 2 and 3"; ([ let ns: i32 ]("and")*, ..tail) => (ns, tail)),
        Ok((ref ns, "")) if *ns == vec![0, 1, 2, 3]
    );

    assert_match!(
        scan!("0 and 1 and 2 3"; ([ let ns: i32 ]("and")*, ..tail) => (ns, tail)),
        Ok((ref ns, " 3")) if *ns == vec![0, 1, 2]
    );

    assert_match!(
        scan!("0 and 1 2 and 3"; ([ let ns: i32 ]("and")*, ..tail) => (ns, tail)),
        Ok((ref ns, " 2 and 3")) if *ns == vec![0, 1]
    );

    assert_match!(
        scan!("0 1 and 2 and 3"; ([ let ns: i32 ]("and")*, ..tail) => (ns, tail)),
        Ok((ref ns, " 1 and 2 and 3")) if *ns == vec![0]
    );

    assert_match!(
        scan!("0 and 1 and 2 and 3 and"; ([ let ns: i32 ]("and")*, ..tail) => (ns, tail)),
        Err(SE { ref at, kind: SEK::Missing }) if at.offset() == 23
    );

    assert_match!(
        scan!("0 and 1 and 2 and 3"; ([ let ns: i32 ]( let sep: &str )*, ..tail) => (ns, sep, tail)),
        Ok((ref ns, ref sep, "")) if *ns == vec![0, 1, 2, 3] && *sep == vec!["and", "and", "and"]
    );
}

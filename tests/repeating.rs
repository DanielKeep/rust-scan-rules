#[macro_use] extern crate quickscan;
#[macro_use] mod util;

#[test]
fn test_repeating() {
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
}

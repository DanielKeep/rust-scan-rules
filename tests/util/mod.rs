macro_rules! assert_match {
    ($e:expr, $p:pat) => {
        match $e {
            $p => (),
            e => panic!("assertion failed: `(left match right)` (left: `{:?}`, right: `{:?}`)",
                e, stringify!($p))
        }
    };
}

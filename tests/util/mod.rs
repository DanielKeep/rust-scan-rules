/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
macro_rules! assert_match {
    ($e:expr, $p:pat) => {
        match $e {
            $p => (),
            e => panic!("assertion failed: `(left match right)` (left: `{:?}`, right: `{:?}`)",
                e, stringify!($p))
        }
    };

    ($e:expr, $p:pat if $cond:expr) => {
        match $e {
            $p if $cond => (),
            e => panic!("assertion failed: `(left match right)` (left: `{:?}`, right: `{:?}`)",
                e, stringify!($p if $cond))
        }
    };
}

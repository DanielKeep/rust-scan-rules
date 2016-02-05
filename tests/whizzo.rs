/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This file is for assorted integration tests that don't really belong anywhere
else and don't deserve an entire file.
*/
#[macro_use] extern crate scan_rules;
#[macro_use] mod util;

#[test]
fn test_float_scan_forms() {
    // Double-plus assurance that "2" really *does* scan as a float.
    assert_match!(scan!("2"; (let f: f32) => f), Ok(2.0f32));
    assert_match!(scan!("2."; (let f: f32) => f), Ok(2.0f32));
    assert_match!(scan!("2.0"; (let f: f32) => f), Ok(2.0f32));
    assert_match!(scan!("2.0e0"; (let f: f32) => f), Ok(2.0f32));
    assert_match!(scan!("2.e0"; (let f: f32) => f), Ok(2.0f32));
    assert_match!(scan!("2e0"; (let f: f32) => f), Ok(2.0f32));
}

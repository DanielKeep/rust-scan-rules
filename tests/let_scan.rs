/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#[macro_use] extern crate scan_rules;
use scan_rules::scanner::Word;

#[test]
fn test_let_scan() {
    let input = "10¥, うまい棒";
    let_scan!(input; (let cost: u32, "¥,", let product: Word));
    assert_eq!(cost, 10);
    assert_eq!(product, "うまい棒");
}

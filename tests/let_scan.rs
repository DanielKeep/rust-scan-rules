#[macro_use] extern crate scan_rules;
use scan_rules::scanner::Word;

#[test]
fn test_let_scan() {
    let input = "10¥, うまい棒";
    let_scan!(input; (let cost: u32, "¥,", let product: Word));
    assert_eq!(cost, 10);
    assert_eq!(product, "うまい棒");
}

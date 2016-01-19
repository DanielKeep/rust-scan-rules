#[macro_use] extern crate lazy_static;
extern crate itertools;
extern crate regex;

#[macro_use] mod macros;

pub use error::{ScanError, ScanErrorKind};
pub use input::{Cursor, ScanInput};
pub use scanner::{ScanFromStr, ScanSelfFromStr, Word};

mod error;
mod input;
mod scanner;

#[doc(hidden)]
pub fn strip_line_term(s: &str) -> &str {
    if s.ends_with("\r\n") {
        &s[0..s.len()-2]
    } else if s.ends_with("\n") {
        &s[0..s.len()-1]
    } else if s.ends_with("\r") {
        &s[0..s.len()-1]
    } else {
        s
    }
}

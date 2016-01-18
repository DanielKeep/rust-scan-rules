#[macro_use] extern crate lazy_static;
extern crate itertools;
extern crate regex;

#[macro_use] mod macros;

pub use error::{ScanError, ScanErrorKind};
pub use input::ScanInput;
pub use scanner::{ScanFromStr, ScanSelfFromStr, Word};

mod error;
mod input;
mod scanner;

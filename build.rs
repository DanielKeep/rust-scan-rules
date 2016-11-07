/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
extern crate rustc_version;
use rustc_version::{version_matches};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    /*
    See <https://github.com/rust-lang/rust/issues/26448#issuecomment-173794570>.
    */
    if version_matches("< 1.7.0") {
        println!("cargo:rustc-cfg=str_into_output_extra_broken");
    }

    if version_matches("1.10.0") {
        println!("cargo:rustc-cfg=macro_inter_stmt_binding_visibility");
    }
}

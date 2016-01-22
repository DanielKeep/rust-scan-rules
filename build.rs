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
}

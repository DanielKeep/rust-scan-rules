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

    if version_matches("1.4.0") {
        /*
        `AddrParseError` doesn't implement `Error` < 1.4.  It's easier to just not implement the scanners for the `std::net` module than futzing with translating errors differently.
        */
        println!("cargo:rustc-cfg=std_net_scanners");
        println!("cargo:rustc-cfg=binary_heap_impls_debug");
        println!("cargo:rustc-cfg=f64_debug_is_roundtrip_accurate");
    }

    if version_matches("1.5.0") {
        println!("cargo:rustc-cfg=socket_addr_vx_scanners")
    }
}

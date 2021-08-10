extern crate bindgen;

use std::path::PathBuf;

//#[cfg(linux)]
fn main() {
    println!("cargo:rustc-link-lib=dylib=okFrontPanel");
    let root_path = PathBuf::from("/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/API");
    println!(
        "cargo:rustc-link-search=native={}",
        root_path.to_str().unwrap()
    );
}

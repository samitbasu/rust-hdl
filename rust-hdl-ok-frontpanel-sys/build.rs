extern crate bindgen;

use std::path::PathBuf;


fn main() {
    println!("cargo:rustc-link-lib=dylib=okFrontPanel");
    let root_path = PathBuf::from(
        std::env::var("OK_FRONTPANEL_DIR")
            .expect("Set OK_FRONTPANEL_DIR to absolute path of the FrontPanel/API directory on this system"));
    println!("cargo:rustc-link-lib=dylib=okFrontPanel");
    println!(
        "cargo:rustc-link-search=native={}",
        root_path.to_str().unwrap()
    );

    let bindings = bindgen::Builder::default()
        .header(root_path.join("okFrontPanel.h").to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

use crate::synth::filter_blackbox_directives;
use crate::ucf_gen::generate_ucf;
use crate::xdc_gen::generate_xdc;
use rust_hdl_core::prelude::*;
use std::fs::{copy, create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn generate_bitstream_xem_7010<U: Block>(mut uut: U, prefix: &str, assets: &[String]) {
    uut.connect_all();
    check_connected(&uut);
    let verilog_text = filter_blackbox_directives(&generate_verilog(&uut));
    let xdc_text = generate_xdc(&uut);
    let dir = PathBuf::from(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut assets: Vec<String> = assets.into();
    std::fs::write(dir.clone().join("top.v"), verilog_text).unwrap();
    std::fs::write(dir.clone().join("top.xdc"), xdc_text).unwrap();
    for asset in &assets {
        let src = PathBuf::from(asset);
        let dest = dir.clone().join(src.file_name().unwrap());
        println!("Copy from {:?} -> {:?}", asset, dest);
        copy(asset, dest).unwrap();
    }
    let mut tcl_file = File::create(dir.clone().join("top.tcl")).unwrap();
    write!(
        tcl_file,
        "\
project new top.xise
project set family Spartan6
project set device xc6slx45
project set package fgg484
project set speed -2
xfile add top.v top.ucf {assets}
project set top top
process run \"Generate Programming File\" -force rerun_all
project close
",
        assets = assets
            .iter()
            .map(|x| PathBuf::from(x)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string())
            .collect::<Vec<_>>()
            .join(" ")
    )
    .unwrap();
}

use rust_hdl_core::prelude::{Block, generate_verilog};
use rust_hdl_core::check_connected::check_connected;
use rust_hdl_synth::yosys_validate;
use crate::pcf_gen::generate_pcf;
use std::path::PathBuf;
use std::str::FromStr;
use std::fs::{remove_dir_all, create_dir, File};
use std::process::{Command, Output};
use std::io::Write;

fn save_stdout(output: Output, dir: &PathBuf, basename: &str) -> Result<(), std::io::Error> {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join(format!("{}.out", basename)))?;
    write!(out_file, "{}", stdout)?;
    let mut err_file = File::create(dir.clone().join(format!("{}.err", basename)))?;
    write!(err_file, "{}", stderr)?;
    Ok(())
}

pub fn generate_bitstream<U: Block>(mut uut: U, prefix: &str) {
    check_connected(&uut);
    let verilog_text = generate_verilog(&uut);
    yosys_validate(prefix, &verilog_text).unwrap();
    let pcf_text = generate_pcf(&uut);
    let dir = PathBuf::from_str(prefix).unwrap();
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut v_file = File::create(dir.clone().join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();
    let pcf_filename = format!("top.pcf");
    let mut pcf_file = File::create(dir.clone().join(&pcf_filename)).unwrap();
    write!(pcf_file, "{}", pcf_text).unwrap();
    let output = Command::new("yosys")
        .current_dir(dir.clone())
        .arg(r#"-p synth_ice40 -top top -blif top.blif"#)
        .arg("top.v")
        .output().unwrap();
    save_stdout(output, &dir, "yosys_synth").unwrap();
    let output = Command::new("arachne-pnr")
        .current_dir(dir.clone())
        .args(&[
            "-r", "-d", "8k", "-P", "cb132", "-p", "top.pcf", "-o", "top.txt", "top.blif",
        ])
        .output().unwrap();
    save_stdout(output, &dir, "arachne");
    let output = Command::new("icepack")
        .current_dir(dir.clone())
        .args(&["top.txt", "top.bin"])
        .output().unwrap();
    save_stdout(output, &dir, "icepack");
}
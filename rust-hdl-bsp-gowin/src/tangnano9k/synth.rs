use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    process::{Command, Output},
    str::FromStr,
};

use rust_hdl::core::{check_error::check_all, prelude::*};
use rust_hdl::fpga::toolchains::gowin::generate_cst;

fn save_stdout(output: Output, dir: &PathBuf, basename: &str) -> Result<(), std::io::Error> {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join(format!("{}.out", basename)))?;
    out_file.write_all(stdout.as_bytes())?;
    let mut err_file = File::create(dir.clone().join(format!("{}.err", basename)))?;
    err_file.write_all(stderr.as_bytes())?;
    Ok(())
}

pub fn generate_bitstream<U: Block>(mut uut: U, download: bool, prefix: &str, yosys: &str) {
    uut.connect_all();
    check_all(&uut).unwrap();

    let verilog_text = generate_verilog(&uut);
    let cst_text = generate_cst(&uut);
    let dir = PathBuf::from_str(prefix).unwrap();

    create_dir_all(&dir).unwrap();

    let mut v_file = File::create(dir.join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();
    let mut cst_file = File::create(dir.join("top.cst")).unwrap();
    write!(cst_file, "{}", cst_text).unwrap();

    let output = Command::new(format!("{yosys}/yosys"))
        .current_dir(dir.clone())
        .arg("-p")
        .arg("read_verilog top.v; synth_gowin -json top.json")
        .output()
        .unwrap();
    save_stdout(output, &dir, "yosys").unwrap();

    let output = Command::new(format!("{yosys}/nextpnr-himbaechel"))
        .current_dir(dir.clone())
        .args([
            "--json",
            "top.json",
            "--write",
            "top.pnr.json",
            "--device",
            "GW1NR-LV9QN88PC6/I5",
            "--vopt",
            "family=GW1N-9C",
            "--vopt",
            "cst=top.cst",
        ])
        .output()
        .unwrap();
    save_stdout(output, &dir, "nextpnr").unwrap();

    let output = Command::new(format!("{yosys}/gowin_pack"))
        .current_dir(dir.clone())
        .args(["-d", "GW1N-9C", "-o", "top.fs", "top.pnr.json"])
        .output()
        .unwrap();
    save_stdout(output, &dir, "pack").unwrap();

    if download {
        let output = Command::new(format!("{yosys}/openFPGALoader"))
            .current_dir(dir.clone())
            .args(["-b", "tangnano9k", "-f", "top.fs"])
            .output()
            .unwrap();
        save_stdout(output, &dir, "loader").unwrap();
    }
}

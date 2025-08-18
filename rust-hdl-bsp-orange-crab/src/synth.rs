use rust_hdl::core::check_error::check_all;
use rust_hdl::core::prelude::*;
use rust_hdl::fpga::toolchains::ecp5::generate_lpf;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::str::FromStr;

/// Represents the field density of the Orange Crab FPGA.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Density {
    /// Represents the 25F variant.
    _25F,
    /// Represents the 85F variant.
    _85F,
}

impl Density {
    /// Creates a new [Density].
    pub const fn new() -> Self {
        Self::_25F
    }

    /// Gets the [Density] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::_25F => "25F",
            Self::_85F => "85F",
        }
    }

    /// Gets the [Density] argument string representation.
    pub const fn as_arg(&self) -> &'static str {
        match self {
            Self::_25F => "--25k",
            Self::_85F => "--85k",
        }
    }
}

impl Default for Density {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn save_stdout(output: Output, dir: &PathBuf, basename: &str) -> Result<(), std::io::Error> {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join(format!("{}.out", basename)))?;
    write!(out_file, "{}", stdout)?;
    let mut err_file = File::create(dir.clone().join(format!("{}.err", basename)))?;
    write!(err_file, "{}", stderr)?;
    Ok(())
}

/// Generates the bitstream from the RustHDL definition.
pub fn generate_bitstream<U: Block>(mut uut: U, prefix: &str, density: Density) {
    uut.connect_all();
    check_all(&uut).unwrap(); // TODO - Change from panic to return an error
    let verilog_text = generate_verilog(&uut);
    let pcf_text = generate_lpf(&uut);

    let dir = PathBuf::from_str(prefix).unwrap();
    let _ = remove_dir_all(&dir);
    let _ = create_dir_all(&dir);

    let mut v_file = File::create(dir.join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();

    let pcf_filename = "top.pcf".to_string();
    let mut pcf_file = File::create(dir.join(pcf_filename)).unwrap();
    write!(pcf_file, "{}", pcf_text).unwrap();

    let output = Command::new("yosys")
        .current_dir(dir.clone())
        .args(["-p", "read_verilog top.v; synth_ecp5 -json top.json"])
        .output()
        .unwrap();
    save_stdout(output, &dir, "yosys_synth").unwrap();

    let output = Command::new("nextpnr-ecp5")
        .current_dir(dir.clone())
        .args([
            "--package",
            "CSFBGA285",
            density.as_arg(),
            "--textcfg",
            "top.asc",
            "--lpf",
            "top.pcf",
            "--json",
            "top.json",
        ])
        .output()
        .unwrap();
    save_stdout(output, &dir, "nextpnr").unwrap();

    let output = Command::new("ecppack")
        .current_dir(dir.clone())
        .args([
            "--compress",
            "--freq",
            "38.8",
            "--input",
            "top.asc",
            "--bit",
            "top.bin",
        ])
        .output()
        .unwrap();
    save_stdout(output, &dir, "ecppack").unwrap();

    let output = Command::new("dfu-suffix")
        .current_dir(dir.clone())
        .args(["-v", "1209", "-p", "5af0", "-a", "top.bin"])
        .output()
        .unwrap();
    save_stdout(output, &dir, "ecppack").unwrap();
}

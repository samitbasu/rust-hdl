use crate::synth_common::filter_blackbox_directives;
use crate::xdc_gen::generate_xdc;
use rust_hdl_core::prelude::*;
use std::fs::{copy, create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct VivadoOptions {
    pub vivado_path: String,
    pub add_mig: bool,
    pub assets: Vec<String>,
}

const VIVADO_PATH: &str = "/opt/Xilinx/Vivado/2018.1/bin/";
const FP_PATH: &str = "/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/FrontPanelHDL/XEM7010-A50";

impl Default for VivadoOptions {
    fn default() -> Self {
        Self {
            vivado_path: VIVADO_PATH.into(),
            add_mig: true,
            assets: [
                "okLibrary.v",
                "okCoreHarness.v",
                "okWireIn.v",
                "okWireOut.v",
                "okTriggerIn.v",
                "okTriggerOut.v",
                "okPipeIn.v",
                "okPipeOut.v",
                "okBTPipeIn.v",
                "okBTPipeOut.v",
            ]
            .iter()
            .map(|x| format!("{}/{}", FP_PATH, x))
            .collect(),
        }
    }
}

pub fn generate_bitstream_xem_7010<U: Block>(mut uut: U, prefix: &str, options: VivadoOptions) {
    uut.connect_all();
    check_connected(&uut);
    let verilog_text = filter_blackbox_directives(&generate_verilog(&uut));
    let xdc_text = generate_xdc(&uut);
    let dir = PathBuf::from(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let assets: Vec<String> = options.assets.clone();
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
        r#"
create_project top . -part xc7a50tfgg484-1 -force

add_files {{top.v top.xdc {assets} }}

update_compile_order

launch_runs synth_1 -jobs 8
wait_on_run synth_1

set status [ get_property STATUS [ get_runs synth_1 ] ]
if {{ $status != "synth_design Complete!" }} {{
 puts "Synthesis Failed"
 exit
}}

launch_runs impl_1 -to_step write_bitstream -jobs 8
wait_on_run impl_1

set status [ get_property STATUS [ get_runs impl_1 ] ]
if {{ $status != "write_bitstream Complete!" }} {{
 puts "Implementation Failed"
 exit
}}

puts "Vivado Run Complete"
exit
"#,
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
    let output = Command::new(format!("{}/vivado", options.vivado_path))
        .current_dir(dir.clone())
        .arg("-mode")
        .arg("tcl")
        .arg("-source")
        .arg("top.tcl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    std::fs::write(dir.clone().join("top.out"), &stdout).unwrap();
    std::fs::write(dir.clone().join("top.err"), &stderr).unwrap();
    assert!(stdout.contains("Vivado Run Complete"));
    copy(
        dir.clone().join("top.runs/impl_1/top.bit"),
        dir.clone().join("top.bit"),
    )
    .unwrap();
}

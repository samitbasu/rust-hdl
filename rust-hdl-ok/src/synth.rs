use crate::ucf_gen::generate_ucf;
use rust_hdl_core::prelude::*;
use std::fs::{copy, create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn filter_blackbox_directives(t: &str) -> String {
    let mut in_black_box = false;
    let mut ret = vec![];
    for line in t.split("\n") {
        in_black_box = in_black_box || line.starts_with("(* blackbox *)");
        if !in_black_box {
            ret.push(line);
        }
        if line.starts_with("endmodule") {
            in_black_box = false;
        }
    }
    ret.join("\n")
}

#[test]
fn test_filter_bb_directives() {
    let p = r#"
blah
more code
goes here

(* blackbox *)
module my_famous_module(
    super_secret_arg1,
    super_secret_arg2,
    super_secret_arg3);
/* Comment */
endmodule

stuff
"#;
    let q = filter_blackbox_directives(p);
    println!("{}", q);
    assert!(!q.contains("blackbox"));
    assert!(!q.contains("module"));
    assert!(!q.contains("endmodule"));
    assert!(q.contains("more code"));
    assert!(q.contains("stuff"));
}

pub fn generate_bitstream_xem_6010<U: Block>(mut uut: U, prefix: &str, assets: &[String]) {
    uut.connect_all();
    check_connected(&uut);
    let verilog_text = filter_blackbox_directives(&generate_verilog(&uut));
    let ucf_text = generate_ucf(&uut);
    let dir = PathBuf::from(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut assets: Vec<String> = assets.into();
    assets.extend_from_slice(&add_mig_core_xem_6010(prefix));
    let mut v_file = File::create(dir.clone().join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();
    let mut ucf_file = File::create(dir.clone().join("top.ucf")).unwrap();
    write!(ucf_file, "{}", ucf_text).unwrap();
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
    let output = Command::new("xtclsh")
        .current_dir(dir.clone())
        .arg("top.tcl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join("top.out")).unwrap();
    write!(out_file, "{}", stdout).unwrap();
    let mut err_file = File::create(dir.clone().join("top.err")).unwrap();
    write!(err_file, "{}", stderr).unwrap();
    assert!(stdout.contains(r#"Process "Generate Programming File" completed successfully"#));
    assert!(stdout.contains(r#"All constraints were met."#));
}

pub fn add_mig_core_xem_6010(prefix: &str) -> Vec<String> {
    let dir = PathBuf::from(prefix).join("core_gen");
    let _ = create_dir(&dir);
    let mut prj_file = File::create(dir.clone().join("mig.prj")).unwrap();
    write!(prj_file, r#"<?xml version="1.0" encoding="UTF-8"?>
<Project NoOfControllers="1" >
    <ModuleName>mig</ModuleName>
    <TargetFPGA>xc6slx45-fgg484/-2</TargetFPGA>
    <Version>3.92</Version>
    <Controller number="3" >
        <MemoryDevice>DDR2_SDRAM/Components/MT47H64M16XX-3</MemoryDevice>
        <TimePeriod>3200</TimePeriod>
        <EnableVoltageRange>0</EnableVoltageRange>
        <DataMask>1</DataMask>
        <CustomPart>FALSE</CustomPart>
        <NewPartName></NewPartName>
        <RowAddress>13</RowAddress>
        <ColAddress>10</ColAddress>
        <BankAddress>3</BankAddress>
        <TimingParameters>
            <Parameters twtr="7.5" trefi="7.8" twr="15" trtp="7.5" trfc="127.5" trp="15" tras="40" trcd="15" />
        </TimingParameters>
        <mrBurstLength name="Burst Length" >4(010)</mrBurstLength>
        <mrCasLatency name="CAS Latency" >5</mrCasLatency>
        <emrDllEnable name="DLL Enable" >Enable-Normal</emrDllEnable>
        <emrOutputDriveStrength name="Output Drive Strength" >Reducedstrength</emrOutputDriveStrength>
        <emrRTT name="RTT (nominal) - ODT" >50ohms</emrRTT>
        <emrPosted name="Additive Latency (AL)" >0</emrPosted>
        <emrOCD name="OCD Operation" >OCD Exit</emrOCD>
        <emrDQS name="DQS# Enable" >Enable</emrDQS>
        <emrRDQS name="RDQS Enable" >Disable</emrRDQS>
        <emrOutputs name="Outputs" >Enable</emrOutputs>
        <mr2SelfRefreshTempRange name="High Temparature Self Refresh Rate" >Disable</mr2SelfRefreshTempRange>
        <PortInterface>NATIVE,NATIVE,NATIVE,NATIVE,NATIVE,NATIVE</PortInterface>
        <Class>Class II</Class>
        <DataClass>Class II</DataClass>
        <InputPinTermination>CALIB_TERM</InputPinTermination>
        <DataTermination>25 Ohms</DataTermination>
        <CalibrationRowAddress></CalibrationRowAddress>
        <CalibrationColumnAddress></CalibrationColumnAddress>
        <CalibrationBankAddress></CalibrationBankAddress>
        <SystemClock>Single-Ended</SystemClock>
        <BypassCalibration>1</BypassCalibration>
        <DebugSignals>Disable</DebugSignals>
        <SystemClock>Single-Ended</SystemClock>
        <Configuration>Two 32-bit bi-directional and four 32-bit unidirectional ports</Configuration>
        <RzqPin>K7</RzqPin>
        <ZioPin>Y2</ZioPin>
        <PortsSelected>Port0</PortsSelected>
        <PortDirections>Bi-directional,none,none,none,none,none</PortDirections>
        <UserMemoryAddressMap>ROW_BANK_COLUMN</UserMemoryAddressMap>
        <ArbitrationAlgorithm>Round Robin</ArbitrationAlgorithm>
        <TimeSlot0>0</TimeSlot0>
        <TimeSlot1>0</TimeSlot1>
        <TimeSlot2>0</TimeSlot2>
        <TimeSlot3>0</TimeSlot3>
        <TimeSlot4>0</TimeSlot4>
        <TimeSlot5>0</TimeSlot5>
        <TimeSlot6>0</TimeSlot6>
        <TimeSlot7>0</TimeSlot7>
        <TimeSlot8>0</TimeSlot8>
        <TimeSlot9>0</TimeSlot9>
        <TimeSlot10>0</TimeSlot10>
        <TimeSlot11>0</TimeSlot11>
    </Controller>
</Project>
"#).unwrap();
    let mut xco_file = File::create(dir.clone().join("coregen.xco")).unwrap();
    write!(
        xco_file,
        r#"
NEWPROJECT coregen.cgc
SET workingdirectory="."
##############################################################
#
#  Generated from component: xilinx.com:ip:mig:3.92
#
##############################################################
#
# BEGIN Project Options
SET addpads = false
SET asysymbol = false
SET busformat = BusFormatAngleBracketNotRipped
SET createndf = false
SET designentry = Verilog
SET device = xc6slx45
SET devicefamily = spartan6
SET flowvendor = Other
SET formalverification = false
SET foundationsym = false
SET implementationfiletype = Ngc
SET package = fgg484
SET removerpms = false
SET simulationfiles = None
SET speedgrade = -2
SET verilogsim = false
SET vhdlsim = false
# END Project Options
# BEGIN Select
SELECT MIG_Virtex-6_and_Spartan-6 family Xilinx,_Inc. 3.92
# END Select
# BEGIN Parameters
CSET component_name=mig
CSET xml_input_file=mig.prj
# END Parameters
# BEGIN Extra information
MISC pkg_timestamp=2013-10-13T18:46:09Z
# END Extra information
GENERATE
# CRC: d7d2c6fc
"#
    )
    .unwrap();
    let _output = Command::new("coregen")
        .current_dir(dir.clone())
        .arg("-b")
        .arg("coregen.xco")
        .output()
        .unwrap();
    // Patch the generated MIG file to fix the clocking
    let mig_source =
        std::fs::read_to_string(dir.clone().join("mig/user_design/rtl/mig.v")).unwrap();
    let mig_source = mig_source.replace(
        "localparam C3_CLKOUT2_DIVIDE       = 16;",
        "localparam C3_CLKOUT2_DIVIDE       = 6; // Patched by Rust-HDL",
    );
    let mig_source = mig_source.replace(
        "localparam C3_CLKFBOUT_MULT        = 2;",
        "localparam C3_CLKFBOUT_MULT        = 6;  // Patched by Rust-HDL",
    );
    let mig_source = mig_source.replace(
        "localparam C3_INCLK_PERIOD         = ((C3_MEMCLK_PERIOD * C3_CLKFBOUT_MULT) / (C3_DIVCLK_DIVIDE * C3_CLKOUT0_DIVIDE * 2));",
        "localparam C3_INCLK_PERIOD          = 10000; // Assumes 100MHz system clock Patched by Rust-HDL"
    );
    std::fs::write(
        dir.clone().join("mig/user_design/rtl/mig_patched.v"),
        mig_source,
    )
    .unwrap();
    [
        "mig/user_design/rtl/infrastructure.v",
        "mig/user_design/rtl/mcb_controller/iodrp_controller.v",
        "mig/user_design/rtl/mcb_controller/iodrp_mcb_controller.v",
        "mig/user_design/rtl/mcb_controller/mcb_raw_wrapper.v",
        "mig/user_design/rtl/mcb_controller/mcb_soft_calibration.v",
        "mig/user_design/rtl/mcb_controller/mcb_soft_calibration_top.v",
        "mig/user_design/rtl/mcb_controller/mcb_ui_top.v",
        "mig/user_design/rtl/memc_wrapper.v",
        "mig/user_design/rtl/mig_patched.v",
    ]
    .iter()
    .map(|p| dir.clone().join(p).to_string_lossy().to_string())
    .collect()
}

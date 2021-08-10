use rust_hdl_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::{OkHandle, OkError};
use std::path::Path;

const FRONTPANEL_DIR: &str = "/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/FrontPanelHDL/XEM6010-LX45";

#[cfg(test)]
pub fn synth_obj<U: Block>(uut: U, dir: &str) {
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    let ucf = rust_hdl_ok::ucf_gen::generate_ucf(&uut);
    println!("{}", ucf);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
    rust_hdl_ok::synth::generate_bitstream_xem_6010(uut, dir, &[
        "okLibrary.v",
        "okCoreHarness.ngc",
        "okWireIn.ngc",
        "TFIFO64x8a_64x8b.ngc",
        "okWireOut.ngc"
    ], FRONTPANEL_DIR);
}

pub fn ok_test_prelude(filename: &str) -> Result<OkHandle, OkError> {
    let hnd = OkHandle::new();
    hnd.open()?;
    hnd.reset_fpga()?;
    let model = hnd.get_board_model();
    let serial = hnd.get_serial_number();
    let vers = hnd.get_firmware_version();
    let dev_id = hnd.get_device_id();
    println!("Board model found {}, serial {}", model, serial);
    println!("Firmware version {:?}", vers);
    println!("Device ID {}", dev_id);
    let (major, minor, micro) = hnd.get_api_version();
    println!("API Version {} {} {}", major, minor, micro);
    let firmware = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(filename);
    println!("Firmware path for bitfile {}", firmware.to_str().unwrap());
    hnd.configure_fpga(firmware.to_str().unwrap())?;
    Ok(hnd)
}
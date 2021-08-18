use rust_hdl_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::{OkError, OkHandle};
use std::path::Path;

const FRONTPANEL_DIR: &str = "/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/FrontPanelHDL/XEM6010-LX45";
const MIG_DIR: &str = "/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/Samples/RAMTester/XEM6010-Verilog";

#[cfg(test)]
pub fn synth_obj<U: Block>(uut: U, dir: &str) {
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    let ucf = rust_hdl_ok::ucf_gen::generate_ucf(&uut);
    println!("{}", ucf);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
    let mut frontpanel_hdl = [
        "okLibrary.v",
        "okCoreHarness.ngc",
        "okWireIn.ngc",
        "TFIFO64x8a_64x8b.ngc",
        "okWireOut.ngc",
        "okTriggerIn.ngc",
        "okTriggerOut.ngc",
        "okPipeIn.ngc",
        "okPipeOut.ngc",
    ]
    .iter()
    .map(|x| format!("{}/{}", FRONTPANEL_DIR, x))
    .collect::<Vec<_>>();
    let mut mig_hdl = [
        "ddr2.v",
        "memc3_infrastructure.v",
        "MIG/memc3_wrapper.v",
        "MIG/iodrp_controller.v",
        "MIG/iodrp_mcb_controller.v",
        "MIG/mcb_raw_wrapper.v",
        "MIG/mcb_soft_calibration.v",
        "MIG/mcb_soft_calibration_top.v",
    ]
    .iter()
    .map(|x| format!("{}/{}", MIG_DIR, x))
    .collect::<Vec<_>>();
    frontpanel_hdl.append(&mut mig_hdl);
    rust_hdl_ok::synth::generate_bitstream_xem_6010(uut, dir, &frontpanel_hdl);
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
    let firmware = Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
    println!("Firmware path for bitfile {}", firmware.to_str().unwrap());
    hnd.configure_fpga(firmware.to_str().unwrap())?;
    Ok(hnd)
}

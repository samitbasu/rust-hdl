use crate::ad7193_sim::AD7193_REG_WIDTHS;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok::spi::OKSPIMasterAddressConfig;
use rust_hdl_ok_frontpanel_sys::{OkError, OkHandle};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub fn find_ok_bus_collisions(vlog: &str) {
    let expr = regex::Regex::new(r#"\.ep_addr\(8'h(\w+)\)"#).unwrap();
    let mut addr_list = vec![];
    for capture in expr.captures_iter(vlog) {
        let port = capture.get(1).unwrap().as_str();
        assert!(
            !addr_list.contains(&port.to_string()),
            "Found duplicate port! {}",
            port
        );
        addr_list.push(port.to_owned());
    }
}

pub fn synth_obj_7010<U: Block>(uut: U, dir: &str) {
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    find_ok_bus_collisions(&vlog);
    let _xcd = rust_hdl_ok::xdc_gen::generate_xdc(&uut);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
    generate_bitstream_xem_7010(uut, dir, Default::default());
}

pub fn synth_obj_6010<U: Block>(uut: U, dir: &str) {
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    find_ok_bus_collisions(&vlog);
    let _ucf = rust_hdl_ok::ucf_gen::generate_ucf(&uut);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
    generate_bitstream_xem_6010(uut, dir, Default::default());
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

pub fn mk_64bit_spi_order(data: u64) -> [u8; 8] {
    let mut buf = [0_u8; 8];
    let mut val = data;
    for i in 0..8 {
        buf[(7 - i) ^ 1] = (val & 0xFF) as u8;
        val = val >> 8;
    }
    buf
}

pub fn mk_spi_order_64bit(buf: &[u8; 8]) -> u64 {
    let mut val = 0_u64;
    for i in 0..8 {
        val = (val << 8) | (buf[(i ^ 1)] & 0xFF) as u64;
    }
    val
}

pub fn ok_do_spi_txn(
    handle: &OkHandle,
    bits: u16,
    outgoing: u64,
    continued: bool,
) -> Result<u64, OkError> {
    let addr = OKSPIMasterAddressConfig::default();
    let out_buf = mk_64bit_spi_order(outgoing);
    println!("Out buf for SPI TXN: {:x?}", out_buf);
    handle.write_to_pipe_in(addr.pipe_in_address as i32, &out_buf)?;
    handle.set_wire_in(addr.wire_bits_address as i32, bits);
    handle.update_wire_ins();
    if !continued {
        handle.activate_trigger_in(addr.trigger_start_address as i32, 0)?;
    } else {
        handle.activate_trigger_in(addr.trigger_start_address as i32, 1)?;
    }
    println!("Trigger SPI txn");
    let mut ok = false;
    for _ in 0..5 {
        handle.update_trigger_outs();
        if handle.is_triggered(addr.trigger_done_address as i32, 1) {
            ok = true;
            break;
        }
        sleep(Duration::from_millis(100));
    }
    println!("SPI Txn done");
    if !ok {
        return Err(OkError { code: -1 });
    }
    println!("Reading result");
    let mut in_buf = [0_u8; 8];
    handle.read_from_pipe_out(addr.pipe_out_address as i32, &mut in_buf)?;
    Ok(mk_spi_order_64bit(&in_buf))
}

pub fn ok_reg_read(handle: &OkHandle, reg_index: u32) -> Result<u64, OkError> {
    let cmd = (((1 << 6) | (reg_index << 3)) << 24) as u64;
    let result = ok_do_spi_txn(handle, 32, cmd, false)?;
    println!("Reg Read result: {:x}", result);
    let width = AD7193_REG_WIDTHS[reg_index as usize];
    if width == 8 {
        Ok((result >> 16) & 0xFF)
    } else {
        Ok(result & 0xFFFFFF)
    }
}

pub fn ok_reg_write(handle: &OkHandle, reg_index: u32, reg_value: u64) -> Result<u64, OkError> {
    let mut cmd = (((0 << 6) | (reg_index << 3)) << 24) as u64;
    if AD7193_REG_WIDTHS[reg_index as usize] == 8 {
        cmd = cmd | reg_value << 16;
    } else {
        cmd = cmd | reg_value;
    }
    ok_do_spi_txn(handle, 32, cmd, false)
}

pub fn ok_adc_reset(handle: &OkHandle) -> Result<(), OkError> {
    ok_do_spi_txn(handle, 64, 0xFFFF_FFFF_FFFF_FFFF_u64, false)?;
    Ok(())
}

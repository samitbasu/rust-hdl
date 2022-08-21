use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

use {
    crate::test_common::tools::ok_test_prelude,
    rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError, OkHandle},
    std::thread::sleep,
};

pub fn mk_u8(dat: &[u16]) -> Vec<u8> {
    let mut ret = vec![0_u8; dat.len() * 2];
    for (ndx, el) in dat.iter().enumerate() {
        ret[2 * ndx] = (el & 0xFF) as u8;
        ret[2 * ndx + 1] = ((el & 0xFF00) >> 8) as u8;
    }
    ret
}

fn send_ping(hnd: &OkHandle, id: u8) -> Result<(), OkError> {
    hnd.write_to_pipe_in(0x80, &mk_u8(&[0x0100 | (id as u16)]))
}

fn read_ping(hnd: &OkHandle) -> Result<u16, OkError> {
    let mut data = [0x0_u8; 2];
    hnd.read_from_pipe_out(0xA0, &mut data)?;
    Ok(make_u16_buffer(&data)[0])
}

fn write_array(hnd: &OkHandle, address: u8, data: &[u16]) -> Result<(), OkError> {
    let mut msg = vec![0_u16; data.len() + 2];
    msg[0] = 0x0300 | (address as u16);
    msg[1] = data.len() as u16;
    for (ndx, el) in data.iter().enumerate() {
        msg[ndx + 2] = *el;
    }
    // Send the message
    hnd.write_to_pipe_in(0x80, &mk_u8(&msg))
}

fn read_array(hnd: &OkHandle, address: u8, len: usize) -> Result<Vec<u16>, OkError> {
    let mut msg = vec![0_u16; 2];
    msg[0] = 0x0200 | (address as u16);
    msg[1] = len as u16;
    hnd.write_to_pipe_in(0x80, &mk_u8(&msg))?;
    let mut data = vec![0x0_u8; len * 2];
    hnd.read_from_pipe_out(0xA0, &mut data)?;
    Ok(make_u16_buffer(&data))
}

pub fn test_opalkelly_soc_hello(bit_name: &str) -> Result<(), OkError> {
    let hnd = ok_test_prelude(bit_name)?;
    for iter in 0..100 {
        println!("Iteration {}", iter);
        send_ping(&hnd, 0x67)?;
        let j = read_ping(&hnd)?;
        assert_eq!(j, 0x167);
        //let to_send = [0xDEAD_u16, 0xBEEF, 0xCAFE, 0xBABE];
        let to_send = (0..256).map(|_| rand::random::<u16>()).collect::<Vec<_>>();
        // Send a set of data elements
        write_array(&hnd, 0, &to_send)?;
        sleep(std::time::Duration::from_millis(100));
        // Read them back
        let ret = read_array(&hnd, 1, to_send.len())?;
        for (ndx, val) in ret.iter().enumerate() {
            assert_eq!(*val, to_send[ndx].wrapping_shl(1))
        }
    }
    hnd.close();
    Ok(())
}

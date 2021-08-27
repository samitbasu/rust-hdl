#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use std::ffi::{CStr, CString};
use std::num::Wrapping;
use std::{thread, time};

//use rand::Rng;

include!("bindings.rs");

pub struct OkHandle {
    hnd: okFrontPanel_HANDLE,
}

#[derive(Debug, Clone)]
pub struct OkError {
    pub code: ok_ErrorCode,
}

impl OkError {
    pub fn make_result(val: i32) -> Result<(), OkError> {
        if val == ok_ErrorCode_ok_NoError {
            return Result::Ok(());
        } else {
            return Result::Err(OkError { code: val });
        }
    }
}

impl Drop for OkHandle {
    fn drop(&mut self) {
        self.close();
        self.destruct();
    }
}

impl OkHandle {
    pub fn new() -> OkHandle {
        OkHandle {
            hnd: unsafe { okFrontPanel_Construct() },
        }
    }

    pub fn destruct(&self) {
        unsafe { okFrontPanel_Destruct(self.hnd) }
    }
    pub fn open(&self) -> Result<(), OkError> {
        let s = CString::new("").expect("Unable to create C String");
        OkError::make_result(unsafe { okFrontPanel_OpenBySerial(self.hnd, s.as_ptr()) })
    }

    pub fn open_with_serial(&self, serial: &str) -> Result<(), OkError> {
        let s = CString::new(serial).expect("Unable to create C String");
        OkError::make_result(unsafe { okFrontPanel_OpenBySerial(self.hnd, s.as_ptr()) })
    }

    pub fn close(&self) {
        unsafe {
            okFrontPanel_Close(self.hnd);
        }
    }

    pub fn rust_string(buffer: &[u8]) -> String {
        // FIXME - there has to be a simpler way...
        let mut ret = vec![];
        for char in buffer {
            if *char != 0_u8 {
                ret.push(*char);
            }
        }
        ret.push(0_u8);
        CStr::from_bytes_with_nul(&ret)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn get_board_model(&self) -> String {
        let model = unsafe { okFrontPanel_GetBoardModel(self.hnd) };
        let mut buffer = vec![0_u8; 256];
        unsafe { okFrontPanel_GetBoardModelString(self.hnd, model, buffer.as_mut_ptr() as _) };
        Self::rust_string(&buffer)
    }

    pub fn reset_firmware(&self, addr: i32) {
        self.set_wire_in(addr, 1);
        self.update_wire_ins();
        thread::sleep(time::Duration::from_millis(1000));
        self.set_wire_in(addr, 0);
        self.update_wire_ins();
    }

    pub fn load_default_pll_configuration(&self) -> Result<(), OkError> {
        OkError::make_result(unsafe { okFrontPanel_LoadDefaultPLLConfiguration(self.hnd) })
    }

    pub fn get_serial_number(&self) -> String {
        let mut buffer = vec![0; 256];
        unsafe { okFrontPanel_GetSerialNumber(self.hnd, buffer.as_mut_ptr() as _) };
        Self::rust_string(&buffer)
    }

    pub fn get_device_id(&self) -> String {
        let mut buffer = vec![0; 256];
        unsafe { okFrontPanel_GetDeviceID(self.hnd, buffer.as_mut_ptr() as _) };
        Self::rust_string(&buffer)
    }

    pub fn get_firmware_version(&self) -> (i32, i32) {
        let major = unsafe { okFrontPanel_GetDeviceMajorVersion(self.hnd) };
        let minor = unsafe { okFrontPanel_GetDeviceMinorVersion(self.hnd) };
        (major, minor)
    }

    pub fn get_api_version(&self) -> (i32, i32, i32) {
        let major = unsafe { okFrontPanel_GetAPIVersionMajor() };
        let minor = unsafe { okFrontPanel_GetAPIVersionMinor() };
        let micro = unsafe { okFrontPanel_GetAPIVersionMicro() };
        (major, minor, micro)
    }

    pub fn set_wire_in(&self, addr: i32, val: u16) {
        unsafe { okFrontPanel_SetWireInValue(self.hnd, addr, val as u64, 0xFFFF) };
    }

    pub fn update_wire_ins(&self) {
        unsafe { okFrontPanel_UpdateWireIns(self.hnd) };
    }

    pub fn get_wire_in(&self, addr: i32) -> Result<u16, OkError> {
        let mut val: u32 = 0;
        let ecode = unsafe { okFrontPanel_GetWireInValue(self.hnd, addr, &mut val) };
        if ecode == ok_ErrorCode_ok_NoError {
            Ok(val as u16)
        } else {
            Err(OkError { code: ecode })
        }
    }

    pub fn update_wire_outs(&self) {
        unsafe { okFrontPanel_UpdateWireOuts(self.hnd) };
    }

    pub fn get_wire_out(&self, addr: i32) -> u16 {
        let val = unsafe { okFrontPanel_GetWireOutValue(self.hnd, addr) } as u16;
        val
    }

    pub fn configure_fpga(&self, firmware: &str) -> Result<(), OkError> {
        let filename = CString::new(firmware).expect("CString new failed");
        let code = unsafe { okFrontPanel_ConfigureFPGA(self.hnd, filename.as_ptr()) };
        OkError::make_result(code)
    }

    pub fn activate_trigger_in(&self, addr: i32, bit: i32) -> Result<(), OkError> {
        let code = unsafe { okFrontPanel_ActivateTriggerIn(self.hnd, addr, bit) };
        OkError::make_result(code)
    }

    pub fn update_trigger_outs(&self) {
        unsafe {
            okFrontPanel_UpdateTriggerOuts(self.hnd);
        }
    }

    pub fn is_triggered(&self, addr: i32, mask: i32) -> bool {
        let b = unsafe { okFrontPanel_IsTriggered(self.hnd, addr, mask as _) };
        if b == 0 {
            false
        } else {
            true
        }
    }

    pub fn write_to_pipe_in(&self, addr: i32, data: &[u8]) -> Result<(), OkError> {
        let cnt = unsafe {
            okFrontPanel_WriteToPipeIn(self.hnd, addr, data.len() as _, data.as_ptr() as _)
        } as usize;
        if cnt == data.len() {
            Ok(())
        } else {
            Err(OkError { code: -1 })
        }
    }

    pub fn write_to_block_pipe_in(
        &self,
        addr: i32,
        blocksize: i32,
        data: &[u8],
    ) -> Result<(), OkError> {
        let cnt = unsafe {
            okFrontPanel_WriteToBlockPipeIn(
                self.hnd,
                addr,
                blocksize,
                data.len() as _,
                data.as_ptr() as _,
            )
        } as usize;
        if cnt == data.len() {
            Ok(())
        } else {
            println!(
                "Error: Write of cnt = {} not equal to data len = {}",
                cnt,
                data.len()
            );
            Err(OkError { code: -1 })
        }
    }

    pub fn read_from_pipe_out(&self, addr: i32, data: &mut [u8]) -> Result<(), OkError> {
        let cnt = unsafe {
            okFrontPanel_ReadFromPipeOut(self.hnd, addr, data.len() as _, data.as_mut_ptr())
        } as usize;
        if cnt == data.len() {
            Ok(())
        } else {
            Err(OkError { code: -1 })
        }
    }

    pub fn read_from_block_pipe_out(
        &self,
        addr: i32,
        blocksize: i32,
        data: &mut [u8],
    ) -> Result<(), OkError> {
        let cnt = unsafe {
            okFrontPanel_ReadFromBlockPipeOut(
                self.hnd,
                addr,
                blocksize,
                data.len() as _,
                data.as_mut_ptr(),
            )
        } as usize;
        if cnt == data.len() {
            Ok(())
        } else {
            Err(OkError { code: -1 })
        }
    }

    pub fn reset_fpga(&self) -> Result<(), OkError> {
        let code = unsafe { okFrontPanel_ResetFPGA(self.hnd) };
        OkError::make_result(code)
    }
}

pub fn make_u16_buffer(data: &[u8]) -> Vec<u16> {
    let mut ret = vec![];
    for i in (0..data.len()).step_by(2) {
        ret.push(((data[i] as u16) | ((data[i + 1] as u16) << 8)) as u16);
    }
    ret
}

pub fn make_u32_buffer(data: &[u16]) -> Vec<u32> {
    let mut ret = vec![];
    for i in (0..data.len()).step_by(2) {
        ret.push(((data[i] as u32) | ((data[i + 1] as u32) << 16)) as u32);
    }
    ret
}

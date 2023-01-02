use rust_hdl_ok_core::core::prelude::*;

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::i2c::i2c_controller::{I2CController, I2CControllerCmd};
use rust_hdl::widgets::prelude::*;

use rust_hdl_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_ok_core::test_common::tools::ok_test_prelude;
use rust_hdl_ok_frontpanel_sys::{OkError, OkHandle};
use std::thread::sleep;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct OKI2CAddressConfig {
    pub trigger_start_address: u8,
    pub trigger_done_address: u8,
    pub wire_in_address: u8,
    pub wire_out_address: u8,
}

// To use the I2C controller, we need to be able to
// use a set of triggers, and a wire
// trigger bit 0 = run
//  - lower 8 bits are the value
//          upper bits are the command
// Command values are 0 BeginWrite (addr = wire_in)
//                    1 = Write      (byte = wire_in)
//                    2 = BeginRead  (addr = wire_in)
//                    3 = Read       (value = wire_out)
//                    4 = EndTransmission
//
// We need two output triggers
// trigger bit 0 = NACK
// trigger bit 1 = ACK
//
// We need a status wire that contains:
// status bit 8 = busy
// status bit 9 = error
//

#[derive(LogicBlock)]
struct OKI2CTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub sda: Signal<InOut, Bit>,
    pub scl: Signal<InOut, Bit>,
    pub i2c: I2CController,
    pub trig_in: TriggerIn,
    pub trig_out: TriggerOut,
    pub wire_in: WireIn,
    pub wire_out: WireOut,
    pub wire_out_latch: DFF<Bits<8>>,
}

impl Logic for OKI2CTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Link the signals
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        Signal::<InOut, Bit>::link(&mut self.sda, &mut self.i2c.sda);
        Signal::<InOut, Bit>::link(&mut self.scl, &mut self.i2c.scl);
        // Clock the internal logic
        self.i2c.clock.next = self.ok_host.ti_clk.val();
        self.trig_in.clk.next = self.ok_host.ti_clk.val();
        self.trig_out.clk.next = self.ok_host.ti_clk.val();
        self.wire_out_latch.clock.next = self.ok_host.ti_clk.val();
        // Latch prevention
        self.wire_out_latch.d.next = self.wire_out_latch.q.val();
        // Connect the OK busses
        self.trig_in.ok1.next = self.ok_host.ok1.val();
        self.trig_out.ok1.next = self.ok_host.ok1.val();
        self.wire_in.ok1.next = self.ok_host.ok1.val();
        self.wire_out.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.trig_out.ok2.val() | self.wire_out.ok2.val();
        // Output wire is [0..0 err busy data_byte]
        self.wire_out.datain.next = bit_cast::<16, 8>(self.wire_out_latch.q.val())
            | (bit_cast::<16, 1>(self.i2c.busy.val().into()) << 8)
            | (bit_cast::<16, 1>(self.i2c.error.val().into()) << 9);
        // Nack and ACK
        self.trig_out.trigger.next = bit_cast::<16, 1>(self.i2c.nack.val().into())
            | (bit_cast::<16, 1>(self.i2c.ack.val().into()) << 1)
            | (bit_cast::<16, 1>(self.i2c.read_valid.val().into()) << 2);
        // Wire in contains the address or data byte to write
        self.i2c.write_data_in.next = self.wire_in.dataout.val().get_bits::<8>(0);
        // Use the remaining triggers to determine the command
        self.i2c.cmd.next = I2CControllerCmd::Noop;
        match self.wire_in.dataout.val().get_bits::<3>(8).index() {
            0 => self.i2c.cmd.next = I2CControllerCmd::BeginWrite,
            1 => self.i2c.cmd.next = I2CControllerCmd::Write,
            2 => self.i2c.cmd.next = I2CControllerCmd::BeginRead,
            3 => self.i2c.cmd.next = I2CControllerCmd::Read,
            4 => self.i2c.cmd.next = I2CControllerCmd::EndTransmission,
            5 => self.i2c.cmd.next = I2CControllerCmd::ReadLast,
            _ => self.i2c.cmd.next = I2CControllerCmd::Noop,
        }
        self.i2c.run.next = self.trig_in.trigger.val().any();
        if self.i2c.read_valid.val() {
            self.wire_out_latch.d.next = self.i2c.read_data_out.val();
        }
    }
}

impl Default for OKI2CTest {
    fn default() -> Self {
        // SCL - J3 p69 - A15
        // SDA - J3 p71 - C17
        let mut sda = Signal::default();
        sda.add_location(0, "C17");
        sda.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        let mut scl = Signal::default();
        scl.add_location(0, "A15");
        scl.add_signal_type(0, SignalType::LowVoltageCMOS_3v3);
        // Delay time is baud period/2.
        // For 100Khz operation, a single bit takes 1e6/100e5 = 10usec
        // So we use have that period for the delay time
        let config = I2CConfig {
            delay_time: Duration::from_micros(5),
            clock_speed_hz: 48_000_000,
        };
        Self {
            hi: XEM6010::hi(),
            ok_host: XEM6010::ok_host(),
            sda,
            scl,
            i2c: I2CController::new(config),
            trig_in: TriggerIn::new(0x46),
            trig_out: TriggerOut::new(0x66),
            wire_in: WireIn::new(0x06),
            wire_out: WireOut::new(0x26),
            wire_out_latch: Default::default(),
        }
    }
}

#[test]
#[ignore]
// Currently requires an I2C chip connected...
// TODO - update this so that the I2C chip is also on the FPGA.
fn test_opalkelly_xem_6010_i2c() {
    let mut uut = OKI2CTest::default();
    uut.hi.link_connect_dest();
    uut.sda.connect();
    uut.scl.connect();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/i2c_test"));
    test_opalkelly_xem_6010_run_i2c().unwrap()
}

fn ok_wait_valid(hnd: &OkHandle) -> Result<(), OkError> {
    while !hnd.is_triggered(0x66, 4) {
        hnd.update_trigger_outs();
    }
    Ok(())
}

fn ok_wait_ack(hnd: &OkHandle) -> Result<(), OkError> {
    let mut acked = false;
    let mut nacked = false;
    let mut error = false;
    while !acked & !nacked & !error {
        nacked = hnd.is_triggered(0x66, 1);
        acked = hnd.is_triggered(0x66, 2);
        hnd.update_trigger_outs();
        hnd.update_wire_outs();
        let status = hnd.get_wire_out(0x26);
        error = (status & 0x0200) != 0;
    }
    if error {
        return Err(OkError {
            code: rust_hdl_ok_frontpanel_sys::ok_ErrorCode_ok_CommunicationError,
        });
    }
    if acked {
        Ok(())
    } else {
        Err(OkError {
            code: rust_hdl_ok_frontpanel_sys::ok_ErrorCode_ok_I2CNack,
        })
    }
}

fn ok_i2c_cmd(hnd: &OkHandle, cmd: u16) -> Result<(), OkError> {
    // Issue a basic transaction - do a beginwrite to address 0x53
    hnd.set_wire_in(0x06, cmd);
    hnd.update_wire_ins();
    hnd.activate_trigger_in(0x46, 0)
}

fn ok_i2c_reset(hnd: &OkHandle) -> Result<(), OkError> {
    hnd.set_wire_in(0x06, 0xFFFF);
    hnd.update_wire_ins();
    hnd.set_wire_in(0x06, 0xFFFF);
    hnd.update_wire_ins();
    Ok(())
}

fn ok_i2c_begin_write(hnd: &OkHandle, addr: u8) -> Result<(), OkError> {
    ok_i2c_cmd(&hnd, (0 << 8) | addr as u16)?;
    ok_wait_ack(&hnd)
}

fn ok_i2c_write(hnd: &OkHandle, data: u8) -> Result<(), OkError> {
    ok_i2c_cmd(&hnd, (1 << 8) | data as u16)?;
    ok_wait_ack(&hnd)
}

fn ok_i2c_begin_read(hnd: &OkHandle, addr: u8) -> Result<(), OkError> {
    ok_i2c_cmd(&hnd, (2 << 8) | addr as u16)?;
    ok_wait_ack(&hnd)
}

fn ok_i2c_read(hnd: &OkHandle) -> Result<u8, OkError> {
    ok_i2c_cmd(&hnd, (3 << 8) | 0x00)?;
    ok_wait_valid(&hnd)?;
    hnd.update_wire_outs();
    let status = hnd.get_wire_out(0x26);
    Ok((status & 0xFF) as u8)
}

fn ok_i2c_read_last(hnd: &OkHandle) -> Result<u8, OkError> {
    ok_i2c_cmd(&hnd, (5 << 8) | 0x00)?;
    ok_wait_valid(&hnd)?;
    hnd.update_wire_outs();
    let status = hnd.get_wire_out(0x26);
    Ok((status & 0xFF) as u8)
}

fn ok_i2c_end_transmission(hnd: &OkHandle) -> Result<(), OkError> {
    ok_i2c_cmd(&hnd, (4 << 8) | 0x00)?;
    // Get the status word
    hnd.update_wire_outs();
    let _status = hnd.get_wire_out(0x26);
    Ok(())
}

fn test_opalkelly_xem_6010_run_i2c() -> Result<(), OkError> {
    let hnd = ok_test_prelude(target_path!("xem_6010/i2c_test/top.bit"))?;
    ok_i2c_reset(&hnd)?;
    ok_i2c_begin_write(&hnd, 0x48)?;
    ok_i2c_write(&hnd, 0x0F)?;
    ok_i2c_begin_read(&hnd, 0x048)?;
    let b1 = ok_i2c_read(&hnd)?;
    let b2 = ok_i2c_read_last(&hnd)?;
    ok_i2c_end_transmission(&hnd)?;
    println!("ID {:x}{:x}", b1, b2);
    for _ in 0..100 {
        ok_i2c_begin_write(&hnd, 0x48)?;
        ok_i2c_write(&hnd, 0x00)?;
        ok_i2c_begin_read(&hnd, 0x048)?;
        let b1 = ok_i2c_read(&hnd)?;
        let b2 = ok_i2c_read_last(&hnd)?;
        ok_i2c_end_transmission(&hnd)?;
        let temp = (b1 as u16) << 8 | (b2 as u16);
        let temp_c = (temp as f32 + 256.0) * 7.8125 / 1000.0;
        println!("Temp {:x}{:x} {} deg C", b1, b2, temp_c);
        sleep(Duration::from_millis(100))
    }
    Ok(())
}

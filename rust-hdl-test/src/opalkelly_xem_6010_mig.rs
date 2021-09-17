use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::mcb_if::MCBInterface1GDDR2;
use rust_hdl_ok::mig::MemoryInterfaceGenerator;
use rust_hdl_ok::ok_hi::OpalKellyHostInterface;
use rust_hdl_ok::ok_host::OpalKellyHost;
use rust_hdl_ok::ok_pipe::{PipeIn, PipeOut};
use rust_hdl_ok::ok_trigger::{TriggerIn, TriggerOut};
use rust_hdl_ok::ok_wire::WireIn;
use rust_hdl_ok::prelude::xem_6010_base_clock;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_widgets::prelude::*;
use std::thread::sleep;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010MIGTest {
    pub hi: OpalKellyHostInterface,
    pub mcb: MCBInterface1GDDR2,
    pub raw_clock: Signal<In, Clock>,
    pub ok_host: OpalKellyHost,
    pub mig: MemoryInterfaceGenerator,
    pub reset: WireIn,
    pub pipe_in: PipeIn,
    pub address: WireIn,
    pub pipe_out: PipeOut,
    pub start_cmd: TriggerIn,
    pub cmd_done: TriggerOut,
    pub read_delay: DFF<Bit>,
}

impl Default for OpalKellyXEM6010MIGTest {
    fn default() -> Self {
        let raw_clock = xem_6010_base_clock();
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            mcb: MCBInterface1GDDR2::xem_6010(),
            raw_clock,
            ok_host: OpalKellyHost::xem_6010(),
            mig: Default::default(),
            reset: WireIn::new(0x00),
            pipe_in: PipeIn::new(0x80),
            address: WireIn::new(0x01),
            pipe_out: PipeOut::new(0xA0),
            start_cmd: TriggerIn::new(0x40),
            cmd_done: TriggerOut::new(0x60),
            read_delay: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010MIGTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interfaces
        self.hi.link(&mut self.ok_host.hi);
        self.mcb.link(&mut self.mig.mcb);
        // Clocks
        self.mig.raw_sys_clk.next = self.raw_clock.val();
        self.mig.p0_wr.clock.next = self.ok_host.ti_clk.val();
        self.mig.p0_rd.clock.next = self.ok_host.ti_clk.val();
        self.mig.p0_cmd.clock.next = self.ok_host.ti_clk.val();
        self.read_delay.clk.next = self.ok_host.ti_clk.val();
        self.start_cmd.clk.next = self.ok_host.ti_clk.val();
        self.cmd_done.clk.next = self.ok_host.ti_clk.val();
        // Reset
        self.mig.reset.next = self.reset.dataout.val().any();
        // Couple the input pipe to the write fifo
        self.mig.p0_wr.data.next = bit_cast::<32, 16>(self.pipe_in.dataout.val());
        self.mig.p0_wr.enable.next = self.pipe_in.write.val();
        // Couple the output pipe to the read fifo
        // Use a delay register, since the MIG FIFOs are 0-delay
        self.pipe_out.datain.next = bit_cast::<16, 32>(self.mig.p0_rd.data.val());
        self.mig.p0_rd.enable.next = self.read_delay.q.val();
        self.read_delay.d.next = self.pipe_out.read.val();
        // Hard code the burst length
        self.mig.p0_cmd.burst_length.next = 63_u32.into();
        // set the address value
        self.mig.p0_cmd.byte_address.next = bit_cast::<30, 16>(self.address.dataout.val());
        // Default command is to do nothing... refresh
        self.mig.p0_cmd.instruction.next = 4_u8.into();
        self.mig.p0_cmd.enable.next = false;
        // Set the appropriate command.
        if self.start_cmd.trigger.val().get_bit(0_usize) {
            self.mig.p0_cmd.instruction.next = 0_u8.into();
            self.mig.p0_cmd.enable.next = true;
        } else if self.start_cmd.trigger.val().get_bit(1_usize) {
            self.mig.p0_cmd.instruction.next = 1_u8.into();
            self.mig.p0_cmd.enable.next = true;
        }
        self.cmd_done.trigger.next = 0_u32.into();
        if self.mig.p0_rd.full.val() {
            self.cmd_done.trigger.next = 1_u32.into();
        }
        // Connect the ok busses
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.start_cmd.ok1.next = self.ok_host.ok1.val();
        self.cmd_done.ok1.next = self.ok_host.ok1.val();
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.address.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next =
            self.pipe_in.ok2.val() | self.pipe_out.ok2.val() | self.cmd_done.ok2.val();
        // Unused inputs
        self.mig.p0_wr.mask.next = 0_u32.into();
    }
}

#[test]
fn test_opalkelly_xem_6010_mig() {
    let mut uut = OpalKellyXEM6010MIGTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.raw_clock.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj_6010(uut, "opalkelly_xem_6010_mig");
}

#[test]
fn test_opalkelly_xem_6010_mig_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_mig/top.bit")?;
    hnd.reset_firmware(0);
    let data = (64..(128 + 64)).collect::<Vec<u8>>();
    hnd.write_to_pipe_in(0x80, &data).unwrap();
    hnd.activate_trigger_in(0x40, 0).unwrap();
    sleep(Duration::from_millis(100));
    hnd.activate_trigger_in(0x40, 1).unwrap();
    sleep(Duration::from_millis(100));
    while !hnd.is_triggered(0x60, 1) {
        hnd.update_trigger_outs();
    }
    let mut data_out = vec![0_u8; 128];
    hnd.read_from_pipe_out(0xA0, &mut data_out).unwrap();
    println!("Output data {:?}", data_out);
    for k in 0..data_out.len() {
        assert_eq!(data[k], data_out[k])
    }
    hnd.set_wire_in(1, 32 * 4);
    hnd.update_wire_ins();
    hnd.activate_trigger_in(0x40, 1).unwrap();
    while !hnd.is_triggered(0x60, 1) {
        hnd.update_trigger_outs();
    }
    hnd.read_from_pipe_out(0xA0, &mut data_out).unwrap();
    println!("Output data {:?}", data_out);
    for k in 0..64 {
        assert_eq!(data[k + 64], data_out[k])
    }
    Ok(())
}

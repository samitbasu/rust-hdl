use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::mcb_if::MCBInterface4GDDR3;
use rust_hdl_ok::mig7::MemoryInterfaceGenerator7Series;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_widgets::prelude::*;
use std::thread::sleep;
use std::time::Duration;

declare_expanding_fifo!(FrontPorch, 16, 4096, 128, 256);
declare_narrowing_fifo!(BackPorch, 128, 256, 16, 4096);

#[derive(LogicState, Clone, Debug, PartialEq, Copy)]
pub enum MIGState {
    Idle,
    Write,
    WaitAck,
    Read,
    WaitRead,
}

#[derive(LogicBlock)]
pub struct OpalKellyXEM7010MIGTest {
    pub hi: OpalKellyHostInterface,
    pub mcb: MCBInterface4GDDR3,
    pub sys_clock_pos: Signal<In, Clock>,
    pub sys_clock_neg: Signal<In, Clock>,
    pub ok_host: OpalKellyHost,
    pub mig: MemoryInterfaceGenerator7Series,
    pub reset: WireIn,
    // These are used to bring data into the MIG
    pub pipe_in: PipeIn,
    front_porch: FrontPorch,
    // The address for the transaction
    pub address: WireIn,
    // These are used to bring data out of the MIG
    back_porch: BackPorch,
    pub pipe_out: PipeOut,
    delay_read: DFF<Bit>,
    // Triggers to start the state machine
    pub start_cmd: TriggerIn,
    pub cmd_done: TriggerOut,
    state: DFF<MIGState>,
}

impl Default for OpalKellyXEM7010MIGTest {
    fn default() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_7010(),
            mcb: MCBInterface4GDDR3::xem_7010(), // Constraints provided by the IP core
            sys_clock_pos: Default::default(),   // Constraints provided by the IP core
            sys_clock_neg: Default::default(),   // Constraints provided by the IP core
            ok_host: OpalKellyHost::xem_7010(),
            mig: Default::default(),
            reset: WireIn::new(0x00),
            front_porch: FrontPorch::new(WordOrder::MostSignificantFirst),
            pipe_in: PipeIn::new(0x80),
            address: WireIn::new(0x01),
            back_porch: BackPorch::new(WordOrder::LeastSignificantFirst),
            pipe_out: PipeOut::new(0xA0),
            delay_read: Default::default(),
            start_cmd: TriggerIn::new(0x40),
            cmd_done: TriggerOut::new(0x60),
            state: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM7010MIGTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interfaces
        self.hi.link(&mut self.ok_host.hi);
        self.mcb.link(&mut self.mig.mcb);
        // Clocks
        self.mig.raw_pos_clock.next = self.sys_clock_pos.val();
        self.mig.raw_neg_clock.next = self.sys_clock_neg.val();
        // Connect the clocks for the input side
        self.front_porch.write_clock.next = self.ok_host.ti_clk.val();
        self.front_porch.read_clock.next = self.mig.clock.val();
        // Connect the clocks for the output side
        self.back_porch.write_clock.next = self.mig.clock.val();
        self.back_porch.read_clock.next = self.ok_host.ti_clk.val();
        // Connect the trigger clocks to the mig clock
        self.start_cmd.clk.next = self.mig.clock.val();
        self.cmd_done.clk.next = self.mig.clock.val();
        // Wire up the input side
        self.front_porch.data_in.next = self.pipe_in.dataout.val();
        self.front_porch.write.next = self.pipe_in.write.val();
        // Wire up the output side
        self.pipe_out.datain.next = self.back_porch.data_out.val();
        self.delay_read.d.next = self.pipe_out.read.val();
        self.back_porch.read.next = self.delay_read.q.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();
        // Connect the OK busses
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.start_cmd.ok1.next = self.ok_host.ok1.val();
        self.cmd_done.ok1.next = self.ok_host.ok1.val();
        self.address.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next =
            self.pipe_in.ok2.val() | self.pipe_out.ok2.val() | self.cmd_done.ok2.val();
        self.mig.address.next = bit_cast::<29, 16>(self.address.dataout.val());
        self.mig.write_data_in.next = self.front_porch.data_out.val();
        self.back_porch.data_in.next = self.mig.read_data_out.val();
        self.back_porch.write.next = false;
        self.mig.command.next = 0_u32.into();
        self.mig.enable.next = false;
        self.mig.write_data_mask.next = 0_u16.into();
        self.mig.reset.next = self.reset.dataout.val().any();
        self.state.d.next = self.state.q.val();
        self.state.clk.next = self.mig.clock.val();
        // Control signals
        self.front_porch.read.next = false;
        self.mig.write_enable.next = false;
        self.mig.write_data_end.next = false;
        self.cmd_done.trigger.next = 0_usize.into();
        match self.state.q.val() {
            MIGState::Idle => {
                if self.start_cmd.trigger.val().get_bit(0_usize.into()) {
                    self.state.d.next = MIGState::Write;
                }
                if self.start_cmd.trigger.val().get_bit(1_usize.into()) {
                    self.state.d.next = MIGState::Read;
                }
            }
            MIGState::Write => {
                if !self.front_porch.empty.val()
                    & self.mig.write_fifo_not_full.val()
                    & self.mig.ready.val()
                {
                    self.mig.write_enable.next = true;
                    self.mig.write_data_end.next = true;
                    self.front_porch.read.next = true;
                    self.state.d.next = MIGState::WaitAck;
                }
            }
            MIGState::WaitAck => {
                self.mig.command.next = 0_u8.into(); // This is a write command
                self.mig.enable.next = true;
                if self.mig.ready.val() {
                    self.state.d.next = MIGState::Idle;
                    self.cmd_done.trigger.next = 1_usize.into();
                }
            }
            MIGState::Read => {
                if !self.back_porch.full.val() & self.mig.ready.val() {
                    self.mig.command.next = 1_u8.into();
                    self.mig.enable.next = true;
                    self.state.d.next = MIGState::WaitRead;
                }
            }
            MIGState::WaitRead => {
                self.back_porch.write.next = self.mig.read_data_valid.val();
                if self.mig.read_data_valid.val() {
                    self.state.d.next = MIGState::Idle;
                    self.cmd_done.trigger.next = 2_usize.into();
                }
            }
        }
    }
}

#[test]
fn test_opalkelly_xem_7010_mig() {
    let mut uut = OpalKellyXEM7010MIGTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_pos.connect();
    uut.sys_clock_neg.connect();
    uut.cmd_done.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem7010_mig");
}

#[test]
fn test_opalkelly_xem_7010_mig_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("xem7010_mig/top.bit")?;
    hnd.reset_firmware(0);
    let data = (0..64).collect::<Vec<u8>>();
    println!("Input data {:?}", data);
    hnd.write_to_pipe_in(0x80, &data).unwrap();
    for packet in 0..4 {
        // Issue a write command
        hnd.set_wire_in(0x1, packet);
        hnd.update_wire_ins();
        hnd.activate_trigger_in(0x40, 0);
        hnd.update_trigger_outs();
        while !hnd.is_triggered(0x60, 0x01) {
            hnd.update_trigger_outs();
        }
        // Issue a read command
        hnd.activate_trigger_in(0x40, 1);
        hnd.update_trigger_outs();
        while !hnd.is_triggered(0x60, 0x02) {
            hnd.update_trigger_outs();
        }
    }
    let mut data_out = vec![0_u8; 64];
    hnd.read_from_pipe_out(0xA0, &mut data_out).unwrap();
    println!("Output data {:?}", data_out);
    Ok(())
}

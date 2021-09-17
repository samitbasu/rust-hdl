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
    //pub address: WireIn,
    // These are used to bring data out of the MIG
    back_porch: BackPorch,
    pub pipe_out: PipeOut,
    delay_read: DFF<Bit>,
    // Triggers to start the state machine
    //  pub start_cmd: TriggerIn,
    //    pub cmd_done: TriggerOut,
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
            //      address: WireIn::new(0x01),
            back_porch: BackPorch::new(WordOrder::LeastSignificantFirst),
            pipe_out: PipeOut::new(0xA0),
            delay_read: Default::default(),
            //    start_cmd: TriggerIn::new(0x40),
            //            cmd_done: TriggerOut::new(0x60),
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
        // Wire up the input side
        self.front_porch.data_in.next = self.pipe_in.dataout.val();
        self.front_porch.write.next = self.pipe_in.write.val();
        // For now, skip the MIG... (but use it's clock)
        self.back_porch.write.next = !self.front_porch.empty.val() & !self.back_porch.full.val();
        self.front_porch.read.next = !self.front_porch.empty.val() & !self.back_porch.full.val();
        self.back_porch.data_in.next = self.front_porch.data_out.val();
        self.pipe_out.datain.next = self.back_porch.data_out.val();
        self.delay_read.d.next = self.pipe_out.read.val();
        self.back_porch.read.next = self.delay_read.q.val();
        self.delay_read.clk.next = self.ok_host.ti_clk.val();
        // Connect the OK busses
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.reset.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.pipe_in.ok2.val() | self.pipe_out.ok2.val();
        self.mig.address.next = 0_u32.into();
        self.mig.write_data_in.next = 0_u128.into();
        self.mig.command.next = 0_u32.into();
        self.mig.enable.next = false;
        self.mig.write_enable.next = false;
        self.mig.write_data_end.next = false;
        self.mig.write_data_mask.next = 0_u16.into();
        self.mig.reset.next = self.reset.dataout.val().any();
    }
}

#[test]
fn test_opalkelly_xem_7010_mig() {
    let mut uut = OpalKellyXEM7010MIGTest::default();
    uut.hi.link_connect_dest();
    uut.mcb.link_connect_dest();
    uut.sys_clock_pos.connect();
    uut.sys_clock_neg.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj_7010(uut, "xem7010_mig");
}

#[test]
fn test_opalkelly_xem_7010_mig_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("xem7010_mig/top.bit")?;
    hnd.reset_firmware(0);
    let data = (64..(32 + 64)).collect::<Vec<u8>>();
    println!("Output data {:?}", data);
    hnd.write_to_pipe_in(0x80, &data).unwrap();
    sleep(Duration::from_millis(100));
    let mut data_out = vec![0_u8; 32];
    hnd.read_from_pipe_out(0xA0, &mut data_out).unwrap();
    println!("Output data {:?}", data_out);
    Ok(())
}

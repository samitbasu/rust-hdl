// Build a SoC and connect it to an OK host

use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl_ok_core::core::prelude::*;
mod test_common;

use rust_hdl_ok_core::xem6010::pins::xem_6010_base_clock;

use rust_hdl_ok_core::xem6010::XEM6010;

use crate::test_common::soc::test_opalkelly_soc_hello;
use test_common::soc::SoCTestChip;

#[derive(LogicBlock)]
struct OpalKellySoCTest {
    hi: OpalKellyHostInterface,
    ok_host: OpalKellyHost,
    sys_clock: Signal<In, Clock>,
    pipe_in: PipeIn,
    pipe_out: PipeOut,
    dut: SoCTestChip,
    read_delay: DFF<Bit>,
}

impl Default for OpalKellySoCTest {
    fn default() -> Self {
        Self {
            hi: XEM6010::hi(),
            ok_host: XEM6010::ok_host(),
            sys_clock: xem_6010_base_clock(),
            pipe_in: PipeIn::new(0x80),
            pipe_out: PipeOut::new(0xA0),
            dut: Default::default(),
            read_delay: Default::default(),
        }
    }
}

impl Logic for OpalKellySoCTest {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        self.read_delay.clock.next = self.ok_host.ti_clk.val();
        self.dut.clock.next = self.ok_host.ti_clk.val();
        self.dut.sys_clock.next = self.sys_clock.val();
        self.dut.from_cpu.data.next = self.pipe_in.dataout.val();
        self.dut.from_cpu.write.next = self.pipe_in.write.val();
        self.pipe_out.datain.next = self.dut.to_cpu.data.val();
        self.read_delay.d.next = self.pipe_out.read.val();
        self.dut.to_cpu.read.next = self.read_delay.q.val();
        self.pipe_in.ok1.next = self.ok_host.ok1.val();
        self.pipe_out.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.pipe_in.ok2.val() | self.pipe_out.ok2.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_soc() {
    let mut uut = OpalKellySoCTest::default();
    uut.hi.link_connect_dest();
    uut.sys_clock.connect();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/soc_hello"));
    test_opalkelly_soc_hello(target_path!("xem_6010/soc_hello/top.bit")).unwrap();
}

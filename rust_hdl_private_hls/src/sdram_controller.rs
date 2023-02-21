use crate::bridge::Bridge;
use crate::bus::{SoCBusResponder, SoCPortController};
use crate::miso_wide_port::MISOWidePort;
use crate::mosi_port::MOSIPort;
use crate::mosi_wide_port::MOSIWidePort;
use crate::HLSNamedPorts;
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct SDRAMController<const R: usize, const C: usize> {
    pub dram: SDRAMDriver<16>,
    pub upstream: SoCBusResponder<16, 8>,
    local_bridge: Bridge<16, 8, 4>,
    data_in: MOSIWidePort<64, 16>,
    address: MOSIWidePort<32, 16>,
    cmd: MOSIPort<16>,
    data_out: MISOWidePort<64, 16>,
    controller: SDRAMBaseController<R, C, 64, 16>,
}

impl<const R: usize, const C: usize> SDRAMController<R, C> {
    pub fn new(
        cas_delay: u32,
        timings: MemoryTimings,
        buffer: OutputBuffer,
    ) -> SDRAMController<R, C> {
        Self {
            dram: Default::default(),
            upstream: Default::default(),
            local_bridge: Bridge::new(["data_in", "address", "cmd", "data_out"]),
            data_in: Default::default(),
            address: Default::default(),
            cmd: Default::default(),
            data_out: Default::default(),
            controller: SDRAMBaseController::new(cas_delay, timings, buffer),
        }
    }
}

impl<const R: usize, const C: usize> HLSNamedPorts for SDRAMController<R, C> {
    fn ports(&self) -> Vec<String> {
        self.local_bridge.ports()
    }
}

impl<const R: usize, const C: usize> Logic for SDRAMController<R, C> {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.local_bridge.upstream);
        SDRAMDriver::<16>::link(&mut self.dram, &mut self.controller.sdram);
        self.controller.clock.next = self.upstream.clock.val();
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[0], &mut self.data_in.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[1], &mut self.address.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[2], &mut self.cmd.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[3], &mut self.data_out.bus);
        self.data_out.port_in.next = self.controller.data_out.val();
        self.data_out.strobe_in.next = self.controller.data_valid.val();
        self.controller.data_in.next = self.data_in.port_out.val();
        self.controller.cmd_address.next = self.address.port_out.val();
        self.controller.write_not_read.next = self.cmd.port_out.val().any();
        self.controller.cmd_strobe.next = self.cmd.strobe_out.val();
        self.cmd.ready.next = !self.controller.busy.val();
    }
}

#[test]
fn test_sdram_controller_synthesizes() {
    let mut uut =
        SDRAMController::<6, 4>::new(3, MemoryTimings::fast_boot_sim(100e6), OutputBuffer::Wired);
    uut.connect_all();
    yosys_validate("sdram_controller_hls", &generate_verilog(&uut)).unwrap();
}

use crate::core::prelude::*;
use crate::hls::bus::FIFOWriteResponder;
use crate::hls::fifo::SyncFIFO;
use crate::hls::miso_port::MISOPort;
use crate::hls::prelude::SoCPortResponder;

#[derive(LogicBlock, Default)]
pub struct MISOFIFOPort<const W: usize, const N: usize, const NP1: usize, const BLOCK: u32> {
    pub bus: SoCPortResponder<W>,
    port: MISOPort<W>,
    fifo: SyncFIFO<Bits<W>, N, NP1, BLOCK>,
    pub fifo_bus: FIFOWriteResponder<Bits<W>>,
}

impl<const W: usize, const N: usize, const NP1: usize, const BLOCK: u32> Logic
    for MISOFIFOPort<W, N, NP1, BLOCK>
{
    #[hdl_gen]
    fn update(&mut self) {
        SoCPortResponder::<W>::link(&mut self.bus, &mut self.port.bus);
        self.fifo.clock.next = self.bus.clock.val();
        self.fifo.bus_read.read.next = self.port.strobe_out.val();
        self.port.ready_in.next = !self.fifo.bus_read.empty.val();
        self.port.port_in.next = self.fifo.bus_read.data.val();
        FIFOWriteResponder::<Bits<W>>::link(&mut self.fifo_bus, &mut self.fifo.bus_write);
    }
}

#[test]
fn test_miso_fifo_port_is_synthesizable() {
    let mut dev = MISOFIFOPort::<16, 4, 5, 1>::default();
    dev.bus.link_connect_dest();
    dev.fifo_bus.link_connect_dest();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    yosys_validate("miso_fifo_port", &vlog).unwrap();
}

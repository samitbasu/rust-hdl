use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, SoCPortResponder};
use crate::hls::fifo::SyncFIFO;
use crate::hls::mosi_port::MOSIPort;

#[derive(LogicBlock, Default)]
pub struct MOSIFIFOPort<const W: usize, const N: usize, const NP1: usize, const BLOCK: u32> {
    pub bus: SoCPortResponder<W>,
    port: MOSIPort<W>,
    fifo: SyncFIFO<Bits<W>, N, NP1, BLOCK>,
    pub fifo_bus: FIFOReadResponder<Bits<W>>,
}

impl<const W: usize, const N: usize, const NP1: usize, const BLOCK: u32> Logic
    for MOSIFIFOPort<W, N, NP1, BLOCK>
{
    #[hdl_gen]
    fn update(&mut self) {
        SoCPortResponder::<W>::link(&mut self.bus, &mut self.port.bus);
        self.fifo.clock.next = self.bus.clock.val();
        self.fifo.bus_write.data.next = self.port.port_out.val();
        self.fifo.bus_write.write.next = self.port.strobe_out.val();
        self.port.ready.next = !self.fifo.bus_write.full.val();
        FIFOReadResponder::<Bits<W>>::link(&mut self.fifo_bus, &mut self.fifo.bus_read);
    }
}

#[test]
fn test_mosi_fifo_port_is_synthesizable() {
    let mut dev = MOSIFIFOPort::<16, 4, 5, 1>::default();
    dev.bus.link_connect_dest();
    dev.fifo_bus.link_connect_dest();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    yosys_validate("mosi_fifo_port", &vlog).unwrap();
}

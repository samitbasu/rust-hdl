use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadController, FIFOWriteController};

#[derive(LogicBlock, Default)]
pub struct FIFOLink<T: Synth> {
    pub read: FIFOReadController<T>,
    pub write: FIFOWriteController<T>,
    will_transfer: Signal<Local, Bit>,
}

impl<T: Synth> Logic for FIFOLink<T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.will_transfer.next = !self.read.empty.val() & !self.write.full.val();
        self.write.data.next = self.read.data.val();
        self.read.read.next = self.will_transfer.val();
        self.write.write.next = self.will_transfer.val();
    }
}

#[test]
fn test_link_synthesizes() {
    let mut uut: FIFOLink<Bits<8>> = Default::default();
    uut.read.empty.connect();
    uut.read.almost_empty.connect();
    uut.read.data.connect();
    uut.write.full.connect();
    uut.write.almost_full.connect();
    uut.connect_all();
    yosys_validate("fifo_link", &generate_verilog(&uut)).unwrap();
}

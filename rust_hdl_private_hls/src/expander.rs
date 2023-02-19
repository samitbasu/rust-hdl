use crate::bus::{FIFOReadController, FIFOWriteController};
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct Expander<const DN: usize, const DW: usize> {
    pub bus_read: FIFOReadController<Bits<DN>>,
    pub bus_write: FIFOWriteController<Bits<DW>>,
    pub clock: Signal<In, Clock>,
    expander: FIFOExpanderN<DN, DW>,
}

impl<const DW: usize, const DN: usize> Logic for Expander<DN, DW> {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, expander);
        // Connect the HLS read bus to the expanders native signals
        self.bus_read.read.next = self.expander.read.val();
        self.expander.empty.next = self.bus_read.empty.val();
        self.expander.data_in.next = self.bus_read.data.val();
        // Connect the HLS write bus to the expanders native signals
        self.expander.full.next = self.bus_write.full.val();
        self.bus_write.data.next = self.expander.data_out.val();
        self.bus_write.write.next = self.expander.write.val();
    }
}

impl<const DW: usize, const DN: usize> Expander<DN, DW> {
    pub fn new(order: WordOrder) -> Self {
        Self {
            bus_read: Default::default(),
            bus_write: Default::default(),
            clock: Default::default(),
            expander: FIFOExpanderN::new(order),
        }
    }
}

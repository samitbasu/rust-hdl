use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadController, FIFOWriteController};
use crate::widgets::prelude::*;

#[derive(LogicBlock)]
pub struct Reducer<const DW: usize, const DN: usize> {
    pub bus_read: FIFOReadController<Bits<DW>>,
    pub bus_write: FIFOWriteController<Bits<DN>>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    reducer: FIFOReducerN<DW, DN>,
}

impl<const DW: usize, const DN: usize> Logic for Reducer<DW, DN> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clock
        clock_reset!(self, clock, reset, reducer);
        // Connect the HLS read bus to the native signals
        self.bus_read.read.next = self.reducer.read.val();
        self.reducer.empty.next = self.bus_read.empty.val();
        self.reducer.data_in.next = self.bus_read.data.val();
        // Connect the HDL write bus to the native signals
        self.reducer.full.next = self.bus_write.full.val();
        self.bus_write.data.next = self.reducer.data_out.val();
        self.bus_write.write.next = self.reducer.write.val();
    }
}

impl<const DW: usize, const DN: usize> Reducer<DW, DN> {
    pub fn new(order: WordOrder) -> Self {
        Self {
            bus_read: Default::default(),
            bus_write: Default::default(),
            clock: Default::default(),
            reset: Default::default(),
            reducer: FIFOReducerN::new(order),
        }
    }
}

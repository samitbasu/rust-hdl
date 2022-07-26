use crate::core::prelude::*;
use crate::dff_setup;
use crate::widgets::dff::DFF;

#[derive(LogicBlock)]
pub struct FIFOReducer<const DW: usize, const DN: usize, const REVERSE: bool> {
    // Data comes by reading from the source FIFO
    pub data_in: Signal<In, Bits<DW>>,
    pub read: Signal<Out, Bit>,
    pub empty: Signal<In, Bit>,
    // Data is written to the output FIFO
    pub data_out: Signal<Out, Bits<DN>>,
    pub write: Signal<Out, Bit>,
    pub full: Signal<In, Bit>,
    // This is a synchronous design.  The clock is assumed
    // to be shared with both the input and output fifos.
    pub clock: Signal<In, Clock>,
    loaded: DFF<Bit>,
    data_available: Signal<Local, Bit>,
    can_write: Signal<Local, Bit>,
    will_run: Signal<Local, Bit>,
    data_to_write: Signal<Local, Bits<DN>>,
    offset: Constant<Bits<DW>>,
    reverse: Constant<Bit>,
}

impl<const DW: usize, const DN: usize, const REVERSE: bool> Default
    for FIFOReducer<DW, DN, REVERSE>
{
    fn default() -> Self {
        assert_eq!(DW, DN * 2);
        Self {
            data_in: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            data_out: Default::default(),
            write: Default::default(),
            full: Default::default(),
            clock: Default::default(),
            loaded: Default::default(),
            data_available: Default::default(),
            can_write: Default::default(),
            will_run: Default::default(),
            data_to_write: Default::default(),
            offset: Constant::new(DN.into()),
            reverse: Constant::new(REVERSE),
        }
    }
}

impl<const DW: usize, const DN: usize, const REVERSE: bool> Logic for FIFOReducer<DW, DN, REVERSE> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the clock
        dff_setup!(self, clock, loaded);
        // Input data is available if we are loaded or if the read interface is not empty
        self.data_available.next = self.loaded.q.val() || !self.empty.val();
        // Output space is available if the write interface is not full and we have data available
        self.can_write.next = self.data_available.val() && !self.full.val();
        // This signal indicates the reducer will actually do something
        self.will_run.next = self.data_available.val() && self.can_write.val();
        // If data is available we select which piece of the input based on the loaded flag
        if self.reverse.val() ^ self.loaded.q.val() {
            self.data_to_write.next = self.data_in.val().get_bits::<DN>(self.offset.val().into());
        } else {
            self.data_to_write.next = self.data_in.val().get_bits::<DN>(0);
        }
        // The input to the output fifo is always data_to_write (although it may not be valid)
        self.data_out.next = self.data_to_write.val();
        // If we have data and we can write, then write!
        self.write.next = self.can_write.val();
        // Toggle loaded if we have data available and can write
        self.loaded.d.next = self.loaded.q.val() ^ self.will_run.val();
        // Advance the read interface if it is not empty and we wont be loaded
        self.read.next = self.loaded.q.val() && self.will_run.val() && !self.empty.val();
    }
}

#[test]
fn fifo_reducer_is_synthesizable() {
    let mut dev: FIFOReducer<8, 4, false> = Default::default();
    dev.connect_all();
    yosys_validate("fifo_reducer", &generate_verilog(&dev)).unwrap();
}

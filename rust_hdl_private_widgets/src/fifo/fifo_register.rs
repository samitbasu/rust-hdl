use rust_hdl_private_core::prelude::*;

use crate::{dff::DFF, dff_setup};

// A synchronous FIFO of depth 1, backed by a pair of registers
#[derive(LogicBlock, Default)]
pub struct RegisterFIFO<T: Synth> {
    pub data_in: Signal<In, T>,
    pub data_out: Signal<Out, T>,
    pub write: Signal<In, Bit>,
    pub read: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub empty: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    value: DFF<T>,
    filled: DFF<Bit>,
    error: DFF<Bit>,
}

impl<T: Synth> Logic for RegisterFIFO<T> {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, value, filled, error);
        // There are two states to consider.  The first is the
        // empty state (no internal data)
        if self.write.val() {
            self.value.d.next = self.data_in.val();
        }
        self.data_out.next = self.value.q.val();
        self.full.next = self.filled.q.val() & !self.read.val();
        self.empty.next = !self.filled.q.val();
        if !self.filled.q.val() {
            // We are empty.  This means our empty flag is true, and
            // should be no read.
            if self.write.val() {
                self.value.d.next = self.data_in.val();
                self.filled.d.next = true;
            }
            // If we have a read with no data, this is an error condition !
            if self.read.val() {
                self.error.d.next = true;
            }
        } else {
            // We have data.  It is possible we can get both a
            // read and a write (pipeline)
            // If we have a write with no read, this is an error condition!
            if self.write.val() & !self.read.val() {
                self.error.d.next = true;
            }
            // If we have a read with no write, then we will be empty next cycle
            if self.read.val() & !self.write.val() {
                self.filled.d.next = false;
            }
        }
    }
}

#[test]
fn test_register_fifo_is_synthesizable() {
    let mut uut = RegisterFIFO::<Bits<16>>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fifo_reg", &vlog).unwrap();
}

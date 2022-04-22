use crate::core::prelude::*;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct ResetSynchronizer {
    pub reset_in: Signal<In, Reset>,
    pub clock: Signal<In, Clock>,
    pub reset_out: Signal<Out, Reset>,
    dff0: DFF<Bit>,
    dff1: DFF<Bit>,
}

// From http://www.sunburst-design.com/papers/CummingsSNUG2003Boston_Resets.pdf
// Adapted for a positive reset.
impl Logic for ResetSynchronizer {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset_in, dff0, dff1);
        self.dff0.d.next = true;
        self.dff1.d.next = self.dff0.q.val();
        if self.dff1.q.val() {
            self.reset_out.next = NO_RESET;
        } else {
            self.reset_out.next = RESET;
        }
    }
}

#[test]
fn reset_synchronizer_is_synthesizable() {
    let mut uut = ResetSynchronizer::default();
    uut.clock.connect();
    uut.reset_in.connect();
    uut.connect_all();
    yosys_validate("reset_synch", &generate_verilog(&uut)).unwrap();
}

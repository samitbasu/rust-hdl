use crate::core::prelude::*;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct AutoReset {
    pub reset: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    dff: DFF<Bits<8>>,
}

impl Logic for AutoReset {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff.clock.next = self.clock.val();
        self.dff.d.next = self.dff.q.val();
        self.reset.next = false.into();
        if !self.dff.q.val().all() {
            self.dff.d.next = self.dff.q.val() + 1_usize;
            self.reset.next = true.into();
        }
    }
}

#[test]
fn test_synch_reset_synchronizes() {
    let mut uut = AutoReset::default();
    uut.connect_all();
    yosys_validate("sync_reset", &generate_verilog(&uut)).unwrap();
}

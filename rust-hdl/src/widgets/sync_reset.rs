use crate::core::prelude::*;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, LogicBlock, Default)]
pub struct SyncReset {
    pub reset: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    dff: DFF<Bit>,
}

impl Logic for SyncReset {
    #[hdl_gen]
    fn update(&mut self) {
        self.reset.next = false;
        self.dff.d.next = self.dff.q.val();
        self.dff.clk.next = self.clock.val();
        if !self.dff.q.val() {
            self.reset.next = true;
            self.dff.d.next = true;
        }
    }
}

#[test]
fn test_synch_reset_synchronizes() {
    let mut uut = SyncReset::default();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("sync_reset", &generate_verilog(&uut)).unwrap();
}

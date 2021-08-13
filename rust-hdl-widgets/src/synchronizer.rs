use crate::dff::DFF;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

#[derive(LogicBlock, Default)]
pub struct BitSynchronizer<F: Domain, T: Domain> {
    pub sig_in: Signal<In, Bit, F>,
    pub sig_out: Signal<Out, Bit, T>,
    pub clock: Signal<In, Clock, T>,
    dff0: DFF<Bit, T>,
    dff1: DFF<Bit, T>,
}

impl<F: Domain, T: Domain> Logic for BitSynchronizer<F, T> {
    #[hdl_gen]
    fn update(&mut self) {
        self.dff0.clk.next = self.clock.val();
        self.dff1.clk.next = self.clock.val();

        // Note!  The raw() call here is needed because we
        // _are_ crossing clock domains.  This should be one
        // of the few places you can call it safely!
        self.dff0.d.next = self.sig_in.val().raw().into();
        self.dff1.d.next = self.dff0.q.val();
        self.sig_out.next = self.dff1.q.val();
    }
}

make_domain!(MHz1, 1_000_000);

#[test]
fn sync_is_synthesizable() {
    rust_hdl_synth::top_wrap!(BitSynchronizer<MHz1, MHz1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.sig_in.connect();
    dev.connect_all();
    yosys_validate("sync", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}

//#[derive(Copy, Clone, LogicState)]

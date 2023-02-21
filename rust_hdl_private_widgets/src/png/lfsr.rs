use crate::{dff_setup, prelude::DFFWithInit};
use rust_hdl_private_core::prelude::*;

// Adopted from Alchitry.com Lucid module `pn_gen`
// This version does not provide seed setting.  It generates a fixed sequence.
#[derive(LogicBlock)]
pub struct LFSRSimple {
    pub clock: Signal<In, Clock>,
    pub strobe: Signal<In, Bit>,
    pub num: Signal<Out, Bits<32>>,
    x: DFFWithInit<Bits<32>>,
    y: DFFWithInit<Bits<32>>,
    z: DFFWithInit<Bits<32>>,
    w: DFFWithInit<Bits<32>>,
    t: Signal<Local, Bits<32>>,
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for LFSRSimple {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            strobe: Default::default(),
            num: Default::default(),
            t: Default::default(),
            x: DFFWithInit::new((SEED & 0xFFFF_FFFF_u128).to_bits()),
            y: DFFWithInit::new(((SEED >> 32) & 0xFFFF_FFFF_u128).to_bits()),
            z: DFFWithInit::new(((SEED >> 64) & 0xFFFF_FFFF_u128).to_bits()),
            w: DFFWithInit::new(((SEED >> 96) & 0xFFFF_FFFF_u128).to_bits()),
        }
    }
}

impl Logic for LFSRSimple {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, x, y, z, w);
        self.num.next = self.w.q.val();
        self.t.next = self.x.q.val() ^ (self.x.q.val() << 11);
        if self.strobe.val() {
            self.x.d.next = self.y.q.val();
            self.y.d.next = self.z.q.val();
            self.z.d.next = self.w.q.val();
            self.w.d.next =
                self.w.q.val() ^ (self.w.q.val() >> 19) ^ self.t.val() ^ (self.t.val() >> 8);
        }
    }
}

#[test]
fn test_lfsr_simple_synthesizes() {
    let mut uut = LFSRSimple::default();
    uut.connect_all();
    yosys_validate("lfsr", &generate_verilog(&uut)).unwrap();
}

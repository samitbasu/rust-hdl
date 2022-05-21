use crate::core::prelude::*;
use crate::dff_setup;
use crate::widgets::dff::DFF;

// Adopted from Alchitry.com Lucid module `pn_gen`
// This version does not provide seed setting.  It generates a fixed sequence.
#[derive(LogicBlock)]
pub struct LFSRSimple {
    pub clock: Signal<In, Clock>,
    pub strobe: Signal<In, Bit>,
    pub num: Signal<Out, Bits<32>>,
    x: DFF<Bits<32>>,
    y: DFF<Bits<32>>,
    z: DFF<Bits<32>>,
    w: DFF<Bits<32>>,
    t: Signal<Local, Bits<32>>,
    x_init: Constant<Bits<32>>,
    y_init: Constant<Bits<32>>,
    z_init: Constant<Bits<32>>,
    w_init: Constant<Bits<32>>,
    boot: DFF<Bit>,
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for LFSRSimple {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            strobe: Default::default(),
            num: Default::default(),
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            w: Default::default(),
            t: Default::default(),
            x_init: Constant::new((SEED & 0xFFFF_FFFF_u128).into()),
            y_init: Constant::new(((SEED >> 32) & 0xFFFF_FFFF_u128).into()),
            z_init: Constant::new(((SEED >> 64) & 0xFFFF_FFFF_u128).into()),
            w_init: Constant::new(((SEED >> 96) & 0xFFFF_FFFF_u128).into()),
            boot: Default::default(),
        }
    }
}

impl Logic for LFSRSimple {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, x, y, z, w, boot);
        self.num.next = self.w.q.val();
        self.t.next = self.x.q.val() ^ (self.x.q.val() << 11_usize);
        if !self.boot.q.val() {
            self.boot.d.next = true;
            self.x.d.next = self.x_init.val();
            self.y.d.next = self.y_init.val();
            self.z.d.next = self.z_init.val();
            self.w.d.next = self.w_init.val();
        }
        if self.strobe.val() & self.boot.q.val() {
            self.x.d.next = self.y.q.val();
            self.y.d.next = self.z.q.val();
            self.z.d.next = self.w.q.val();
            self.w.d.next = self.w.q.val()
                ^ (self.w.q.val() >> 19_usize)
                ^ self.t.val()
                ^ (self.t.val() >> 8_usize);
        }
    }
}

#[test]
fn test_lfsr_simple_synthesizes() {
    let mut uut = LFSRSimple::default();
    uut.connect_all();
    yosys_validate("lfsr", &generate_verilog(&uut)).unwrap();
}

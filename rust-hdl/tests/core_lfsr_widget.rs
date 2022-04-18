use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

struct Xorshift128State {
    x: [u32; 4],
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for Xorshift128State {
    fn default() -> Self {
        Self {
            x: [
                ((SEED >> 96) & 0xFFFF_FFFF_u128) as u32,
                ((SEED >> 64) & 0xFFFF_FFFF_u128) as u32,
                ((SEED >> 32) & 0xFFFF_FFFF_u128) as u32,
                ((SEED >> 0) & 0xFFFF_FFFF_u128) as u32,
            ],
        }
    }
}

impl Xorshift128State {
    fn get(&mut self) -> u32 {
        let ret = self.x[0];
        let mut t = self.x[3];
        let s = self.x[0];
        self.x[3] = self.x[2];
        self.x[2] = self.x[1];
        self.x[1] = s;
        t ^= t << 11;
        t ^= t >> 8;
        self.x[0] = t ^ s ^ (s >> 19);
        ret
    }
}

#[test]
fn test_lfsr_operation() {
    let mut uut = LFSRSimple::default();
    uut.clock.connect();
    uut.strobe.connect();
    uut.reset.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<LFSRSimple>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<LFSRSimple>| {
        let mut x = sim.init()?;
        let mut lf = Xorshift128State::default();
        reset_sim!(sim, clock, reset, x);
        wait_clock_cycles!(sim, clock, x, 10);
        for _ in 0..1000 {
            sim_assert_eq!(sim, x.num.val().index() as u32, lf.get(), x);
            x.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100_000, &vcd_path!("lfsr.vcd"))
        .unwrap();
}

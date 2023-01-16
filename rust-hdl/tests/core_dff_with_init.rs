use rust_hdl::prelude::*;

#[derive(LogicBlock)]
struct DFFWithNonzeroInit {
    clock: Signal<In, Clock>,
    dff: DFFWithInit<Bits<8>>,
    pub count: Signal<Out, Bits<8>>,
}

impl Default for DFFWithNonzeroInit {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            dff: DFFWithInit::new(42.into()),
            count: Default::default(),
        }
    }
}

impl Logic for DFFWithNonzeroInit {
    #[hdl_gen]
    fn update(&mut self) {
        dff_setup!(self, clock, dff);
        self.count.next = self.dff.q.val();
        self.dff.d.next = self.dff.q.val() + 1;
    }
}

#[test]
fn test_dff_with_nonzero_init() {
    let mut uut = DFFWithNonzeroInit::default();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<DFFWithNonzeroInit>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<DFFWithNonzeroInit>| {
        let mut x = sim.init()?;
        sim_assert_eq!(sim, x.count.val(), 42, x);
        for i in 0..12 {
            wait_clock_cycle!(sim, clock, x);
            sim_assert_eq!(sim, x.count.val(), 42 + i + 1, x);
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 10000, &vcd_path!("dff_non_zero.vcd"))
        .unwrap()
}

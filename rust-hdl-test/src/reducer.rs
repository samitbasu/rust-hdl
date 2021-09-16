use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock, Default)]
struct ReducerTest {
    pub clock: Signal<In, Clock>,
    pub fifo_in: SynchronousFIFO<Bits<32>, 4, 5, 1>,
    pub fifo_out: SynchronousFIFO<Bits<4>, 8, 9, 1>,
    pub redux: FIFOReducerN<32, 4, true>,
}

impl Logic for ReducerTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo_in.clock.next = self.clock.val();
        self.fifo_out.clock.next = self.clock.val();
        self.redux.clock.next = self.clock.val();
        self.redux.empty.next = self.fifo_in.empty.val();
        self.redux.data_in.next = self.fifo_in.data_out.val();
        self.fifo_in.read.next = self.redux.read.val();
        self.redux.full.next = self.fifo_out.full.val();
        self.fifo_out.data_in.next = self.redux.data_out.val();
        self.fifo_out.write.next = self.redux.write.val();
    }
}

#[test]
fn test_reducer_works() {
    let mut uut = ReducerTest::default();
    uut.clock.connect();
    uut.fifo_in.data_in.connect();
    uut.fifo_in.write.connect();
    uut.fifo_out.read.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ReducerTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<ReducerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [0xDEADBEEF_u32, 0xCAFEBABE] {
            x = sim.watch(|x| !x.fifo_in.full.val(), x)?;
            x.fifo_in.data_in.next = datum.into();
            x.fifo_in.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_in.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ReducerTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [
            0xD_u32, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF, 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE,
        ] {
            x = sim.watch(|x| !x.fifo_out.empty.val(), x)?;
            sim_assert!(sim, x.fifo_out.data_out.val() == Bits::<4>::from(datum), x);
            x.fifo_out.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_out.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create("reducern.vcd").unwrap(),
    )
    .unwrap()
}

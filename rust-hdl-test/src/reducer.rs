use rust_hdl_core::prelude::*;
use rust_hdl_widgets::fifo_expander_n::WordOrder;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
struct ReducerTest {
    pub clock: Signal<In, Clock>,
    pub fifo_in: SynchronousFIFO<Bits<32>, 4, 5, 1>,
    pub fifo_out: SynchronousFIFO<Bits<4>, 8, 9, 1>,
    pub redux: FIFOReducerN<32, 4>,
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

impl ReducerTest {
    pub fn new(order: WordOrder) -> Self {
        Self {
            clock: Default::default(),
            fifo_in: Default::default(),
            fifo_out: Default::default(),
            redux: FIFOReducerN::new(order),
        }
    }
}

#[test]
fn test_reducer_works() {
    let mut uut = ReducerTest::new(WordOrder::MostSignificantFirst);
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
    sim.run(Box::new(uut), 100_000).unwrap()
}

#[test]
fn test_reducer_works_least_sig_word_first() {
    let mut uut = ReducerTest::new(WordOrder::LeastSignificantFirst);
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
        for datum in [0xFEEBDAED_u32, 0xEBABEFAC] {
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
    sim.run(Box::new(uut), 100_000).unwrap()
}

declare_narrowing_fifo!(Slim, 32, 16, 4, 256);

#[derive(LogicBlock)]
struct SlimTest {
    pub clock: Signal<In, Clock>,
    pub fifo: Slim,
}

impl Logic for SlimTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.write_clock.next = self.clock.val();
        self.fifo.read_clock.next = self.clock.val();
    }
}

impl SlimTest {
    pub fn new(order: WordOrder) -> Self {
        Self {
            clock: Default::default(),
            fifo: Slim::new(order),
        }
    }
}

#[test]
fn test_slim_works() {
    let mut uut = SlimTest::new(WordOrder::MostSignificantFirst);
    uut.clock.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.fifo.read.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SlimTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SlimTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [0xDEADBEEF_u32, 0xCAFEBABE] {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = datum.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<SlimTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [
            0xD_u32, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF, 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE,
        ] {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert!(sim, x.fifo.data_out.val() == Bits::<4>::from(datum), x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 100_000).unwrap()
}

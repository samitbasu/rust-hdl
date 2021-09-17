use rust_hdl_core::prelude::*;
use rust_hdl_widgets::fifo_expander_n::WordOrder;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
struct ExpanderTest {
    pub clock: Signal<In, Clock>,
    pub fifo_in: SynchronousFIFO<Bits<4>, 8, 9, 1>,
    pub fifo_out: SynchronousFIFO<Bits<32>, 4, 5, 1>,
    pub xpand: FIFOExpanderN<4, 32>,
}

impl Logic for ExpanderTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo_in.clock.next = self.clock.val();
        self.fifo_out.clock.next = self.clock.val();
        self.xpand.clock.next = self.clock.val();
        self.xpand.empty.next = self.fifo_in.empty.val();
        self.xpand.data_in.next = self.fifo_in.data_out.val();
        self.fifo_in.read.next = self.xpand.read.val();
        self.xpand.full.next = self.fifo_out.full.val();
        self.fifo_out.data_in.next = self.xpand.data_out.val();
        self.fifo_out.write.next = self.xpand.write.val();
    }
}

impl ExpanderTest {
    pub fn new(word_order: WordOrder) -> Self {
        Self {
            clock: Default::default(),
            fifo_in: Default::default(),
            fifo_out: Default::default(),
            xpand: FIFOExpanderN::new(word_order),
        }
    }
}

#[test]
fn test_expander_works() {
    let mut uut = ExpanderTest::new(WordOrder::MostSignificantFirst);
    uut.clock.connect();
    uut.fifo_in.data_in.connect();
    uut.fifo_in.write.connect();
    uut.fifo_out.read.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ExpanderTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<ExpanderTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [
            0xD_u32, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF, 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE,
        ] {
            x = sim.watch(|x| !x.fifo_in.full.val(), x)?;
            x.fifo_in.data_in.next = datum.into();
            x.fifo_in.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_in.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ExpanderTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [0xDEADBEEF_u32, 0xCAFEBABE] {
            x = sim.watch(|x| !x.fifo_out.empty.val(), x)?;
            sim_assert!(sim, x.fifo_out.data_out.val() == Bits::<32>::from(datum), x);
            x.fifo_out.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_out.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create("expandern.vcd").unwrap(),
    )
    .unwrap()
}

#[test]
fn test_expander_works_with_lsw_first() {
    let mut uut = ExpanderTest::new(WordOrder::LeastSignificantFirst);
    uut.clock.connect();
    uut.fifo_in.data_in.connect();
    uut.fifo_in.write.connect();
    uut.fifo_out.read.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<ExpanderTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<ExpanderTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [
            0xD_u32, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF, 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE,
        ] {
            x = sim.watch(|x| !x.fifo_in.full.val(), x)?;
            x.fifo_in.data_in.next = datum.into();
            x.fifo_in.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_in.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<ExpanderTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [0xFEEBDAED_u32, 0xEBABEFAC_u32] {
            x = sim.watch(|x| !x.fifo_out.empty.val(), x)?;
            sim_assert!(sim, x.fifo_out.data_out.val() == Bits::<32>::from(datum), x);
            x.fifo_out.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo_out.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create("expandern_lsw.vcd").unwrap(),
    )
    .unwrap()
}

declare_expanding_fifo!(Fatten, 4, 256, 32, 16);

#[derive(LogicBlock)]
struct FattenTest {
    pub clock: Signal<In, Clock>,
    pub fifo: Fatten,
}

impl Logic for FattenTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.read_clock.next = self.clock.val();
        self.fifo.write_clock.next = self.clock.val();
    }
}

impl FattenTest {
    pub fn new(word_order: WordOrder) -> Self {
        Self {
            clock: Default::default(),
            fifo: Fatten::new(word_order),
        }
    }
}

#[test]
fn test_fatten_works() {
    let mut uut = FattenTest::new(WordOrder::MostSignificantFirst);
    uut.clock.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.fifo.read.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<FattenTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<FattenTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [
            0xD_u32, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF, 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE,
        ] {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = datum.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<FattenTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for datum in [0xDEADBEEF_u32, 0xCAFEBABE] {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert!(sim, x.fifo.data_out.val() == Bits::<32>::from(datum), x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create("fattenn.vcd").unwrap(),
    )
    .unwrap()
}

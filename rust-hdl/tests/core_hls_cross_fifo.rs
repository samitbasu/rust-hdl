use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

mod test_common;
use crate::test_common::fifo_tester::bursty_vec;
use test_common::fifo_tester::{LazyFIFOFeeder, LazyFIFOReader};

#[derive(LogicBlock)]
struct CrossWidenTestFixture {
    feeder: LazyFIFOFeeder<Bits<4>, 12>,
    cross: CrossWiden<4, 5, 6, 16, 3, 4>,
    reader: LazyFIFOReader<Bits<16>, 10>,
    clock: Signal<In, Clock>,
    reset: Signal<In, Reset>,
}

impl Logic for CrossWidenTestFixture {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, feeder, reader);
        FIFOWriteController::<Bits<4>>::join(&mut self.feeder.bus, &mut self.cross.narrow_bus);
        FIFOReadResponder::<Bits<16>>::join(&mut self.cross.wide_bus, &mut self.reader.bus);
        self.cross.wide_clock.next = self.clock.val();
        self.cross.wide_reset.next = self.reset.val();
        self.cross.narrow_clock.next = self.clock.val();
        self.cross.narrow_reset.next = self.reset.val();
    }
}

impl Default for CrossWidenTestFixture {
    fn default() -> Self {
        let data1 = (0..256)
            .map(|_| Bits::<16>::from(rand::thread_rng().gen::<u16>()))
            .collect::<Vec<_>>();
        let mut data2 = vec![];
        for x in &data1 {
            for offset in &[0, 4, 8, 12] {
                data2.push(x.get_bits::<4>(*offset));
            }
        }
        Self {
            feeder: LazyFIFOFeeder::new(&data2, &bursty_vec(1024)),
            cross: CrossWiden::new(WordOrder::LeastSignificantFirst),
            reader: LazyFIFOReader::new(&data1, &bursty_vec(256)),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

#[test]
fn test_cross_widen_test_fixture() {
    let mut uut = CrossWidenTestFixture::default();
    uut.clock.connect();
    uut.feeder.start.connect();
    uut.reader.start.connect();
    uut.reset.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<CrossWidenTestFixture>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<CrossWidenTestFixture>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        wait_clock_true!(sim, clock, x);
        x.feeder.start.next = true;
        x.reader.start.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.feeder.start.next = false;
        x.reader.start.next = false;
        x = sim.watch(|x| x.feeder.done.val() & x.reader.done.val(), x)?;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.reader.error.val(), x);
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 100_000, &mut vcd);
    std::fs::write(vcd_path!("cross_widen_hls.vcd"), vcd).unwrap();
    ret.unwrap();
}

#[derive(LogicBlock)]
struct CrossNarrowTestFixture {
    feeder: LazyFIFOFeeder<Bits<16>, 10>,
    cross: CrossNarrow<16, 3, 4, 4, 5, 6>,
    reader: LazyFIFOReader<Bits<4>, 12>,
    clock: Signal<In, Clock>,
    reset: Signal<In, Reset>,
}

impl Logic for CrossNarrowTestFixture {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, feeder, reader);
        FIFOWriteController::<Bits<16>>::join(&mut self.feeder.bus, &mut self.cross.wide_bus);
        FIFOReadResponder::<Bits<4>>::join(&mut self.cross.narrow_bus, &mut self.reader.bus);
        self.cross.wide_clock.next = self.clock.val();
        self.cross.wide_reset.next = self.reset.val();
        self.cross.narrow_clock.next = self.clock.val();
        self.cross.narrow_reset.next = self.reset.val();
    }
}

impl Default for CrossNarrowTestFixture {
    fn default() -> Self {
        let data1 = (0..256)
            .map(|_| Bits::<16>::from(rand::thread_rng().gen::<u16>()))
            .collect::<Vec<_>>();
        let mut data2 = vec![];
        for x in &data1 {
            for offset in &[0, 4, 8, 12] {
                data2.push(x.get_bits::<4>(*offset));
            }
        }
        Self {
            feeder: LazyFIFOFeeder::new(&data1, &bursty_vec(256)),
            cross: CrossNarrow::new(WordOrder::LeastSignificantFirst),
            reader: LazyFIFOReader::new(&data2, &bursty_vec(1024)),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

#[test]
fn test_cross_narrow_test_fixture() {
    let mut uut = CrossNarrowTestFixture::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.feeder.start.connect();
    uut.reader.start.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<CrossNarrowTestFixture>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<CrossNarrowTestFixture>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        wait_clock_true!(sim, clock, x);
        x.feeder.start.next = true;
        x.reader.start.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.feeder.start.next = false;
        x.reader.start.next = false;
        x = sim.watch(|x| x.feeder.done.val() & x.reader.done.val(), x)?;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.reader.error.val(), x);
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 100_000, &mut vcd);
    std::fs::write(vcd_path!("cross_narrow_hls.vcd"), vcd).unwrap();
    ret.unwrap();
}

use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::test_helpers;
use rust_hdl::widgets::test_helpers::{LazyFIFOFeeder, LazyFIFOReader};

#[derive(LogicBlock, Default)]
struct HLSFIFOTest {
    fifo: AsyncFIFO<Bits<8>, 3, 4, 1>,
    clock: Signal<In, Clock>,
}

impl Logic for HLSFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.write_clock.next = self.clock.val();
        self.fifo.read_clock.next = self.clock.val();
    }
}

#[test]
fn test_hls_fifo_works() {
    let mut uut = HLSFIFOTest::default();
    uut.fifo.bus_write.write.connect();
    uut.fifo.bus_write.data.connect();
    uut.fifo.bus_read.read.connect();
    uut.fifo.bus_read.data.connect();
    uut.connect_all();
    let rdata = (0..128)
        .map(|_| rand::random::<u8>().to_bits())
        .collect::<Vec<_>>();
    let data = rdata.clone();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<HLSFIFOTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<HLSFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.bus_write.full.val(), x)?;
            x.fifo.bus_write.data.next = (*sample).into();
            x.fifo.bus_write.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.bus_write.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<HLSFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &data {
            x = sim.watch(|x| !x.fifo.bus_read.empty.val(), x)?;
            sim_assert!(sim, x.fifo.bus_read.data.val() == *sample, x);
            x.fifo.bus_read.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.bus_read.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("hls_fifo.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock)]
struct FIFOTestFixture {
    feeder: LazyFIFOFeeder<Bits<8>, 10>,
    fifo: SyncFIFO<Bits<8>, 4, 5, 1>,
    reader: LazyFIFOReader<Bits<8>, 10>,
    clock: Signal<In, Clock>,
}

impl Logic for FIFOTestFixture {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, feeder, fifo, reader);
        FIFOWriteController::<Bits<8>>::join(&mut self.feeder.bus, &mut self.fifo.bus_write);
        FIFOReadController::<Bits<8>>::join(&mut self.reader.bus, &mut self.fifo.bus_read);
    }
}

impl FIFOTestFixture {
    pub fn new(data: &[Bits<8>]) -> FIFOTestFixture {
        FIFOTestFixture {
            feeder: LazyFIFOFeeder::new(
                data,
                &(0..data.len())
                    .map(|_| test_helpers::bursty_rand())
                    .collect::<Vec<_>>(),
            ),
            fifo: SyncFIFO::default(),
            reader: LazyFIFOReader::new(
                data,
                &(0..data.len())
                    .map(|_| test_helpers::bursty_rand())
                    .collect::<Vec<_>>(),
            ),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_feeder_works() {
    let data = (0..256)
        .map(|_| rand::thread_rng().gen::<u8>().to_bits())
        .collect::<Vec<_>>();
    let mut uut = FIFOTestFixture::new(&data);
    uut.feeder.start.connect();
    uut.reader.start.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fifo_feed", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<FIFOTestFixture>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<FIFOTestFixture>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.feeder.start.next = true;
        x.reader.start.next = true;
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.feeder.done.val() & x.reader.done.val(), x)?;
        sim_assert!(sim, !x.reader.error.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 100_000, &mut vcd);
    std::fs::write(vcd_path!("fifo_stress_test.vcd"), vcd).unwrap();
    ret.unwrap();
}

#[derive(LogicBlock)]
struct FIFOTestFixtureAsync {
    feeder: LazyFIFOFeeder<Bits<8>, 10>,
    fifo: AsyncFIFO<Bits<8>, 4, 5, 1>,
    reader: LazyFIFOReader<Bits<8>, 10>,
    clock_write: Signal<In, Clock>,
    clock_read: Signal<In, Clock>,
}

impl Logic for FIFOTestFixtureAsync {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock_write, feeder);
        clock!(self, clock_read, reader);
        self.fifo.write_clock.next = self.clock_write.val();
        self.fifo.read_clock.next = self.clock_read.val();
        FIFOWriteController::<Bits<8>>::join(&mut self.feeder.bus, &mut self.fifo.bus_write);
        FIFOReadController::<Bits<8>>::join(&mut self.reader.bus, &mut self.fifo.bus_read);
    }
}

impl FIFOTestFixtureAsync {
    pub fn new(data: &[Bits<8>]) -> FIFOTestFixtureAsync {
        Self {
            feeder: LazyFIFOFeeder::new(
                data,
                &(0..data.len())
                    .map(|_| test_helpers::bursty_rand())
                    .collect::<Vec<_>>(),
            ),
            fifo: Default::default(),
            reader: LazyFIFOReader::new(
                data,
                &(0..data.len())
                    .map(|_| test_helpers::bursty_rand())
                    .collect::<Vec<_>>(),
            ),
            clock_write: Default::default(),
            clock_read: Default::default(),
        }
    }
}

#[test]
fn test_feeder_async_works() {
    let data = (0..256)
        .map(|_| rand::thread_rng().gen::<u8>().to_bits())
        .collect::<Vec<_>>();
    let mut uut = FIFOTestFixtureAsync::new(&data);
    uut.clock_read.connect();
    uut.clock_write.connect();
    uut.feeder.start.connect();
    uut.reader.start.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("fifo_feed_async", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<FIFOTestFixtureAsync>| {
        x.clock_read.next = !x.clock_read.val()
    });
    sim.add_clock(4, |x: &mut Box<FIFOTestFixtureAsync>| {
        x.clock_write.next = !x.clock_write.val()
    });
    sim.add_testbench(move |mut sim: Sim<FIFOTestFixtureAsync>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock_write, x);
        x.feeder.start.next = true;
        wait_clock_cycle!(sim, clock_write, x);
        x.feeder.start.next = false;
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<FIFOTestFixtureAsync>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock_read, x);
        x.reader.start.next = true;
        wait_clock_cycle!(sim, clock_read, x);
        x.reader.start.next = false;
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<FIFOTestFixtureAsync>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.feeder.done.val() & x.reader.done.val(), x)?;
        sim_assert!(sim, !x.reader.error.val(), x);
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 100_000, &mut vcd);
    std::fs::write(vcd_path!("fifo_stress_test_async.vcd"), vcd).unwrap();
    ret.unwrap();
}

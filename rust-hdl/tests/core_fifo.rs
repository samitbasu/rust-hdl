use rand::Rng;
use rust_hdl::prelude::*;

#[derive(LogicBlock, Default)]
struct SyncVecTest {
    pub clock1: Signal<In, Clock>,
    pub clock2: Signal<In, Clock>,
    pub sender: SyncSender<Bits<8>>,
    pub recv: SyncReceiver<Bits<8>>,
}

impl Logic for SyncVecTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock1, sender);
        clock!(self, clock2, recv);
        self.sender.ack_in.next = self.recv.ack_out.val();
        self.recv.flag_in.next = self.sender.flag_out.val();
        self.recv.sig_cross.next = self.sender.sig_cross.val();
    }
}

#[test]
fn test_sync_vec() {
    let mut uut = SyncVecTest::default();
    uut.sender.sig_in.connect();
    uut.sender.send.connect();
    uut.connect_all();
    yosys_validate("sync", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SyncVecTest>| {
        x.clock2.next = !x.clock2.val()
    });
    sim.add_clock(9, |x: &mut Box<SyncVecTest>| {
        x.clock1.next = !x.clock1.val()
    });
    sim.add_testbench(move |mut sim: Sim<SyncVecTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock1, x);
        for i in 0..150 {
            x.sender.sig_in.next = i.into();
            x.sender.send.next = true;
            wait_clock_cycle!(sim, clock1, x);
            x.sender.send.next = false;
            x = sim.watch(|x| !x.sender.busy.val(), x)?;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<SyncVecTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock2, x);
        for i in 0..150 {
            x = sim.watch(|x| x.recv.update.val(), x)?;
            sim_assert!(sim, x.recv.sig_out.val().eq(&i), x);
            wait_clock_cycle!(sim, clock2, x);
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("vsync.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_vector_synchronizer() {
    type TestCircuit = TopWrap<VectorSynchronizer<Bits<8>>>;
    let mut dev: TestCircuit = TopWrap::new(VectorSynchronizer::default());
    dev.uut.clock_in.connect();
    dev.uut.clock_out.connect();
    dev.uut.send.connect();
    dev.uut.sig_in.connect();
    dev.connect_all();
    yosys_validate("vsync", &generate_verilog(&dev)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<TestCircuit>| {
        x.uut.clock_out.next = !x.uut.clock_out.val()
    });
    sim.add_clock(9, |x: &mut Box<TestCircuit>| {
        x.uut.clock_in.next = !x.uut.clock_in.val()
    });
    sim.add_testbench(move |mut sim: Sim<TestCircuit>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.uut.clock_in.val().clk, x)?;
        for i in 0..150 {
            x.uut.sig_in.next = i.into();
            x.uut.send.next = true;
            x = sim.watch(|x| !x.uut.clock_in.val().clk, x)?;
            x = sim.watch(|x| x.uut.clock_in.val().clk, x)?;
            x.uut.send.next = false;
            x = sim.watch(|x| !x.uut.busy.val(), x)?;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<TestCircuit>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.uut.clock_out.val().clk, x)?;
        for i in 0..150 {
            x = sim.watch(|x| x.uut.update.val(), x)?;
            sim_assert!(sim, x.uut.sig_out.val().eq(&i), x);
            x = sim.watch(|x| !x.uut.clock_out.val().clk, x)?;
            x = sim.watch(|x| x.uut.clock_out.val().clk, x)?;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(dev),
        100_000,
        std::fs::File::create(vcd_path!("vsync2.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct SynchronousFIFOTest {
    pub clock: Signal<In, Clock>,
    pub fifo: SynchronousFIFO<Bits<16>, 4, 5, 4>,
}

impl Logic for SynchronousFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, fifo);
    }
}

#[test]
fn test_sync_fifo_read_behavior_bug() {
    let mut uut = SynchronousFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SynchronousFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0..8 {
            x.fifo.data_in.next = counter.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        for counter in 0..6 {
            sim_assert!(sim, x.fifo.data_out.val() == counter, x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        wait_clock_cycle!(sim, clock, x, 2);
        for counter in 6..8 {
            sim_assert!(sim, x.fifo.data_out.val() == counter, x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 10_000, &vcd_path!("fifo_bug_test.vcd"))
        .unwrap()
}

declare_sync_fifo!(BigFIFO, Bits<8>, 1024, 256);

#[derive(LogicBlock, Default)]
struct BigFIFOTest {
    pub clock: Signal<In, Clock>,
    pub fifo: BigFIFO,
}

impl Logic for BigFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, fifo);
    }
}

#[test]
fn test_almost_empty_is_accurate_in_large_fifo() {
    let mut uut = BigFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<BigFIFOTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<BigFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0..1024 {
            x.fifo.data_in.next = (counter % 0xFF).into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        sim_assert!(sim, x.fifo.full.val(), x);
        sim_assert!(sim, !x.fifo.almost_empty.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_to_file(
        Box::new(uut),
        50_000,
        &vcd_path!("fifo_big_almost_empty.vcd"),
    )
    .unwrap();
}

#[test]
fn test_almost_empty_is_accurate_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SynchronousFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0..4 {
            x.fifo.data_in.next = counter.into();
            x.fifo.write.next = true;
            sim_assert!(sim, x.fifo.almost_empty.val(), x);
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.fifo.almost_empty.val(), x);
        let mut drain = 0;
        while !x.fifo.empty.val() {
            drain += 1;
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim_assert!(sim, drain == 4, x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        10_000,
        std::fs::File::create(vcd_path!("fifo_almost_empty.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_fifo_can_be_filled_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    yosys_validate("fifo_3", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..16)
        .map(|_| rand::random::<u16>().to_bits())
        .collect::<Vec<_>>();
    sim.add_clock(5, |x: &mut Box<SynchronousFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x.fifo.data_in.next = (*sample).into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        sim_assert!(sim, !x.fifo.overflow.val(), x);
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert!(sim, x.fifo.data_out.val().eq(sample), x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        10_000,
        std::fs::File::create(vcd_path!("fifo_fill.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_fifo_works_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    yosys_validate("fifo_4", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..1024)
        .map(|_| rand::random::<u16>().to_bits())
        .collect::<Vec<_>>();
    let rdata_read = rdata.clone();
    sim.add_clock(5, |x: &mut Box<SynchronousFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = (*sample).into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.underflow.val(), x);
        sim_assert!(sim, !x.fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert!(sim, x.fifo.data_out.val().eq(sample), x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.underflow.val(), x);
        sim_assert!(sim, !x.fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("fifo.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct AsynchronousFIFOTest {
    pub read_clock: Signal<In, Clock>,
    pub write_clock: Signal<In, Clock>,
    pub fifo: AsynchronousFIFO<Bits<16>, 4, 5, 4>,
}

impl Logic for AsynchronousFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.write_clock.next = self.write_clock.val();
        self.fifo.read_clock.next = self.read_clock.val();
    }
}

#[test]
fn test_fifo_timing() {
    let uut = AsynchronousFIFOTest::default();
    check_timing(&uut);
}

#[test]
fn test_fifo_works_asynchronous_fifo() {
    let mut uut = AsynchronousFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    yosys_validate("fifo_5", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..1024)
        .map(|_| rand::random::<u16>().to_bits())
        .collect::<Vec<_>>();
    let rdata_read = rdata.clone();
    sim.add_clock(5, |x: &mut Box<AsynchronousFIFOTest>| {
        x.read_clock.next = !x.read_clock.val()
    });
    sim.add_clock(4, |x: &mut Box<AsynchronousFIFOTest>| {
        x.write_clock.next = !x.write_clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<AsynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, write_clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = (*sample).into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, write_clock, x);
            x.fifo.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, write_clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.underflow.val(), x);
        sim_assert!(sim, !x.fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<AsynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, read_clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert!(sim, x.fifo.data_out.val().eq(sample), x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, read_clock, x);
            x.fifo.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, read_clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.underflow.val(), x);
        sim_assert!(sim, !x.fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("afifo.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct AsyncBigFIFOTest {
    pub read_clock: Signal<In, Clock>,
    pub write_clock: Signal<In, Clock>,
    pub fifo: AsynchronousFIFO<Bits<16>, 10, 11, 256>,
}

impl Logic for AsyncBigFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.write_clock.next = self.write_clock.val();
        self.fifo.read_clock.next = self.read_clock.val();
    }
}

#[test]
fn test_almost_empty_is_accurate_in_large_async_fifo() {
    let mut uut = AsyncBigFIFOTest::default();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<AsyncBigFIFOTest>| {
        x.read_clock.next = !x.read_clock.val()
    });
    sim.add_clock(4, |x: &mut Box<AsyncBigFIFOTest>| {
        x.write_clock.next = !x.write_clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<AsyncBigFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, write_clock, x);
        for counter in 0..1024 {
            x.fifo.data_in.next = counter.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, write_clock, x);
            x.fifo.write.next = false;
        }
        sim_assert!(sim, x.fifo.full.val(), x);
        sim_assert!(sim, !x.fifo.almost_empty.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        50_000,
        std::fs::File::create(vcd_path!("fifo_big_almost_empty_async.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock, Default)]
struct ReducerFIFOTest {
    pub clock: Signal<In, Clock>,
    pub wide_fifo: SynchronousFIFO<Bits<16>, 4, 5, 4>,
    pub narrow_fifo: SynchronousFIFO<Bits<8>, 4, 5, 4>,
    pub reducer: FIFOReducer<16, 8, false>,
}

impl Logic for ReducerFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, wide_fifo, narrow_fifo, reducer);
        // Connect the reducer to the wide (upstream) FIFO
        self.reducer.data_in.next = self.wide_fifo.data_out.val();
        self.reducer.empty.next = self.wide_fifo.empty.val();
        self.wide_fifo.read.next = self.reducer.read.val();
        // Connect the reducer to the narrow (downstream) FIFO
        self.narrow_fifo.data_in.next = self.reducer.data_out.val();
        self.narrow_fifo.write.next = self.reducer.write.val();
        self.reducer.full.next = self.narrow_fifo.full.val();
    }
}

#[test]
fn test_fifo_reducer_works() {
    let mut uut = ReducerFIFOTest::default();
    uut.wide_fifo.write.connect();
    uut.wide_fifo.data_in.connect();
    uut.narrow_fifo.read.connect();
    uut.connect_all();
    yosys_validate("fifo_5b", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..256)
        .map(|_| rand::random::<u16>().to_bits())
        .collect::<Vec<_>>();
    let mut rdata_read = vec![];
    for x in &rdata {
        rdata_read.push(x.get_bits::<8>(0));
        rdata_read.push(x.get_bits::<8>(8));
    }
    sim.add_clock(5, |x: &mut Box<ReducerFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<ReducerFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.wide_fifo.full.val(), x)?;
            x.wide_fifo.data_in.next = (*sample).into();
            x.wide_fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.wide_fifo.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.wide_fifo.underflow.val(), x);
        sim_assert!(sim, !x.wide_fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<ReducerFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.narrow_fifo.empty.val(), x)?;
            sim_assert!(sim, x.narrow_fifo.data_out.val().eq(sample), x);
            x.narrow_fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.narrow_fifo.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.narrow_fifo.underflow.val(), x);
        sim_assert!(sim, !x.narrow_fifo.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        Box::new(uut),
        100_000,
        std::fs::File::create(vcd_path!("fifo_reducer.vcd")).unwrap(),
    )
    .unwrap();
}

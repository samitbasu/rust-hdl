use rand::Rng;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;

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
        self.sender.clock.next = self.clock1.val();
        self.recv.clock.next = self.clock2.val();
        self.sender.ack_in.next = self.recv.ack_out.val();
        self.recv.flag_in.next = self.sender.flag_out.val();
        self.recv.sig_cross.next = self.sender.sig_cross.val();
    }
}

#[test]
fn test_sync_vec() {
    let mut uut = SyncVecTest::default();
    uut.clock1.connect();
    uut.sender.sig_in.connect();
    uut.clock2.connect();
    uut.sender.send.connect();
    uut.connect_all();
    yosys_validate("sync", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut SyncVecTest| x.clock2.next = !x.clock2.val());
    sim.add_clock(9, |x: &mut SyncVecTest| x.clock1.next = !x.clock1.val());
    sim.add_testbench(move |mut sim: Sim<SyncVecTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock1, x);
        for i in 0..150 {
            x.sender.sig_in.next = (i as u32).into();
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
    sim.run_traced(uut, 100_000, std::fs::File::create("vsync.vcd").unwrap())
        .unwrap();
}

#[test]
fn test_vector_synchronizer() {
    rust_hdl_synth::top_wrap!(VectorSynchronizer<Bits<8>>, TestCircuit);
    let mut dev: TestCircuit = Default::default();
    dev.uut.clock_in.connect();
    dev.uut.clock_out.connect();
    dev.uut.send.connect();
    dev.uut.sig_in.connect();
    dev.connect_all();
    yosys_validate("vsync", &generate_verilog(&dev)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut TestCircuit| {
        x.uut.clock_out.next = !x.uut.clock_out.val()
    });
    sim.add_clock(9, |x: &mut TestCircuit| {
        x.uut.clock_in.next = !x.uut.clock_in.val()
    });
    sim.add_testbench(move |mut sim: Sim<TestCircuit>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.uut.clock_in.val().0, x)?;
        for i in 0..150 {
            x.uut.sig_in.next = (i as u32).into();
            x.uut.send.next = true;
            x = sim.watch(|x| !x.uut.clock_in.val().0, x)?;
            x = sim.watch(|x| x.uut.clock_in.val().0, x)?;
            x.uut.send.next = false;
            x = sim.watch(|x| !x.uut.busy.val(), x)?;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<TestCircuit>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.uut.clock_out.val().0, x)?;
        for i in 0..150 {
            x = sim.watch(|x| x.uut.update.val(), x)?;
            sim_assert!(sim, x.uut.sig_out.val().eq(&i), x);
            x = sim.watch(|x| !x.uut.clock_out.val().0, x)?;
            x = sim.watch(|x| x.uut.clock_out.val().0, x)?;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(dev, 100_000, std::fs::File::create("vsync.vcd").unwrap())
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
        self.fifo.clock.next = self.clock.val();
    }
}

#[test]
fn test_almost_empty_is_accurate_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.clock.connect();
    uut.fifo.read_if.read.connect();
    uut.fifo.write_if.data_in.connect();
    uut.fifo.write_if.write.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut SynchronousFIFOTest| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0_u32..4_u32 {
            x.fifo.write_if.data_in.next = counter.into();
            x.fifo.write_if.write.next = true;
            sim_assert!(sim, x.fifo.read_if.almost_empty.val(), x);
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write_if.write.next = false;
        }
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.fifo.read_if.almost_empty.val(), x);
        let mut drain = 0_u32;
        while !x.fifo.read_if.empty.val() {
            drain += 1;
            x.fifo.read_if.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read_if.read.next = false;
        }
        sim_assert!(sim, drain == 4, x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(
        uut,
        10_000,
        std::fs::File::create("fifo_almost_empty.vcd").unwrap(),
    )
    .unwrap();
}

#[test]
fn test_fifo_can_be_filled_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.clock.connect();
    uut.fifo.read_if.read.connect();
    uut.fifo.write_if.data_in.connect();
    uut.fifo.write_if.write.connect();
    uut.connect_all();
    yosys_validate("fifo_3", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..16)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    sim.add_clock(5, |x: &mut SynchronousFIFOTest| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x.fifo.write_if.data_in.next = (*sample).into();
            x.fifo.write_if.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write_if.write.next = false;
        }
        sim_assert!(sim, !x.fifo.write_if.overflow.val(), x);
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.read_if.empty.val(), x)?;
            sim_assert!(sim, x.fifo.read_if.data_out.val().eq(sample), x);
            x.fifo.read_if.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read_if.read.next = false;
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(uut, 10_000, std::fs::File::create("fifo_fill.vcd").unwrap())
        .unwrap();
}

#[test]
fn test_fifo_works_synchronous_fifo() {
    let mut uut = SynchronousFIFOTest::default();
    uut.clock.connect();
    uut.fifo.read_if.read.connect();
    uut.fifo.write_if.data_in.connect();
    uut.fifo.write_if.write.connect();
    uut.connect_all();
    yosys_validate("fifo_4", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..1024)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    let rdata_read = rdata.clone();
    sim.add_clock(5, |x: &mut SynchronousFIFOTest| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.write_if.full.val(), x)?;
            x.fifo.write_if.data_in.next = (*sample).into();
            x.fifo.write_if.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write_if.write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.read_if.underflow.val(), x);
        sim_assert!(sim, !x.fifo.write_if.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<SynchronousFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.fifo.read_if.empty.val(), x)?;
            sim_assert!(sim, x.fifo.read_if.data_out.val().eq(sample), x);
            x.fifo.read_if.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read_if.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim_assert!(sim, !x.fifo.read_if.underflow.val(), x);
        sim_assert!(sim, !x.fifo.write_if.overflow.val(), x);
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(uut, 100_000, std::fs::File::create("fifo.vcd").unwrap())
        .unwrap();
}

use rand::Rng;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;
use rust_hdl_widgets::sync_fifo::SyncFIFO;

make_domain!(Mhz1, 1_000_000);

#[derive(LogicBlock, Default)]
struct SyncFIFOTest {
    pub clock: Signal<In, Clock, Mhz1>,
    pub fifo: SyncFIFO<Bits<16>, Mhz1, 4, 4>,
}

impl Logic for SyncFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.clock.next = self.clock.val();
    }
}

#[test]
fn test_fifo_can_be_filled() {
    let mut uut = SyncFIFOTest::default();
    uut.clock.connect();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    yosys_validate("fifo", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..16)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    sim.add_clock(5, |x: &mut SyncFIFOTest| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SyncFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x.fifo.data_in.next = (*sample).into();
            x.fifo.write.next = true.into();
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false.into();
        }
        if x.fifo.overflow.val().raw() {
            return sim.halt(x);
        }
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(uut, 10_000, std::fs::File::create("fifo_fill.vcd").unwrap())
        .unwrap();
}

#[test]
fn test_fifo_works() {
    let mut uut = SyncFIFOTest::default();
    uut.clock.connect();
    uut.fifo.read.connect();
    uut.fifo.data_in.connect();
    uut.fifo.write.connect();
    uut.connect_all();
    yosys_validate("fifo", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    let rdata = (0..1024)
        .map(|_| Bits::<16>::from(rand::random::<u16>()))
        .collect::<Vec<_>>();
    let rdata_read = rdata.clone();
    sim.add_clock(5, |x: &mut SyncFIFOTest| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<SyncFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata {
            x = sim.watch(|x| !x.fifo.full.val().raw(), x)?;
            x.fifo.data_in.next = (*sample).into();
            x.fifo.write.next = true.into();
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false.into();
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        assert!(!x.fifo.underflow.val().raw());
        assert!(!x.fifo.overflow.val().raw());
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(move |mut sim: Sim<SyncFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for sample in &rdata_read {
            x = sim.watch(|x| !x.fifo.empty.val().raw(), x)?;
            assert_eq!(x.fifo.data_out.val().raw(), *sample);
            x.fifo.read.next = true.into();
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false.into();
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        assert!(!x.fifo.underflow.val().raw());
        assert!(!x.fifo.overflow.val().raw());
        sim.done(x)?;
        Ok(())
    });
    sim.run_traced(uut, 100_000, std::fs::File::create("fifo.vcd").unwrap())
        .unwrap();
}

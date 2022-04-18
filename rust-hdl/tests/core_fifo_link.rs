use crate::test_common::fifo_tester::{bursty_vec, LazyFIFOFeeder, LazyFIFOReader};
use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::bus::{FIFOReadController, FIFOWriteResponder};
use rust_hdl::hls::fifo::SyncFIFO;
use rust_hdl::hls::fifo_linker::FIFOLink;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

mod test_common;

#[derive(LogicBlock)]
struct FIFOBridgeTest {
    feeder: LazyFIFOFeeder<Bits<8>, 12>,
    fp: SyncFIFO<Bits<8>, 4, 5, 1>,
    bp: SyncFIFO<Bits<8>, 4, 5, 1>,
    reader: LazyFIFOReader<Bits<8>, 12>,
    lnk: FIFOLink<Bits<8>>,
    clock: Signal<In, Clock>,
    reset: Signal<In, ResetN>,
}

impl Logic for FIFOBridgeTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, fp, bp, reader, feeder);
        FIFOWriteController::<Bits<8>>::join(&mut self.feeder.bus, &mut self.fp.bus_write);
        FIFOReadResponder::<Bits<8>>::join(&mut self.fp.bus_read, &mut self.lnk.read);
        FIFOWriteController::<Bits<8>>::join(&mut self.lnk.write, &mut self.bp.bus_write);
        FIFOReadResponder::<Bits<8>>::join(&mut self.bp.bus_read, &mut self.reader.bus);
    }
}

impl Default for FIFOBridgeTest {
    fn default() -> Self {
        let data1 = (0..256)
            .map(|_| Bits::<8>::from(rand::thread_rng().gen::<u8>()))
            .collect::<Vec<_>>();
        let data2 = data1.clone();
        Self {
            feeder: LazyFIFOFeeder::new(&data2, &bursty_vec(256)),
            fp: Default::default(),
            bp: Default::default(),
            reader: LazyFIFOReader::new(&data1, &bursty_vec(256)),
            lnk: Default::default(),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

#[test]
fn test_fifo_linker() {
    let mut uut = FIFOBridgeTest::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.feeder.start.connect();
    uut.reader.start.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<FIFOBridgeTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<FIFOBridgeTest>| {
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
    sim.run_to_file(Box::new(uut), 100_000, &vcd_path!("hls_fifo_link.vcd"))
        .unwrap();
}

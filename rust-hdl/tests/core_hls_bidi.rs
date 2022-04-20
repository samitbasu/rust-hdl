use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

mod test_common;
use crate::test_common::fifo_tester::bursty_vec;
use test_common::fifo_tester::{LazyFIFOFeeder, LazyFIFOReader};

#[derive(LogicBlock)]
struct BusTest {
    dtm_feeder: LazyFIFOFeeder<Bits<8>, 10>,
    dtm_reader: LazyFIFOReader<Bits<8>, 10>,
    mtd_feeder: LazyFIFOFeeder<Bits<8>, 10>,
    mtd_reader: LazyFIFOReader<Bits<8>, 10>,
    device_to_bus_fifo: SyncFIFO<Bits<8>, 4, 5, 1>,
    device_from_bus_fifo: SyncFIFO<Bits<8>, 4, 5, 1>,
    pub device: BidiSimulatedDevice<Bits<8>>,
    pub master: BidiMaster<Bits<8>>,
    master_from_bus_fifo: SyncFIFO<Bits<8>, 4, 5, 1>,
    master_to_bus_fifo: SyncFIFO<Bits<8>, 4, 5, 1>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
}

impl Default for BusTest {
    fn default() -> Self {
        let dlen = 256;
        let data1 = (0..dlen)
            .map(|_| Bits::<8>::from(rand::thread_rng().gen::<u8>()))
            .collect::<Vec<_>>();
        let data2 = (0..dlen)
            .map(|_| Bits::<8>::from(rand::thread_rng().gen::<u8>()))
            .collect::<Vec<_>>();

        Self {
            dtm_feeder: LazyFIFOFeeder::new(&data1, &bursty_vec(data1.len())),
            dtm_reader: LazyFIFOReader::new(&data1, &bursty_vec(data1.len())),
            mtd_feeder: LazyFIFOFeeder::new(&data2, &bursty_vec(data2.len())),
            mtd_reader: LazyFIFOReader::new(&data2, &bursty_vec(data2.len())),
            device_to_bus_fifo: Default::default(),
            device_from_bus_fifo: Default::default(),
            device: Default::default(),
            master: Default::default(),
            master_from_bus_fifo: Default::default(),
            master_to_bus_fifo: Default::default(),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

impl Logic for BusTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the components
        clock_reset!(
            self,
            clock,
            reset,
            master,
            dtm_feeder,
            dtm_reader,
            mtd_feeder,
            mtd_reader,
            device_to_bus_fifo,
            device_from_bus_fifo,
            master_from_bus_fifo,
            master_to_bus_fifo
        );
        self.device.clock.next = self.clock.val();
        // Connect the busses
        FIFOReadController::<Bits<8>>::join(
            &mut self.device.data_to_bus,
            &mut self.device_to_bus_fifo.bus_read,
        );
        FIFOWriteController::<Bits<8>>::join(
            &mut self.device.data_from_bus,
            &mut self.device_from_bus_fifo.bus_write,
        );
        FIFOReadController::<Bits<8>>::join(
            &mut self.master.data_to_bus,
            &mut self.master_to_bus_fifo.bus_read,
        );
        FIFOWriteController::<Bits<8>>::join(
            &mut self.master.data_from_bus,
            &mut self.master_from_bus_fifo.bus_write,
        );
        BidiBusM::<Bits<8>>::join(&mut self.master.bus, &mut self.device.bus);
        FIFOWriteController::<Bits<8>>::join(
            &mut self.dtm_feeder.bus,
            &mut self.device_to_bus_fifo.bus_write,
        );
        FIFOWriteController::<Bits<8>>::join(
            &mut self.mtd_feeder.bus,
            &mut self.master_to_bus_fifo.bus_write,
        );
        FIFOReadController::<Bits<8>>::join(
            &mut self.dtm_reader.bus,
            &mut self.master_from_bus_fifo.bus_read,
        );
        FIFOReadController::<Bits<8>>::join(
            &mut self.mtd_reader.bus,
            &mut self.device_from_bus_fifo.bus_read,
        );
    }
}

#[test]
fn test_bidi2_bus_test_synthesizes() {
    let mut uut = BusTest::default();
    uut.mtd_feeder.start.connect();
    uut.mtd_reader.start.connect();
    uut.dtm_feeder.start.connect();
    uut.dtm_reader.start.connect();
    uut.clock.connect();
    uut.reset.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("tribus", &vlog).unwrap();
}

#[test]
fn test_bidi2_bus_works() {
    let mut uut = BusTest::default();
    uut.mtd_feeder.start.connect();
    uut.mtd_reader.start.connect();
    uut.dtm_feeder.start.connect();
    uut.dtm_reader.start.connect();
    uut.clock.connect();
    uut.reset.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("tribus_0", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<BusTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        wait_clock_true!(sim, clock, x);
        x.dtm_feeder.start.next = true;
        x.dtm_reader.start.next = true;
        x.mtd_feeder.start.next = true;
        x.mtd_reader.start.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.dtm_feeder.start.next = false;
        x.dtm_reader.start.next = false;
        x.mtd_feeder.start.next = false;
        x.mtd_reader.start.next = false;
        x = sim.watch(
            |x| {
                x.dtm_feeder.done.val()
                    & x.dtm_reader.done.val()
                    & x.mtd_feeder.done.val()
                    & x.mtd_reader.done.val()
            },
            x,
        )?;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.dtm_reader.error.val(), x);
        sim_assert!(sim, !x.mtd_reader.error.val(), x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 500_000, &vcd_path!("bidi_stress.vcd"))
        .unwrap();
}

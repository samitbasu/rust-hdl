use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

#[derive(LogicBlock, Default)]
struct HLSFIFOTest {
    fifo: HLSFIFO<8, 3, 4, 1>,
    clock: Signal<In, Clock>,
}

impl Logic for HLSFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.bus_write.clock.next = self.clock.val();
        self.fifo.bus_read.clock.next = self.clock.val();
    }
}

#[test]
fn test_hls_fifo_works() {
    let mut uut = HLSFIFOTest::default();
    uut.clock.connect();
    uut.fifo.bus_write.write.connect();
    uut.fifo.bus_write.data.connect();
    uut.fifo.bus_read.read.connect();
    uut.fifo.bus_read.data.connect();
    uut.connect_all();
    let rdata = (0..128)
        .map(|_| Bits::<8>::from(rand::random::<u8>()))
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

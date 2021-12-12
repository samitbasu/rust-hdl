use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use rust_hdl::widgets::bidirectional_bus::BidirectionalBusSlave;
use rand::Rng;

#[derive(LogicBlock, Default)]
struct BusTest {
    pub slave: BidirectionalBusSlave<Bits<8>, 4, 5>,
    pub master: BidirectionalBusMaster<Bits<8>, 4, 5>,
    pub clock: Signal<In, Clock>,
}

impl Logic for BusTest {
    fn update(&mut self) {
        self.master.clock.next = self.clock.val();
        self.slave.clock.next = self.clock.val();
        self.master.bus.simulate_connected_tristate(&mut self.slave.bus);
        self.master.bus_empty.next = self.slave.bus_empty.val();
        self.master.bus_full.next = self.slave.bus_full.val();
        self.slave.bus_read.next = self.master.bus_read.val();
        self.slave.bus_write.next = self.master.bus_write.val();
        self.slave.slave_is_reading.next = self.master.slave_is_reading.val();
    }
}

#[test]
fn test_bidi_bus_works() {
    let mut uut = BusTest::default();
    uut.clock.connect();
    uut.slave.data_in.connect();
    uut.slave.data_read.connect();
    uut.slave.data_write.connect();
    uut.master.data_in.connect();
    uut.master.data_read.connect();
    uut.master.data_write.connect();
    uut.slave.slave_is_reading.connect();
    uut.slave.bus_read.connect();
    uut.slave.bus_write.connect();
    uut.master.bus_empty.connect();
    uut.master.bus_full.connect();
    uut.slave.clock.connect();
    uut.master.clock.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("tribus", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<BusTest>| x.clock.next = !x.clock.val());
    let s_to_m = (0..1024)
        .map(|_| Bits::<8>::from(rand::random::<u8>()))
        .collect::<Vec<_>>();
    let s_to_m_verify = s_to_m.clone();
    let m_to_s = (0..1024)
        .map(|_| Bits::<8>::from(rand::random::<u8>()))
        .collect::<Vec<_>>();
    let m_to_s_verify = m_to_s.clone();
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
       let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in m_to_s.clone() {
            x.master.data_write.next = false;
            x = sim.watch(|x| !x.master.data_full.val(), x)?;
            x.master.data_in.next = val.into();
            x.master.data_write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.data_write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        // Write some data into the slave's FIFO
        wait_clock_true!(sim, clock, x);
        for val in s_to_m.clone() {
            x.slave.data_write.next = false;
            x = sim.watch(|x| !x.slave.data_full.val(), x)?;
            x.slave.data_in.next = val.into();
            x.slave.data_write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.slave.data_write.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in s_to_m_verify.clone() {
            x = sim.watch(|x| !x.master.data_empty.val(), x)?;
            sim_assert!(sim, x.master.data_out.val() == val, x);
            wait_clock_true!(sim, clock, x);
            x.master.data_read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.data_read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in m_to_s_verify.clone() {
            x = sim.watch(|x| !x.slave.data_empty.val(), x)?;
            wait_clock_true!(sim, clock, x);
            x.slave.data_read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.slave.data_read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    let mut vcd = vec![];
    sim.run_traced(Box::new(uut), 1_000_000, &mut vcd).unwrap();
    std::fs::write(vcd_path!("bus_tri_stress.vcd"), vcd).unwrap();
}
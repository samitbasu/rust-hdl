use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock, Default)]
struct BusTest {
    pub device: BidiDevice<Bits<8>, 4, 5>,
    pub master: BidiMaster<Bits<8>, 4, 5>,
    pub clock: Signal<In, Clock>,
}

impl Logic for BusTest {
    fn update(&mut self) {
        self.master.bus_clock.next = self.clock.val();
        self.master.data_clock.next = self.clock.val();
        self.device.clock.next = self.clock.val();
        self.master.bus.sig_empty.next = self.device.bus.sig_empty.val();
        self.master.bus.sig_full.next = self.device.bus.sig_full.val();
        self.device.bus.sig_not_read.next = self.master.bus.sig_not_read.val();
        self.device.bus.sig_not_write.next = self.master.bus.sig_not_write.val();
        self.device.bus.sig_master.next = self.master.bus.sig_master.val();
        self.master
            .bus
            .sig_inout
            .simulate_connected_tristate(&mut self.device.bus.sig_inout);
    }
}

#[test]
fn test_bidi2_bus_works() {
    let mut uut = BusTest::default();
    uut.clock.connect();
    uut.device.clock.connect();
    uut.master.bus_clock.connect();
    uut.master.data_clock.connect();
    uut.device.data.to_bus.connect();
    uut.device.data.read.connect();
    uut.device.data.write.connect();
    uut.master.data.to_bus.connect();
    uut.master.data.read.connect();
    uut.master.data.write.connect();
    uut.device.bus.sig_master.connect();
    uut.device.bus.sig_not_read.connect();
    uut.device.bus.sig_not_write.connect();
    uut.master.bus.sig_empty.connect();
    uut.master.bus.sig_full.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("tribus", &vlog).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<BusTest>| x.clock.next = !x.clock.val());
    let s_to_m = (0..256)
        .map(|_| Bits::<8>::from(rand::random::<u8>()))
        .collect::<Vec<_>>();
    let s_to_m_verify = s_to_m.clone();
    let m_to_s = (0..256)
        .map(|_| Bits::<8>::from(rand::random::<u8>()))
        .collect::<Vec<_>>();
    let m_to_s_verify = m_to_s.clone();
    sim.add_testbench(move |mut sim: Sim<BusTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for val in m_to_s.clone() {
            x.master.data.write.next = false;
            x = sim.watch(|x| !x.master.data.full.val(), x)?;
            x.master.data.to_bus.next = val.into();
            x.master.data.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.data.write.next = false;
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
            x.device.data.write.next = false;
            x = sim.watch(|x| !x.device.data.full.val(), x)?;
            x.device.data.to_bus.next = val.into();
            x.device.data.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.device.data.write.next = false;
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
            x = sim.watch(|x| !x.master.data.empty.val(), x)?;
            sim_assert!(sim, x.master.data.from_bus.val() == val, x);
            wait_clock_true!(sim, clock, x);
            x.master.data.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.master.data.read.next = false;
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
            x = sim.watch(|x| !x.device.data.empty.val(), x)?;
            sim_assert!(sim, x.device.data.from_bus.val() == val, x);
            wait_clock_true!(sim, clock, x);
            x.device.data.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.device.data.read.next = false;
            if rand::thread_rng().gen::<f64>() < 0.3 {
                for _ in 0..(rand::thread_rng().gen::<u8>() % 40) {
                    wait_clock_cycle!(sim, clock, x);
                }
            }
        }
        sim.done(x)
    });
    let mut vcd = vec![];
    let ret = sim.run_traced(Box::new(uut), 100_000, &mut vcd);
    std::fs::write(vcd_path!("bus2_tri_stress.vcd"), vcd).unwrap();
    ret.unwrap()
}

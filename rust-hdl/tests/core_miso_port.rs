use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct MISOPortTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MISOPort<16, 8>,
    port_b: MISOPort<16, 8>,
    clock: Signal<In, Clock>,
}

impl Logic for MISOPortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.port_a.clock.next = self.clock.val();
        self.port_b.clock.next = self.clock.val();
        // Input ports for port_a
        self.port_a.bus.strobe.next = self.bus_m.strobe.val();
        self.port_a.bus.addr.next = self.bus_m.addr.val();
        self.port_a.bus.from_master.next = self.bus_m.from_master.val();
        // Input ports for port_b
        self.port_b.bus.strobe.next = self.bus_m.strobe.val();
        self.port_b.bus.addr.next = self.bus_m.addr.val();
        self.port_b.bus.from_master.next = self.bus_m.from_master.val();
        // Output ports
        self.bus_m.to_master.next =
            self.port_a.bus.to_master.val() | self.port_b.bus.to_master.val();
        self.bus_m.ready.next = self.port_a.bus.ready.val() | self.port_b.bus.ready.val();
    }
}

impl Default for MISOPortTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MISOPort::new(34_usize.into()),
            port_b: MISOPort::new(36_usize.into()),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MISOPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}

#[test]
fn test_port_test_works() {
    let mut uut = MISOPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MISOPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xDEAD_u16, x);
        x.bus_m.addr.next = 36_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xCAFE_u16, x);
        wait_clock_cycles!(sim, clock, x, 50);
        x.bus_m.addr.next = 34_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xBEEF_u16, x);
        x.bus_m.addr.next = 36_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        sim_assert!(sim, x.bus_m.to_master.val() == 0xBABE_u16, x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.port_a.port_in.next = 0xDEAD_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        x.port_a.port_in.next = 0xBEEF_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.port_b.port_in.next = 0xCAFE_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        x.port_b.port_in.next = 0xBABE_u16.into();
        wait_clock_cycles!(sim, clock, x, 50);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        2000,
        std::fs::File::create(vcd_path!("miso_port.vcd")).unwrap(),
    )
    .unwrap();
}

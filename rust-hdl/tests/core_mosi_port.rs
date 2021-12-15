use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct MOSIPortTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MOSIPort<16, 8>,
    port_b: MOSIPort<16, 8>,
    clock: Signal<In, Clock>,
}

impl Logic for MOSIPortTest {
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

impl Default for MOSIPortTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MOSIPort::new(35_usize.into()),
            port_b: MOSIPort::new(37_usize.into()),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MOSIPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}

#[test]
fn test_port_test_works() {
    let mut uut = MOSIPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 37_usize.into();
        x.bus_m.from_master.next = 0xDEAD_u16.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        x.bus_m.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus_m.strobe.next = false;
        x.bus_m.addr.next = 35_usize.into();
        x.bus_m.from_master.next = 0xBEEF_u16.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        x.bus_m.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus_m.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_a.port_out.val() == 0xBEEF_u16, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xDEAD_u16, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_port.vcd")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_port_pipeline() {
    let mut uut = MOSIPortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 37_usize.into();
        x = sim.watch(|x| x.bus_m.ready.val(), x)?;
        for val in [0xDEAD_u16, 0xBEEF, 0xBABE, 0xCAFE] {
            x.bus_m.from_master.next = val.into();
            x.bus_m.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus_m.strobe.next = false;
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        for val in [0xDEAD_u16, 0xBEEF, 0xBABE, 0xCAFE] {
            x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
            sim_assert!(sim, x.port_b.port_out.val() == val, x);
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_port_pipeline.vcd")).unwrap(),
    )
        .unwrap();
}




#[derive(LogicBlock)]
struct MOSIWidePortTest {
    bus_m: LocalBusM<16, 8>,
    port_a: MOSIWidePort<64, 16, 8>,
    port_b: MOSIWidePort<64, 16, 8>,
    clock: Signal<In, Clock>,
}

impl Logic for MOSIWidePortTest {
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

impl Default for MOSIWidePortTest {
    fn default() -> Self {
        Self {
            bus_m: Default::default(),
            port_a: MOSIWidePort::new(35_usize.into()),
            port_b: MOSIWidePort::new(37_usize.into()),
            clock: Default::default(),
        }
    }
}

#[test]
fn test_wport_test_synthesizes() {
    let mut uut = MOSIWidePortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("wide_test_port", &vlog).unwrap();
}

#[test]
fn test_wide_port_test_works() {
    let mut uut = MOSIWidePortTest::default();
    uut.clock.connect();
    uut.bus_m.addr.connect();
    uut.bus_m.from_master.connect();
    uut.bus_m.strobe.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIWidePortTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus_m.addr.next = 35_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        for val in [0xDEAD_u16, 0xBEEF_u16, 0xCAFE_u16, 0x1234_u16] {
            x.bus_m.strobe.next = true;
            x.bus_m.from_master.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus_m.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        x.bus_m.addr.next = 37_usize.into();
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, x.bus_m.ready.val(), x);
        for val in [
            0xBABE_u16, 0x5EA1_u16, 0xFACE_u16, 0xABCD_u16, 0xBABA_u16, 0xCECE_u16, 0x4321_u16,
            0x89AB_u16,
        ] {
            x.bus_m.strobe.next = true;
            x.bus_m.from_master.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus_m.strobe.next = false;
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_a.port_out.val() == 0xDEADBEEFCAFE1234_u64, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xBABE5EA1FACEABCD_u64, x);
        wait_clock_cycle!(sim, clock, x);
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert!(sim, x.port_b.port_out.val() == 0xBABACECE432189AB_u64, x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        1000,
        std::fs::File::create(vcd_path!("mosi_wide_port.vcd")).unwrap(),
    )
    .unwrap();
}

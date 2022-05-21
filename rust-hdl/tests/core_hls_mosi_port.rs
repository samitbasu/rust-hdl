use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

#[derive(LogicBlock)]
struct MOSIPortTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MOSIPort<16>,
    port_b: MOSIPort<16>,
    clock: Signal<In, Clock>,
}

impl Default for MOSIPortTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a", "port_b"]),
            port_a: Default::default(),
            port_b: Default::default(),
            clock: Default::default(),
        }
    }
}

impl Logic for MOSIPortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.port_b.bus);
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MOSIPortTest::default();
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}
#[test]
fn test_port_test_works() {
    let mut uut = MOSIPortTest::default();
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 10);
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        x.bus.from_controller.next = 0xDEAD_u16.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        x.bus.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.strobe.next = false;
        x.bus.address.next = 0_usize.into();
        x.bus.from_controller.next = 0xBEEF_u16.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        x.bus.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x.port_a.ready.next = true;
        x = sim.watch(|x| x.port_a.strobe_out.val(), x)?;
        sim_assert_eq!(sim, x.port_a.port_out.val(), 0xBEEF_u16, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        x.port_b.ready.next = true;
        x = sim.watch(|x| x.port_b.strobe_out.val(), x)?;
        sim_assert_eq!(sim, x.port_b.port_out.val(), 0xDEAD_u16, x);
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
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIPortTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 10);
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [0xDEAD_u16, 0xBEEF, 0xBABE, 0xCAFE] {
            x.bus.from_controller.next = val.into();
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortTest>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, clock, x, 10);
        x.port_b.ready.next = true;
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
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MOSIWidePort<64, 16>,
    port_b: MOSIWidePort<64, 16>,
    clock: Signal<In, Clock>,
}

impl Default for MOSIWidePortTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a", "port_b"]),
            port_a: Default::default(),
            port_b: Default::default(),
            clock: Default::default(),
        }
    }
}

impl HLSNamedPorts for MOSIWidePortTest {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

impl Logic for MOSIWidePortTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.port_b.bus);
    }
}

#[test]
fn test_wport_test_synthesizes() {
    let mut uut = MOSIWidePortTest::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("wide_test_port", &vlog).unwrap();
}

#[test]
fn test_wide_port_test_works() {
    let mut uut = MOSIWidePortTest::default();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MOSIWidePortTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MOSIWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 0_usize.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [0xDEAD_u16, 0xBEEF_u16, 0xCAFE_u16, 0x1234_u16] {
            x.bus.strobe.next = true;
            x.bus.from_controller.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address.next = 1_usize.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        for val in [
            0xBABE_u16, 0x5EA1_u16, 0xFACE_u16, 0xABCD_u16, 0xBABA_u16, 0xCECE_u16, 0x4321_u16,
            0x89AB_u16,
        ] {
            x.bus.strobe.next = true;
            x.bus.from_controller.next = val.into();
            wait_clock_cycle!(sim, clock, x);
        }
        x.bus.strobe.next = false;
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

#[derive(LogicBlock)]
struct MOSIPortFIFOTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 1>,
    port_a: MOSIFIFOPort<16, 4, 5, 1>,
    clock: Signal<In, Clock>,
}

impl Default for MOSIPortFIFOTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a"]),
            port_a: Default::default(),
            clock: Default::default(),
        }
    }
}

impl HLSNamedPorts for MOSIPortFIFOTest {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

impl Logic for MOSIPortFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.bus.clock.next = self.clock.val();
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
    }
}

#[test]
fn test_mosi_port_fifo_synthesizes() {
    let mut uut = MOSIPortFIFOTest::default();
    uut.port_a.fifo_bus.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("mosi_port_fifo", &vlog).unwrap();
}

#[test]
fn test_mosi_port_fifo_works() {
    let mut uut = MOSIPortFIFOTest::default();
    uut.port_a.fifo_bus.link_connect_dest();
    uut.connect_all();
    let mut sim = Simulation::new();
    let vals = [0xDEAD_u16, 0xBEEF_u16, 0xBABE_u16, 0xCAFE_u16];
    sim.add_clock(5, |x: &mut Box<MOSIPortFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.bus.address.next = 0_usize.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.bus.address_strobe.next = false;
        for val in vals.clone() {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            x.bus.from_controller.next = val.into();
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycles!(sim, clock, x, 100);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MOSIPortFIFOTest>| {
        let mut x = sim.init()?;
        hls_fifo_read!(sim, clock, x, port_a.fifo_bus, &vals.clone());
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("mosi_fifo.vcd")).unwrap(),
    )
    .unwrap();
}

use rand::Rng;
use rust_hdl::prelude::*;

#[derive(LogicBlock)]
struct MISOPortTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MISOPort<16>,
    port_b: MISOPort<16>,
}

impl Default for MISOPortTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a", "port_b"]),
            port_a: Default::default(),
            port_b: Default::default(),
        }
    }
}

impl Logic for MISOPortTest {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.port_b.bus);
    }
}

#[test]
fn test_port_test_synthesizes() {
    let mut uut = MISOPortTest::default();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.ready_in.connect();
    uut.port_b.ready_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_port", &vlog).unwrap();
}

#[test]
fn test_port_test_works() {
    let mut uut = MISOPortTest::default();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.ready_in.connect();
    uut.port_b.ready_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MISOPortTest>| {
        x.bus.clock.next = !x.bus.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bus.clock, x);
        x.bus.address.next = 0.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        sim_assert!(sim, x.bus.to_controller.val() == 0xDEAD, x);
        x.bus.address.next = 1.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        sim_assert!(sim, x.bus.to_controller.val() == 0xCAFE, x);
        wait_clock_cycles!(sim, bus.clock, x, 50);
        x.bus.address.next = 0.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        sim_assert!(sim, x.bus.to_controller.val() == 0xBEEF, x);
        x.bus.address.next = 1.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        x = sim.watch(|x| x.bus.ready.val(), x)?;
        sim_assert!(sim, x.bus.to_controller.val() == 0xBABE, x);
        wait_clock_cycle!(sim, bus.clock, x);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        x.port_a.ready_in.next = true;
        wait_clock_true!(sim, bus.clock, x);
        x.port_a.port_in.next = 0xDEAD.into();
        wait_clock_cycles!(sim, bus.clock, x, 50);
        x.port_a.port_in.next = 0xBEEF.into();
        wait_clock_cycles!(sim, bus.clock, x, 50);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortTest>| {
        let mut x = sim.init()?;
        x.port_b.ready_in.next = true;
        wait_clock_true!(sim, bus.clock, x);
        x.port_b.port_in.next = 0xCAFE.into();
        wait_clock_cycles!(sim, bus.clock, x, 50);
        x.port_b.port_in.next = 0xBABE.into();
        wait_clock_cycles!(sim, bus.clock, x, 50);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        2000,
        std::fs::File::create(vcd_path!("miso_port.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock)]
struct MISOWidePortTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 2>,
    port_a: MISOWidePort<64, 16>,
    port_b: MISOWidePort<64, 16>,
}

impl Default for MISOWidePortTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a", "port_b"]),
            port_a: Default::default(),
            port_b: Default::default(),
        }
    }
}

impl Logic for MISOWidePortTest {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.port_b.bus);
    }
}

#[test]
fn test_wide_port_test_synthesizes() {
    let mut uut = MISOWidePortTest::default();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.strobe_in.connect();
    uut.port_b.strobe_in.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("test_wide_port", &vlog).unwrap();
}

#[test]
fn test_wide_port_test_works() {
    let mut uut = MISOWidePortTest::default();
    uut.port_a.port_in.connect();
    uut.port_b.port_in.connect();
    uut.port_a.strobe_in.connect();
    uut.port_b.strobe_in.connect();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<MISOWidePortTest>| {
        x.bus.clock.next = !x.bus.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bus.clock, x);
        x.bus.address.next = 0.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        for val in [0xDEAD, 0xBEEF, 0x1234, 0xABCD] {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            sim_assert!(sim, x.bus.to_controller.val() == val, x);
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, bus.clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address.next = 1.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        for val in [0xCAFE, 0xFEED, 0xBABE, 0x5EA1] {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            sim_assert!(sim, x.bus.to_controller.val() == val, x);
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, bus.clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address.next = 0.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        for val in [0x0123, 0x4567, 0x89AB, 0xCDEF] {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            sim_assert!(sim, x.bus.to_controller.val() == val, x);
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, bus.clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycles!(sim, bus.clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bus.clock, x);
        wait_clock_cycles!(sim, bus.clock, x, 25);
        x.port_a.port_in.next = 0xDEADBEEF1234ABCD.into();
        x.port_a.strobe_in.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, bus.clock, x, 50);
        x.port_a.port_in.next = 0x0123_4567_89AB_CDEF.into();
        x.port_a.strobe_in.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, bus.clock, x, 50);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOWidePortTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bus.clock, x);
        x.port_b.port_in.next = 0xCAFEFEEDBABE5EA1.into();
        x.port_b.strobe_in.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.port_a.strobe_in.next = false;
        wait_clock_cycles!(sim, bus.clock, x, 50);
        sim.done(x)
    });
    let ret = sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("miso_wide_port.vcd")).unwrap(),
    );
    ret.unwrap();
}

#[derive(LogicBlock)]
struct MISOPortFIFOTest {
    bus: SoCBusController<16, 2>,
    bridge: Bridge<16, 2, 1>,
    port_a: MISOFIFOPort<16, 2, 3, 1>,
}

impl Default for MISOPortFIFOTest {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            bridge: Bridge::new(["port_a"]),
            port_a: Default::default(),
        }
    }
}

impl Logic for MISOPortFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusController::<16, 2>::join(&mut self.bus, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
    }
}

#[test]
fn test_miso_fifo_synthesizes() {
    let mut uut = MISOPortFIFOTest::default();
    uut.bus.clock.connect();
    uut.bus.address.connect();
    uut.bus.address_strobe.connect();
    uut.bus.from_controller.connect();
    uut.bus.strobe.connect();
    uut.port_a.fifo_bus.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("miso_fifo", &vlog).unwrap();
}

#[test]
fn test_miso_fifo_works() {
    let mut uut = MISOPortFIFOTest::default();
    uut.port_a.fifo_bus.link_connect_dest();
    uut.connect_all();
    let mut sim = Simulation::new();
    let test_data = [0xDEAD, 0xBEEF, 0xCAFE, 0xBABE, 0x1234, 0x5678, 0x5423];
    sim.add_clock(5, |x: &mut Box<MISOPortFIFOTest>| {
        x.bus.clock.next = !x.bus.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_cycles!(sim, bus.clock, x, 10);
        wait_clock_true!(sim, bus.clock, x);
        hls_fifo_write_lazy!(sim, bus.clock, x, port_a.fifo_bus, &test_data.clone());
        wait_clock_cycles!(sim, bus.clock, x, 10);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<MISOPortFIFOTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bus.clock, x);
        wait_clock_cycles!(sim, bus.clock, x, 20);
        x.bus.address.next = 0.into();
        x.bus.address_strobe.next = true;
        wait_clock_cycle!(sim, bus.clock, x);
        x.bus.address_strobe.next = false;
        for val in test_data.clone() {
            x = sim.watch(|x| x.bus.ready.val(), x)?;
            sim_assert_eq!(sim, x.bus.to_controller.val(), val, x);
            x.bus.strobe.next = true;
            wait_clock_cycle!(sim, bus.clock, x);
            x.bus.strobe.next = false;
        }
        wait_clock_cycles!(sim, bus.clock, x, 10);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        5000,
        std::fs::File::create(vcd_path!("miso_fifo.vcd")).unwrap(),
    )
    .unwrap();
}

use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

#[derive(LogicBlock)]
struct RouterTest {
    router: Router<16, 8, 6>,
    clock: Signal<In, Clock>,
}

impl Default for RouterTest {
    fn default() -> Self {
        let router = Router::<16, 8, 6>::new([4, 8, 12, 4, 4, 4]);
        Self {
            router,
            clock: Default::default(),
        }
    }
}

impl Logic for RouterTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.router.upstream.clock.next = self.clock.val();
    }
}

#[cfg(test)]
fn make_test_router() -> RouterTest {
    let mut uut = RouterTest::default();
    uut.router.upstream.address.connect();
    uut.router.upstream.address_strobe.connect();
    uut.router.upstream.from_controller.connect();
    uut.router.upstream.strobe.connect();
    uut.router.upstream.clock.connect();
    for i in 0..6 {
        uut.router.nodes[i].ready.connect();
        uut.router.nodes[i].to_controller.connect();
    }
    uut.clock.connect();
    uut.router.connect_all();
    uut
}

#[test]
fn test_router_is_synthesizable() {
    let router = make_test_router();
    let vlog = generate_verilog(&router);
    println!("{}", vlog);
    yosys_validate("router", &vlog).unwrap();
}

#[test]
fn test_router_function() {
    let router = make_test_router();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<RouterTest>| x.clock.next = !x.clock.val());
    sim.add_testbench(move |mut sim: Sim<RouterTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.router.upstream.address.next = 7_usize.into();
        x.router.upstream.address_strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.router.upstream.address_strobe.next = false;
        x = sim.watch(|x| x.router.upstream.ready.val(), x)?;
        x.router.upstream.from_controller.next = 0xDEAD_u16.into();
        x.router.upstream.strobe.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.router.upstream.strobe.next = false;
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<RouterTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x = sim.watch(|x| x.router.nodes[1].address.val() == 0x03_u8, x)?;
        x.router.nodes[1].ready.next = true;
        x = sim.watch(|x| x.router.nodes[1].strobe.val(), x)?;
        sim_assert!(
            sim,
            x.router.nodes[1].from_controller.val() == 0xDEAD_u16,
            x
        );
        wait_clock_cycles!(sim, clock, x, 10);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(router),
        1000,
        std::fs::File::create(vcd_path!("router.vcd")).unwrap(),
    )
    .unwrap();
}

#[derive(LogicBlock)]
struct RouterTestDevice {
    pub upstream: SoCBusResponder<16, 8>,
    bridge: Bridge<16, 8, 5>,
    mosi_ports: [MOSIPort<16>; 5],
}

impl Default for RouterTestDevice {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            bridge: Default::default(),
            mosi_ports: array_init::array_init(|_| Default::default()),
        }
    }
}

impl Logic for RouterTestDevice {
    #[hdl_gen]
    fn update(&mut self) {
        self.upstream.link(&mut self.bridge.upstream);
        for i in 0_usize..5 {
            SoCPortController::<16>::join(&mut self.bridge.nodes[i], &mut self.mosi_ports[i].bus);
            self.mosi_ports[i].ready.next = self.mosi_ports[i].bus.select.val();
        }
    }
}

#[test]
fn test_device_synthesizes() {
    let mut uut = RouterTestDevice::default();
    uut.upstream.clock.connect();
    uut.upstream.from_controller.connect();
    uut.upstream.address.connect();
    uut.upstream.address_strobe.connect();
    uut.upstream.strobe.connect();
    for i in 0..5 {
        uut.mosi_ports[i].ready.connect();
    }
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("router_test_device", &vlog).unwrap();
}

#[derive(LogicBlock)]
struct RouterTestSetup {
    pub upstream: SoCBusResponder<16, 8>,
    router: Router<16, 8, 3>,
    dev_a: [RouterTestDevice; 3],
    clock: Signal<In, Clock>,
}

impl Default for RouterTestSetup {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            router: Router::new([5, 5, 5]),
            dev_a: array_init::array_init(|_| Default::default()),
            clock: Default::default(),
        }
    }
}

impl Logic for RouterTestSetup {
    #[hdl_gen]
    fn update(&mut self) {
        self.upstream.link(&mut self.router.upstream);
        for i in 0_usize..3 {
            SoCBusController::<16, 8>::join(&mut self.router.nodes[i], &mut self.dev_a[0].upstream);
        }
        self.upstream.clock.next = self.clock.val();
    }
}

#[cfg(test)]
fn make_router_test_setup() -> RouterTestSetup {
    let mut uut = RouterTestSetup::default();
    uut.upstream.address.connect();
    uut.upstream.address_strobe.connect();
    uut.upstream.strobe.connect();
    uut.upstream.from_controller.connect();
    uut.clock.connect();
    for dev in 0..3 {
        for port in 0..5 {
            uut.dev_a[dev].mosi_ports[port].ready.connect();
        }
    }
    uut.connect_all();
    uut
}

#[test]
fn test_router_test_setup_synthesizes() {
    let uut = make_router_test_setup();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("router_test_setup", &vlog).unwrap();
}

#[test]
fn test_router_test_setup_works() {
    let uut = make_router_test_setup();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<RouterTestSetup>| {
        x.clock.next = !x.clock.val()
    });
    let dataset = [
        0xBEAF_u16, 0xDEED, 0xCAFE, 0xBABE, 0x1234, 0x5678, 0x900B, 0xB001, 0xDEAD, 0xBEEF, 0x5EA1,
        0x5AFE, 0xAAAA, 0x5A13, 0x8675,
    ];
    sim.add_testbench(move |mut sim: Sim<RouterTestSetup>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for address in 0_u8..15 {
            // Sweep the address space...
            x.upstream.address.next = address.into();
            x.upstream.from_controller.next = dataset[address as usize].into();
            x.upstream.address_strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.upstream.address_strobe.next = false;
            x = sim.watch(|x| x.upstream.ready.val(), x)?;
            x.upstream.strobe.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.upstream.strobe.next = false;
            wait_clock_cycle!(sim, clock, x);
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<RouterTestSetup>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for dev in 0..3 {
            for node in 0..5 {
                x = sim.watch(
                    move |x| {
                        x.dev_a[dev.clone()].mosi_ports[node.clone()]
                            .bus
                            .select
                            .val()
                    },
                    x,
                )?;
                x = sim.watch(
                    move |x| {
                        x.dev_a[dev.clone()].mosi_ports[node.clone()]
                            .strobe_out
                            .val()
                    },
                    x,
                )?;
                println!(
                    "Dataset {} {} {:x} {:x}",
                    dev,
                    node,
                    dataset[dev * 5 + node],
                    x.dev_a[dev.clone()].mosi_ports[node.clone()].port_out.val()
                );
                sim_assert!(
                    sim,
                    x.dev_a[dev.clone()].mosi_ports[node.clone()].port_out.val()
                        == dataset[dev * 5 + node],
                    x
                );
                wait_clock_cycle!(sim, clock, x);
            }
        }
        sim.done(x)
    });
    sim.run_traced(
        Box::new(uut),
        10000,
        std::fs::File::create(vcd_path!("router_test_setup_function.vcd")).unwrap(),
    )
    .unwrap();
}

use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::hls::router_rom::RouterROM;

// Test the case of a stacked router configuration

// This is simply 2 ports on a bridge interface
#[derive(LogicBlock)]
struct BridgeTest {
    upstream: SoCBusResponder<16, 8>,
    bridge: Bridge<16, 8, 2>,
    port_a: MOSIPort<16>,
    port_b: MISOPort<16>,
}

impl Default for BridgeTest {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            bridge: Bridge::new(["port_a_mosi", "port_b_miso"]),
            port_a: Default::default(),
            port_b: Default::default(),
        }
    }
}

impl Logic for BridgeTest {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.bridge.upstream);
        SoCPortController::<16>::join(&mut self.bridge.nodes[0], &mut self.port_a.bus);
        SoCPortController::<16>::join(&mut self.bridge.nodes[1], &mut self.port_b.bus);
    }
}

impl HLSNamedPorts for BridgeTest {
    fn ports(&self) -> Vec<String> {
        self.bridge.ports()
    }
}

#[test]
fn test_bridge_test_stack_synthesizes() {
    let mut uut = TopWrap::new(BridgeTest::default());
    uut.uut.upstream.link_connect_dest();
    uut.uut.port_a.ready.connect();
    uut.uut.port_b.port_in.connect();
    uut.uut.port_b.ready_in.connect();
    uut.connect_all();
    yosys_validate("bridge_test_stack", &generate_verilog(&uut)).unwrap();
}

// This is 2 bridges on a router
#[derive(LogicBlock)]
struct BridgePair {
    upstream: SoCBusResponder<16, 8>,
    router: RouterROM<16, 8, 2>,
    devs: [BridgeTest; 2],
}

impl Default for BridgePair {
    fn default() -> Self {
        let devs = [BridgeTest::default(), BridgeTest::default()];
        Self {
            upstream: Default::default(),
            router: RouterROM::new(["dev_0", "dev_1"], [&devs[0], &devs[1]]),
            devs: Default::default(),
        }
    }
}

impl Logic for BridgePair {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.router.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[0], &mut self.devs[0].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[1], &mut self.devs[1].upstream);
    }
}

impl HLSNamedPorts for BridgePair {
    fn ports(&self) -> Vec<String> {
        self.router.ports()
    }
}

#[test]
fn test_bridge_pair_synthesizes() {
    let mut uut = TopWrap::new(BridgePair::default());
    uut.uut.upstream.link_connect_dest();
    uut.uut.devs[0].port_a.ready.connect();
    uut.uut.devs[0].port_b.port_in.connect();
    uut.uut.devs[0].port_b.ready_in.connect();
    uut.uut.devs[1].port_a.ready.connect();
    uut.uut.devs[1].port_b.port_in.connect();
    uut.uut.devs[1].port_b.ready_in.connect();
    uut.connect_all();
    yosys_validate("bridge_pair_stack", &generate_verilog(&uut)).unwrap();
}

// The high level widget has 2 bridge pairs and a bridge attached to a router
#[derive(LogicBlock)]
struct RouterNest {
    upstream: SoCBusResponder<16, 8>,
    router: RouterROM<16, 8, 3>,
    pairs: [BridgePair; 2],
    solo: BridgeTest,
}

impl Default for RouterNest {
    fn default() -> Self {
        let pairs = [BridgePair::default(), BridgePair::default()];
        let solo = BridgeTest::default();
        Self {
            upstream: Default::default(),
            router: RouterROM::new(
                ["pairs_0", "pairs_1", "solo"],
                [&pairs[0], &pairs[1], &solo],
            ),
            pairs,
            solo,
        }
    }
}

impl HLSNamedPorts for RouterNest {
    fn ports(&self) -> Vec<String> {
        self.router.ports()
    }
}

impl Logic for RouterNest {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.router.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[0], &mut self.pairs[0].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[1], &mut self.pairs[1].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[2], &mut self.solo.upstream);
    }
}

#[cfg(test)]
fn mk_router_nest() -> RouterNest {
    let mut uut = RouterNest::default();
    uut.upstream.link_connect_dest();
    for j in 0..2 {
        for i in 0..2 {
            uut.pairs[j].devs[i].port_a.ready.connect();
            uut.pairs[j].devs[i].port_b.port_in.connect();
            uut.pairs[j].devs[i].port_b.ready_in.connect();
        }
    }
    uut.solo.port_a.ready.connect();
    uut.solo.port_b.port_in.connect();
    uut.solo.port_b.ready_in.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_router_nest_synthesizes() {
    let uut = TopWrap::new(mk_router_nest());
    println!("{:?}", uut.uut.ports());
    assert_eq!(uut.uut.ports().len(), 10);
    assert_eq!(
        uut.uut.ports(),
        [
            "pairs_0_dev_0_port_a_mosi", // 0
            "pairs_0_dev_0_port_b_miso", // 1
            "pairs_0_dev_1_port_a_mosi", // 2
            "pairs_0_dev_1_port_b_miso", // 3
            "pairs_1_dev_0_port_a_mosi",
            "pairs_1_dev_0_port_b_miso",
            "pairs_1_dev_1_port_a_mosi",
            "pairs_1_dev_1_port_b_miso",
            "solo_port_a_mosi",
            "solo_port_b_miso"
        ]
    );
    yosys_validate("router_nest", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_nested_router_function() {
    let uut = mk_router_nest();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<RouterNest>| {
        x.upstream.clock.next = !x.upstream.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<RouterNest>| {
        let mut x = sim.init()?;
        x.upstream.reset.next = NO_RESET;
        let test_one = x
            .ports()
            .iter()
            .position(|v| v == "pairs_0_dev_1_port_a_mosi")
            .unwrap();
        let test_two = x
            .ports()
            .iter()
            .position(|v| v == "solo_port_a_mosi")
            .unwrap();
        bus_address_strobe!(sim, x, upstream, test_one);
        bus_write_strobe!(sim, x, upstream, 0xDEAD_u16);
        bus_address_strobe!(sim, x, upstream, test_two);
        bus_write_strobe!(sim, x, upstream, 0xBEEF_u16);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<RouterNest>| {
        let mut x = sim.init()?;
        x.pairs[0].devs[1].port_a.ready.next = true;
        x = sim.watch(|x| x.pairs[0].devs[1].port_a.strobe_out.val(), x)?;
        sim_assert!(
            sim,
            x.pairs[0].devs[1].port_a.port_out.val() == 0xDEAD_u16,
            x
        );
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<RouterNest>| {
        let mut x = sim.init()?;
        x.solo.port_a.ready.next = true;
        x = sim.watch(|x| x.solo.port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.solo.port_a.port_out.val() == 0xBEEF_u16, x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000, &vcd_path!("nested_router.vcd"))
        .unwrap()
}

#[derive(LogicBlock)]
pub struct HLSTester {
    pub upstream: SoCBusResponder<16, 8>,
    local_bridge: Bridge<16, 8, 2>,
    port_in: MOSIWidePort<32, 16>,
    port_out: MISOWidePort<32, 16>,
}

impl HLSNamedPorts for HLSTester {
    fn ports(&self) -> Vec<String> {
        self.local_bridge.ports()
    }
}

impl Default for HLSTester {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            local_bridge: Bridge::new(["port_in", "port_out"]),
            port_in: Default::default(),
            port_out: Default::default(),
        }
    }
}

impl Logic for HLSTester {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.local_bridge.upstream);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[0], &mut self.port_in.bus);
        SoCPortController::<16>::join(&mut self.local_bridge.nodes[1], &mut self.port_out.bus);
        self.port_out.strobe_in.next = self.port_in.strobe_out.val();
        self.port_out.port_in.next = self.port_in.port_out.val();
    }
}

#[derive(LogicBlock)]
pub struct HLSLevel2 {
    pub upstream: SoCBusResponder<16, 8>,
    router: RouterROM<16, 8, 4>,
    debugs: [HLSTester; 4],
}

impl HLSNamedPorts for HLSLevel2 {
    fn ports(&self) -> Vec<String> {
        self.router.ports()
    }
}

impl Default for HLSLevel2 {
    fn default() -> Self {
        let debugs = array_init::array_init(|_| Default::default());
        let router = RouterROM::new(
            ["d0", "d1", "d2", "d3"],
            [&debugs[0], &debugs[1], &debugs[2], &debugs[3]],
        );
        Self {
            upstream: Default::default(),
            router,
            debugs,
        }
    }
}

#[derive(LogicBlock)]
pub struct HLSLevel1 {
    pub upstream: SoCBusResponder<16, 8>,
    router: RouterROM<16, 8, 2>,
    left: HLSLevel2,
    right: HLSLevel2,
}

impl HLSNamedPorts for HLSLevel1 {
    fn ports(&self) -> Vec<String> {
        self.router.ports()
    }
}

impl Default for HLSLevel1 {
    fn default() -> Self {
        let left = HLSLevel2::default();
        let right = HLSLevel2::default();
        let router = RouterROM::new(["left", "right"], [&left, &right]);
        Self {
            upstream: Default::default(),
            router,
            left,
            right,
        }
    }
}

impl Logic for HLSLevel1 {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.router.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[0], &mut self.left.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[1], &mut self.right.upstream);
    }
}

impl Logic for HLSLevel2 {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.router.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[0], &mut self.debugs[0].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[1], &mut self.debugs[1].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[2], &mut self.debugs[2].upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[3], &mut self.debugs[3].upstream);
    }
}

#[derive(LogicBlock)]
pub struct HLSLevel0 {
    pub upstream: SoCBusResponder<16, 8>,
    router: RouterROM<16, 8, 2>,
    top: HLSLevel1,
    bottom: HLSLevel1,
}

impl Default for HLSLevel0 {
    fn default() -> Self {
        let left = HLSLevel1::default();
        let right = HLSLevel1::default();
        let router = RouterROM::new(["top", "bottom"], [&left, &right]);
        Self {
            upstream: Default::default(),
            router,
            top: left,
            bottom: right,
        }
    }
}

impl Logic for HLSLevel0 {
    #[hdl_gen]
    fn update(&mut self) {
        SoCBusResponder::<16, 8>::link(&mut self.upstream, &mut self.router.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[0], &mut self.top.upstream);
        SoCBusController::<16, 8>::join(&mut self.router.nodes[1], &mut self.bottom.upstream);
    }
}

impl HLSNamedPorts for HLSLevel0 {
    fn ports(&self) -> Vec<String> {
        self.router.ports()
    }
}

#[cfg(test)]
fn mk_hls_tester() -> HLSLevel0 {
    let mut uut = HLSLevel0::default();
    uut.upstream.link_connect_dest();
    uut.connect_all();
    uut
}

#[test]
fn test_nested_router_function_wide_fifo() {
    let uut = mk_hls_tester();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<HLSLevel0>| {
        x.upstream.clock.next = !x.upstream.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<HLSLevel0>| {
        let mut x = sim.init()?;
        x.upstream.reset.next = NO_RESET;
        bus_address_strobe!(sim, x, upstream, 2);
        bus_write_strobe!(sim, x, upstream, 0xDEAD_u16);
        bus_write_strobe!(sim, x, upstream, 0xBEEF_u16);
        bus_address_strobe!(sim, x, upstream, 2);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 10_000, &vcd_path!("nested_router_fifo.vcd"))
        .unwrap();
}

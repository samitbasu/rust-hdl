use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;

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
    router: Router<16, 8, 2>,
    devs: [BridgeTest; 2],
}

impl Default for BridgePair {
    fn default() -> Self {
        let devs = [BridgeTest::default(), BridgeTest::default()];
        Self {
            upstream: Default::default(),
            router: Router::new(["dev_0", "dev_1"], [&devs[0], &devs[1]]),
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
    router: Router<16, 8, 3>,
    pairs: [BridgePair; 2],
    solo: BridgeTest,
}

impl Default for RouterNest {
    fn default() -> Self {
        let pairs = [BridgePair::default(), BridgePair::default()];
        let solo = BridgeTest::default();
        Self {
            upstream: Default::default(),
            router: Router::new(
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
        bus_address_strobe!(sim, x, upstream, 2);
        bus_write_strobe!(sim, x, upstream, 0xDEAD_u16);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<RouterNest>| {
        let mut x = sim.init()?;
        x.pairs[0].devs[1].port_a.ready.next = true;
        x = sim.watch(|x| x.pairs[0].devs[1].port_a.strobe_out.val(), x)?;
        sim_assert!(sim, x.pairs[0].devs[1].port_a.port_out.val() == 0xDEAD_u16, x);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 1_000, &vcd_path!("nested_router.vcd"))
        .unwrap()
}

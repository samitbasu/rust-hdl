use rust_hdl::core::prelude::*;
use rust_hdl::widgets::dff::DFF;

// A simple bus bridge.  It connects to the master on the one side, and
// then exposes a number of device ports on the other side.  Data is
// routed based on the address selection.

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCBusDevice"]
pub struct SoCBusMaster<const D: usize> {
    pub select: Signal<Out, Bit>,
    pub from_master: Signal<Out, Bits<D>>,
    pub to_master: Signal<In, Bits<D>>,
    pub ready: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<Out, Clock>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCBusMaster"]
pub struct SoCBusDevice<const D: usize> {
    pub select: Signal<In, Bit>,
    pub from_master: Signal<In, Bits<D>>,
    pub to_master: Signal<Out, Bits<D>>,
    pub ready: Signal<Out, Bit>,
    pub strobe: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
}

#[derive(LogicBlock)]
pub struct SoCBridge<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusDevice<D>,
    pub address: Signal<In, Bits<A>>,
    pub nodes: [SoCBusMaster<D>; N],
}

impl<const D: usize, const A: usize, const N: usize> Default for SoCBridge<D, A, N> {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            address: Default::default(),
            nodes: array_init::array_init(|_| Default::default()),
        }
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for SoCBridge<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.upstream.ready.next = false;
        self.upstream.to_master.next = 0_usize.into();
        for i in 0_usize..N {
            self.nodes[i].from_master.next = 0_usize.into();
            self.nodes[i].select.next = false;
            self.nodes[i].strobe.next = false;
            self.nodes[i].clock.next = self.upstream.clock.val();
            if self.address.val().index() == i {
                self.nodes[i].from_master.next = self.upstream.from_master.val();
                self.nodes[i].select.next = self.upstream.select.val();
                self.upstream.to_master.next = self.nodes[i].to_master.val();
                self.upstream.ready.next = self.nodes[i].ready.val();
            }
        }
    }
}

#[test]
fn test_bridge_is_synthesizable() {
    let mut uut = SoCBridge::<16, 8, 6>::default();
    uut.upstream.select.connect();
    uut.upstream.ready.connect();
    uut.upstream.from_master.connect();
    uut.upstream.strobe.connect();
    uut.upstream.clock.connect();
    uut.address.connect();
    for ndx in 0..6 {
        uut.nodes[ndx].to_master.connect();
        uut.nodes[ndx].ready.connect();
    }
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("soc_bridge", &vlog).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct SoCMOSIPort<const D: usize> {
    pub bus: SoCBusDevice<D>,
    pub strobe_out: Signal<Out, Bit>,
    pub ready: Signal<In, Bit>,
    pub port_out: Signal<Out, Bits<D>>,
    state: DFF<Bits<D>>,
    active: DFF<Bit>,
    strobe: DFF<Bit>,
}

impl<const D: usize> Logic for SoCMOSIPort<D> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal flops
        self.state.clk.next = self.bus.clock.val();
        self.active.clk.next = self.bus.clock.val();
        self.strobe.clk.next = self.bus.clock.val();
        self.port_out.next = self.state.q.val();
        self.state.d.next = self.state.q.val();
        self.active.d.next = self.bus.select.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.active.q.val() {
            self.bus.ready.next = self.ready.val();
            if self.bus.strobe.val() {
                self.state.d.next = self.bus.from_master.val();
            }
            self.strobe.d.next = self.bus.strobe.val();
        }
        self.bus.to_master.next = 0_usize.into();
    }
}

#[test]
fn test_mosi_port_is_synthesizable() {
    let mut uut = SoCMOSIPort::<8>::default();
    uut.bus.strobe.connect();
    uut.bus.select.connect();
    uut.bus.clock.connect();
    uut.bus.ready.connect();
    uut.bus.from_master.connect();
    uut.ready.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("soc_mosi_bridge", &vlog).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct SoCTest<const D: usize> {
    pub bridge: SoCBridge<D, 3, 4>,
    pub port_a: SoCMOSIPort<D>,
    pub port_b: SoCMOSIPort<D>,
}

impl<const D: usize> Logic for SoCTest<D> {
    #[hdl_gen]
    fn update(&mut self) {
        self.bridge.nodes[0].join(&mut self.port_a.bus);
        self.bridge.nodes[1].join(&mut self.port_b.bus);
    }
}

#[test]
fn test_linking_from_bridge() {
    let mut uut = SoCTest::<8>::default();
    uut.bridge.upstream.ready.connect();
    uut.bridge.upstream.from_master.connect();
    uut.bridge.upstream.clock.connect();
    uut.bridge.upstream.select.connect();
    uut.bridge.upstream.strobe.connect();
    uut.bridge.address.connect();
    for ndx in 0..4 {
        uut.bridge.nodes[ndx].to_master.connect();
        uut.bridge.nodes[ndx].ready.connect();
    }
    uut.port_a.ready.connect();
    uut.port_b.ready.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("bridge", &vlog).unwrap();
}

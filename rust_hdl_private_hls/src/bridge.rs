use crate::bus::*;
use crate::HLSNamedPorts;
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;

// A simple bus bridge.  It connects to the master on the one side, and
// then exposes a number of device ports on the other side.  Data is
// routed based on the address selection.  Note that reset is resynchronized
// by the bridge.  This means different "devices" will come out of reset
// at different times as the reset de-assert propagates through the network.

#[derive(LogicBlock)]
pub struct Bridge<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusResponder<D, A>,
    pub nodes: [SoCPortController<D>; N],
    pub clock_out: Signal<Out, Clock>,
    address_latch: DFF<Bits<A>>,
    _port_names: Vec<String>,
}

impl<const D: usize, const A: usize, const N: usize> Bridge<D, A, N> {
    pub fn new(names: [&str; N]) -> Self {
        assert!(N <= 2_usize.pow(A as u32));
        Self {
            upstream: Default::default(),
            nodes: array_init::array_init(|_| Default::default()),
            clock_out: Default::default(),
            address_latch: Default::default(),
            _port_names: names.iter().map(|x| x.to_string()).collect(),
        }
    }
}

impl<const D: usize, const A: usize, const N: usize> HLSNamedPorts for Bridge<D, A, N> {
    fn ports(&self) -> Vec<String> {
        self._port_names.clone()
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for Bridge<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock_out.next = self.upstream.clock.val();
        self.upstream.ready.next = false;
        self.upstream.to_controller.next = 0.into();
        dff_setup!(self, clock_out, address_latch);
        for i in 0..N {
            self.nodes[i].from_controller.next = 0.into();
            self.nodes[i].select.next = false;
            self.nodes[i].strobe.next = false;
            self.nodes[i].clock.next = self.upstream.clock.val();
            if self.address_latch.q.val().index() == i {
                self.nodes[i].from_controller.next = self.upstream.from_controller.val();
                self.nodes[i].select.next = true;
                self.nodes[i].strobe.next = self.upstream.strobe.val();
                self.upstream.to_controller.next = self.nodes[i].to_controller.val();
                self.upstream.ready.next = self.nodes[i].ready.val();
            }
        }
        if self.upstream.address_strobe.val() {
            self.address_latch.d.next = self.upstream.address.val();
            self.upstream.ready.next = false;
        }
    }
}

#[test]
fn test_bridge_is_synthesizable() {
    let mut uut = Bridge::<16, 8, 6>::new(["a", "b", "c", "d", "e", "f"]);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("soc_bridge", &vlog).unwrap();
}

use crate::core::prelude::*;
use crate::widgets::soc::bus::*;
use crate::widgets::prelude::DFF;

// A simple bus bridge.  It connects to the master on the one side, and
// then exposes a number of device ports on the other side.  Data is
// routed based on the address selection.

#[derive(LogicBlock)]
pub struct Bridge<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusResponder<D, A>,
    pub nodes: [SoCPortController<D>; N],
    address_latch: DFF<Bits<A>>,
}

impl<const D: usize, const A: usize, const N: usize> Default for Bridge<D, A, N> {
    fn default() -> Self {
        assert!(N <= 2_usize.pow(A as u32));
        Self {
            upstream: Default::default(),
            nodes: array_init::array_init(|_| Default::default()),
            address_latch: Default::default(),
        }
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for Bridge<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.upstream.ready.next = false;
        self.upstream.to_controller.next = 0_usize.into();
        self.address_latch.clk.next = self.upstream.clock.val();
        self.address_latch.d.next = self.address_latch.q.val();
        for i in 0_usize..N {
            self.nodes[i].from_controller.next = 0_usize.into();
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
    let mut uut = Bridge::<16, 8, 6>::default();
    uut.upstream.address.connect();
    uut.upstream.address_strobe.connect();
    uut.upstream.ready.connect();
    uut.upstream.from_controller.connect();
    uut.upstream.strobe.connect();
    uut.upstream.clock.connect();
    uut.upstream.address.connect();
    for ndx in 0..6 {
        uut.nodes[ndx].to_controller.connect();
        uut.nodes[ndx].ready.connect();
    }
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("soc_bridge", &vlog).unwrap();
}

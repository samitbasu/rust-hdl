use crate::core::prelude::*;
use crate::hls::bus::{SoCBusController, SoCBusResponder};
use crate::hls::HLSNamedPorts;
use crate::widgets::prelude::*;

// A router allows you to connect multiple bridges to a single master
// Each bridge is assigned a base address (they must be non-overlapping).
// The master then sees each port on the bridge mapped to the offset
// of it's base address.  Note that you can stack routers if needed.

#[derive(LogicBlock)]
pub struct Router<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusResponder<D, A>,
    pub nodes: [SoCBusController<D, A>; N],
    node_start_address: [Constant<Bits<A>>; N],
    node_end_address: [Constant<Bits<A>>; N],
    active: DFF<Bits<8>>,
    virtual_address: DFF<Bits<A>>,
    address_strobe_delay: DFF<Bit>,
    clock: Signal<Local, Clock>,
    _address_map: Vec<String>,
}

impl<const D: usize, const A: usize, const N: usize> HLSNamedPorts for Router<D, A, N> {
    fn ports(&self) -> Vec<String> {
        self._address_map.clone()
    }
}

impl<const D: usize, const A: usize, const N: usize> Router<D, A, N> {
    pub fn new(downstream_names: [&str; N], downstream_devices: [&dyn HLSNamedPorts; N]) -> Self {
        let address_count = downstream_devices
            .iter()
            .map(|x| x.ports().len())
            .collect::<Vec<_>>();
        let mut _address_map = vec![];
        for ndx in 0..N {
            let prefix = downstream_names[ndx];
            _address_map.extend(
                downstream_devices[ndx]
                    .ports()
                    .iter()
                    .map(|x| format!("{}_{}", prefix, x)),
            );
        }
        let zero = Constant::<Bits<A>>::new(0.into());
        let mut node_start_address: [Constant<Bits<A>>; N] = array_init::array_init(|_| zero);
        let mut node_end_address: [Constant<Bits<A>>; N] = array_init::array_init(|_| zero);
        let mut offset = 0;
        for (ndx, count) in address_count.iter().enumerate() {
            assert_ne!(*count, 0);
            node_start_address[ndx] = Constant::<Bits<A>>::new(offset.into());
            node_end_address[ndx] = Constant::<Bits<A>>::new((offset + count).into());
            offset = offset + count;
        }
        Self {
            upstream: Default::default(),
            nodes: array_init::array_init(|_| Default::default()),
            node_start_address,
            node_end_address,
            active: Default::default(),
            virtual_address: Default::default(),
            address_strobe_delay: Default::default(),
            clock: Default::default(),
            _address_map,
        }
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for Router<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock.next = self.upstream.clock.val();
        self.upstream.ready.next = false;
        self.upstream.to_controller.next = 0.into();
        dff_setup!(self, clock, active, virtual_address, address_strobe_delay);
        // Delay the address strobe by 1 clock cycle to allow the virtual address
        // calculation to be pipelined.
        self.address_strobe_delay.d.next = self.upstream.address_strobe.val();
        for i in 0..N {
            self.nodes[i].from_controller.next = 0.into();
            self.nodes[i].address.next = 0.into();
            self.nodes[i].address_strobe.next = false;
            self.nodes[i].strobe.next = false;
            self.nodes[i].clock.next = self.clock.val();
            if (self.upstream.address.val() >= self.node_start_address[i].val())
                & (self.upstream.address.val() < self.node_end_address[i].val())
                & self.upstream.address_strobe.val()
            {
                self.active.d.next = i.into();
                self.virtual_address.d.next =
                    self.upstream.address.val() - self.node_start_address[i].val();
            }
            if self.active.q.val().index() == i {
                self.nodes[i].from_controller.next = self.upstream.from_controller.val();
                self.nodes[i].address.next = self.virtual_address.q.val();
                self.nodes[i].strobe.next = self.upstream.strobe.val();
                self.upstream.to_controller.next = self.nodes[i].to_controller.val();
                self.upstream.ready.next = self.nodes[i].ready.val();
                self.nodes[i].address_strobe.next = self.address_strobe_delay.q.val();
            }
        }
        if self.upstream.address_strobe.val() {
            self.upstream.ready.next = false;
        }
    }
}

#[test]
fn test_router_is_synthesizable() {
    use crate::hls::bridge::Bridge;
    let bridge1 = Bridge::<16, 8, 4>::new(["Top", "Left", "Right", "Bottom"]);
    let bridge2 = Bridge::<16, 8, 2>::new(["Red", "Blue"]);
    let bridge3 =
        Bridge::<16, 8, 6>::new(["Club", "Spades", "Diamond", "Heart", "Joker", "Instruction"]);
    let mut router =
        Router::<16, 8, 3>::new(["Sides", "Colors", "Faces"], [&bridge1, &bridge2, &bridge3]);
    router.connect_all();
    let vlog = generate_verilog(&router);
    yosys_validate("router", &vlog).unwrap();
}

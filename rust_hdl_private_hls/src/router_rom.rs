use crate::bus::{SoCBusController, SoCBusResponder};
use crate::HLSNamedPorts;
use rust_hdl_private_core::prelude::*;
use rust_hdl_private_widgets::prelude::*;
use std::collections::BTreeMap;

// A RouterROM allows you to connect multiple bridges to a single master
// Each bridge is assigned a base address (they must be non-overlapping).
// The master then sees each port on the bridge mapped to the offset
// of it's base address.  Note that you can stack RouterROMs if needed.

#[derive(LogicBlock)]
pub struct RouterROM<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusResponder<D, A>,
    pub nodes: [SoCBusController<D, A>; N],
    node_decode: ROM<Bits<8>, A>,
    virtual_decode: ROM<Bits<A>, A>,
    active: DFF<Bits<8>>,
    virtual_address: DFF<Bits<A>>,
    address_strobe_delay: DFF<Bit>,
    clock: Signal<Local, Clock>,
    _address_map: Vec<String>,
}

impl<const D: usize, const A: usize, const N: usize> HLSNamedPorts for RouterROM<D, A, N> {
    fn ports(&self) -> Vec<String> {
        self._address_map.clone()
    }
}

impl<const D: usize, const A: usize, const N: usize> RouterROM<D, A, N> {
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
        // Make the node decode ROM
        let mut offset = 0;
        let mut node_rom = BTreeMap::new();
        let mut base_rom = BTreeMap::new();
        for (ndx, count) in address_count.iter().enumerate() {
            assert_ne!(*count, 0);
            for addr in 0..*count {
                let address: Bits<A> = (offset + addr).to_bits();
                let node: Bits<8> = ndx.to_bits();
                node_rom.insert(address, node);
                let virtual_address: Bits<A> = addr.to_bits();
                base_rom.insert(address, virtual_address);
            }
            offset = offset + count;
        }
        Self {
            upstream: Default::default(),
            nodes: array_init::array_init(|_| Default::default()),
            node_decode: ROM::new(node_rom),
            virtual_decode: ROM::new(base_rom),
            active: Default::default(),
            virtual_address: Default::default(),
            address_strobe_delay: Default::default(),
            clock: Default::default(),
            _address_map,
        }
    }
}

impl<const D: usize, const A: usize, const N: usize> Logic for RouterROM<D, A, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock.next = self.upstream.clock.val();
        self.upstream.ready.next = false;
        self.upstream.to_controller.next = 0.into();
        dff_setup!(self, clock, active, virtual_address, address_strobe_delay);
        self.node_decode.address.next = self.upstream.address.val();
        self.virtual_decode.address.next = self.upstream.address.val();
        if self.upstream.address_strobe.val() {
            self.active.d.next = self.node_decode.data.val();
            self.virtual_address.d.next = self.virtual_decode.data.val();
        }
        // Delay the address strobe by 1 clock cycle to allow the virtual address
        // calculation to be pipelined.
        self.address_strobe_delay.d.next = self.upstream.address_strobe.val();
        for i in 0..N {
            self.nodes[i].from_controller.next = 0x5ea1.into();
            self.nodes[i].address.next = 0.into();
            self.nodes[i].address_strobe.next = false;
            self.nodes[i].strobe.next = false;
            self.nodes[i].clock.next = self.clock.val();
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
fn test_router_rom_is_synthesizable() {
    use crate::bridge::Bridge;
    let bridge1 = Bridge::<16, 8, 4>::new(["Top", "Left", "Right", "Bottom"]);
    let bridge2 = Bridge::<16, 8, 2>::new(["Red", "Blue"]);
    let bridge3 =
        Bridge::<16, 8, 6>::new(["Club", "Spades", "Diamond", "Heart", "Joker", "Instruction"]);
    let mut router =
        RouterROM::<16, 8, 3>::new(["Sides", "Colors", "Faces"], [&bridge1, &bridge2, &bridge3]);
    router.connect_all();
    let vlog = generate_verilog(&router);
    yosys_validate("router_rom", &vlog).unwrap();
}

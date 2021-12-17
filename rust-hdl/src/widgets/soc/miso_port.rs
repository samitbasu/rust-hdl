// A simple, local bus for attaching stuff together on the FPGA
use crate::core::prelude::*;
use crate::widgets::dff::DFF;
use crate::widgets::soc::bus::LocalBusD;

// An input port simply stores the value written to it's input back to
// the master.  The address comparison logic is registered to improve the
// timing analysis of the bus.
#[derive(LogicBlock)]
pub struct MISOPort<const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub port_in: Signal<In, Bits<D>>,
    pub clock: Signal<In, Clock>,
    pub ready_in: Signal<In, Bit>,
    pub strobe_out: Signal<Out, Bit>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
}

impl<const D: usize, const A: usize> MISOPort<D, A> {
    pub fn new(address: Bits<A>) -> Self {
        Self {
            bus: Default::default(),
            port_in: Default::default(),
            clock: Default::default(),
            ready_in: Default::default(),
            strobe_out: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(address),
        }
    }
}

impl<const D: usize, const A: usize> Logic for MISOPort<D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.address_active.clk.next = self.clock.val();
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        self.bus.to_master.next = 0_usize.into();
        self.bus.ready.next = false;
        self.strobe_out.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = self.ready_in.val();
            self.bus.to_master.next = self.port_in.val();
            self.strobe_out.next = self.bus.strobe.val();
        }
    }
}

#[test]
fn test_local_in_port_is_synthesizable() {
    let mut dev = MISOPort::<16, 8>::new(53_u8.into());
    dev.bus.from_master.connect();
    dev.bus.addr.connect();
    dev.clock.connect();
    dev.bus.strobe.connect();
    dev.port_in.connect();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    println!("{}", vlog);
    yosys_validate("localin", &vlog).unwrap();
}

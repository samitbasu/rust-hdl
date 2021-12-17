use crate::core::prelude::*;
use crate::widgets::dff::DFF;
use crate::widgets::soc::bus::LocalBusD;

// An output port simply stores the value written to that memory location
// by the master.  The value is latched.
// The strobe from the master is also forwarded.  This allows you to
// build logic that knows when the value was changed, or treat the
// strobe like a trigger.
#[derive(LogicBlock)]
pub struct MOSIPort<const D: usize, const A: usize> {
    pub bus: LocalBusD<D, A>,
    pub port_out: Signal<Out, Bits<D>>,
    pub clock: Signal<In, Clock>,
    pub strobe_out: Signal<Out, Bit>,
    pub ready: Signal<In, Bit>,
    state: DFF<Bits<D>>,
    address_active: DFF<Bit>,
    my_address: Constant<Bits<A>>,
    strobe: DFF<Bit>,
}

impl<const D: usize, const A: usize> MOSIPort<D, A> {
    pub fn new(address: Bits<A>) -> Self {
        Self {
            bus: Default::default(),
            port_out: Default::default(),
            clock: Default::default(),
            strobe_out: Default::default(),
            ready: Default::default(),
            state: Default::default(),
            address_active: Default::default(),
            my_address: Constant::new(address),
            strobe: Default::default(),
        }
    }
}

impl<const D: usize, const A: usize> Logic for MOSIPort<D, A> {
    #[hdl_gen]
    fn update(&mut self) {
        self.state.clk.next = self.clock.val();
        self.strobe.clk.next = self.clock.val();
        self.address_active.clk.next = self.clock.val();
        self.port_out.next = self.state.q.val();
        self.state.d.next = self.state.q.val();
        self.address_active.d.next = self.bus.addr.val() == self.my_address.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.address_active.q.val() {
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
fn test_local_out_port_is_synthesizable() {
    let mut dev = MOSIPort::<16, 8>::new(53_u8.into());
    dev.bus.from_master.connect();
    dev.bus.addr.connect();
    dev.clock.connect();
    dev.bus.strobe.connect();
    dev.connect_all();
    dev.ready.connect();
    let vlog = generate_verilog(&dev);
    println!("{}", vlog);
    yosys_validate("localout", &vlog).unwrap();
}

use crate::bus::SoCPortResponder;
use rust_hdl_private_core::prelude::*;

// An output port simply stores the value written to that memory location
// by the master.  The value is latched.
// The strobe from the master is also forwarded.  This allows you to
// build logic that knows when the value was changed, or treat the
// strobe like a trigger.
#[derive(LogicBlock, Default)]
pub struct MOSIPort<const D: usize> {
    pub bus: SoCPortResponder<D>,
    pub port_out: Signal<Out, Bits<D>>,
    pub strobe_out: Signal<Out, Bit>,
    pub ready: Signal<In, Bit>,
    pub clock_out: Signal<Out, Clock>,
    state: DFF<Bits<D>>,
    address_active: DFF<Bit>,
    strobe: DFF<Bit>,
}

impl<const D: usize> Logic for MOSIPort<D> {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock_out.next = self.bus.clock.val();
        dff_setup!(self, clock_out, state, address_active, strobe);
        self.port_out.next = self.state.q.val();
        self.address_active.d.next = self.bus.select.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = self.ready.val() & self.bus.select.val();
            if self.bus.strobe.val() {
                self.state.d.next = self.bus.from_controller.val();
            }
            self.strobe.d.next = self.bus.strobe.val() & self.ready.val();
        }
        self.bus.to_controller.next = 0.into();
    }
}

#[test]
fn test_local_out_port_is_synthesizable() {
    let mut dev = MOSIPort::<16>::default();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    yosys_validate("localout", &vlog).unwrap();
}

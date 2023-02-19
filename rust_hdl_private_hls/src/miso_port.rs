// A simple, local bus for attaching stuff together on the FPGA
use crate::bus::SoCPortResponder;
use rust_hdl__core::prelude::*;

// An input port simply stores the value written to it's input back to
// the master.  The address comparison logic is registered to improve the
// timing analysis of the bus.
#[derive(LogicBlock, Default)]
pub struct MISOPort<const D: usize> {
    pub bus: SoCPortResponder<D>,
    pub port_in: Signal<In, Bits<D>>,
    pub clock_out: Signal<Out, Clock>,
    pub ready_in: Signal<In, Bit>,
    pub strobe_out: Signal<Out, Bit>,
    address_active: DFF<Bit>,
}

impl<const D: usize> Logic for MISOPort<D> {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock_out.next = self.bus.clock.val();
        dff_setup!(self, clock_out, address_active);
        self.address_active.d.next = self.bus.select.val();
        self.bus.to_controller.next = 0.into();
        self.bus.ready.next = false;
        self.strobe_out.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = self.ready_in.val();
            self.bus.to_controller.next = self.port_in.val();
            self.strobe_out.next = self.bus.strobe.val();
        }
    }
}

#[test]
fn test_local_in_port_is_synthesizable() {
    let mut dev = MISOPort::<16>::default();
    dev.connect_all();
    let vlog = generate_verilog(&dev);
    yosys_validate("localin", &vlog).unwrap();
}

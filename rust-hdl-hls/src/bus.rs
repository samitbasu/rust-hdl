// A simple, local bus for attaching stuff together on the FPGA
use rust_hdl_core::prelude::*;

// Ultimately, a device will have multiple ports.  It will represent
// a "chunk" of bus addresses that can be communicated with via the
// controller.  The question is how do those addresses get routed
// and what do they mean.  Suppose we have 5 ports to describe a
// device.  The natural way to integrate those is to stack them
// behind a bridge:
//
//              +-------
//  --- bus --> | port 1
//              | port 2
//              |   |
//              | port 5
//              +-------
//
// This means each port simply needs a "chip-select" type line, and the address
// of the port is N + base, where base is the base address of the bridge.
//
// We can then stack bridges using a router.  The router will need to assign
// non-overlapping addresses to the bridges, and route the traffic based
// on those ranges.  For example, if there are 2 of these 5-port devices
// attached to bridges B1 and B2.  Then we need to do something like
//
//             +--------+              +-------
//             |        | ---- bus --> |  B1
//  -- bus --> |        |              +-------
//             |        | ---- bus --> |  B2
//             +--------+              +-------

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCBusResponder"]
pub struct SoCBusController<const D: usize, const A: usize> {
    pub address: Signal<Out, Bits<A>>,
    pub address_strobe: Signal<Out, Bit>,
    pub from_controller: Signal<Out, Bits<D>>,
    pub to_controller: Signal<In, Bits<D>>,
    pub ready: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<Out, Clock>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCBusController"]
pub struct SoCBusResponder<const D: usize, const A: usize> {
    pub address: Signal<In, Bits<A>>,
    pub address_strobe: Signal<In, Bit>,
    pub from_controller: Signal<In, Bits<D>>,
    pub to_controller: Signal<Out, Bits<D>>,
    pub ready: Signal<Out, Bit>,
    pub strobe: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCPortResponder"]
pub struct SoCPortController<const D: usize> {
    pub select: Signal<Out, Bit>,
    pub from_controller: Signal<Out, Bits<D>>,
    pub to_controller: Signal<In, Bits<D>>,
    pub ready: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<Out, Clock>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "SoCPortController"]
pub struct SoCPortResponder<const D: usize> {
    pub select: Signal<In, Bit>,
    pub from_controller: Signal<In, Bits<D>>,
    pub to_controller: Signal<Out, Bits<D>>,
    pub ready: Signal<Out, Bit>,
    pub strobe: Signal<In, Bit>,
    pub clock: Signal<In, Clock>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOWriteResponder"]
pub struct FIFOWriteController<T: Synth> {
    pub data: Signal<Out, T>,
    pub write: Signal<Out, Bit>,
    pub full: Signal<In, Bit>,
    pub almost_full: Signal<In, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOWriteController"]
pub struct FIFOWriteResponder<T: Synth> {
    pub data: Signal<In, T>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOReadResponder"]
pub struct FIFOReadController<T: Synth> {
    pub data: Signal<In, T>,
    pub read: Signal<Out, Bit>,
    pub empty: Signal<In, Bit>,
    pub almost_empty: Signal<In, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "FIFOReadController"]
pub struct FIFOReadResponder<T: Synth> {
    pub data: Signal<Out, T>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
}

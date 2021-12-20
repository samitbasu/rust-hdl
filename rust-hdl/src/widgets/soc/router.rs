use crate::core::prelude::*;
//use crate::widgets::soc::bus::{SoCBusMaster, SoCBusDevice};

// A router allows you to connect multiple bridges to a single master
// Each bridge is assigned a base address (they must be non-overlapping).
// The master then sees each port on the bridge mapped to the offset
// of it's base address.  Note that you can stack routers if needed.

/*
#[derive(LogicBlock)]
pub struct Router<const D: usize, const A: usize, const N: usize> {
    pub upstream: SoCBusDebvice<D>,
    pub address: Signal<In, Bits<A>>,
    pub nodes: [SoCBusMaster<D>; N],
    pub address_out: [Signal<Out, Bits<A>>]
}
 */

use crate::core::prelude::*;
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
}

// A simple, local bus for attaching stuff together on the FPGA
use crate::core::prelude::*;

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct LocalBusM<const D: usize, const A: usize> {
    pub addr: Signal<Out, Bits<A>>,
    pub from_master: Signal<Out, Bits<D>>,
    pub to_master: Signal<In, Bits<D>>,
    pub ready: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
pub struct LocalBusD<const D: usize, const A: usize> {
    pub addr: Signal<In, Bits<A>>,
    pub from_master: Signal<In, Bits<D>>,
    pub to_master: Signal<Out, Bits<D>>,
    pub ready: Signal<Out, Bit>,
    pub strobe: Signal<In, Bit>,
}

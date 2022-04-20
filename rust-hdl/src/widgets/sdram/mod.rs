pub mod basic_controller;
pub mod buffer;
pub mod burst_controller;
pub mod cmd;
pub mod fifo_sdram;
pub mod timings;

use crate::core::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputBuffer {
    Wired,
    DelayOne,
    DelayTwo,
}

#[derive(LogicInterface, Clone, Debug, Default)]
#[join = "SDRAMDevice"]
pub struct SDRAMDriver<const D: usize> {
    pub clk: Signal<Out, Clock>,
    pub reset: Signal<Out, Reset>,
    pub we_not: Signal<Out, Bit>,
    pub cas_not: Signal<Out, Bit>,
    pub ras_not: Signal<Out, Bit>,
    pub cs_not: Signal<Out, Bit>,
    pub bank: Signal<Out, Bits<2>>,
    pub address: Signal<Out, Bits<13>>,
    pub write_data: Signal<Out, Bits<D>>,
    pub read_data: Signal<In, Bits<D>>,
    pub write_enable: Signal<Out, Bit>,
}

#[derive(LogicInterface, Clone, Debug, Default)]
#[join = "SDRAMDriver"]
pub struct SDRAMDevice<const D: usize> {
    pub clk: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    pub we_not: Signal<In, Bit>,
    pub cas_not: Signal<In, Bit>,
    pub ras_not: Signal<In, Bit>,
    pub cs_not: Signal<In, Bit>,
    pub bank: Signal<In, Bits<2>>,
    pub address: Signal<In, Bits<13>>,
    pub write_data: Signal<In, Bits<D>>,
    pub read_data: Signal<Out, Bits<D>>,
    pub write_enable: Signal<In, Bit>,
}

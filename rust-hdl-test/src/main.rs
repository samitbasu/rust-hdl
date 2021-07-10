use rust_hdl_core::bits::{Bit, Bits};
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constant::Constant;
use rust_hdl_core::dff::DFF;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::hdl_gen;
use rust_hdl_macros::LogicBlock;
use strobe::Strobe;

mod nested_ports;
mod strobe;
mod base_tests;

fn main() {
    let x = crate::strobe::Strobe::<4>::default();
    let y = x.hdl();
    println!("{:?}", y);
}

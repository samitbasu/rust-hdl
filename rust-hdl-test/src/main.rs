use rust_hdl_core::logic::Logic;

mod base_tests;
mod fifo;
mod nested_ports;
mod strobe;

fn main() {
    let x = crate::strobe::Strobe::<4>::default();
    let y = x.hdl();
    println!("{:?}", y);
}

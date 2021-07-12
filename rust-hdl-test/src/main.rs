use rust_hdl_core::block::Block;
use rust_hdl_core::logic::Logic;
use rust_hdl_core::simulate::{Endpoint, Simulation};
use rust_hdl_macros::LogicBlock;
use rust_hdl_widgets::strobe::Strobe;
use std::fs::File;

mod base_tests;
mod fifo;
mod nested_ports;

#[derive(LogicBlock)]
struct UUT {
    strobe: Strobe<32>,
}

impl Logic for UUT {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.strobe.enable.connect();
        self.strobe.clock.connect();
    }
}

#[test]
fn test_strobe() {
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut UUT| x.strobe.clock.next = !x.strobe.clock.val());
    sim.add_testbench(|mut ep: Endpoint<UUT>| {
        let mut x = ep.init()?;
        x.strobe.enable.next = true;
        x = ep.wait(10_000_000, x)?;
        ep.done(x)?;
        Ok(())
    });
    let mut uut = UUT {
        strobe: Strobe::new(1_000, 10),
    };
    uut.connect_all();
    sim.run_traced(uut, 100_000, File::create("strobe.vcd").unwrap())
        .unwrap();
}

fn main() {
    let x = rust_hdl_widgets::strobe::Strobe::<16>::new(100, 1);
    let y = x.hdl();
    println!("{:?}", y);
}

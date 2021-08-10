use std::fs::File;

use rust_hdl_core::prelude::*;
use rust_hdl_macros::LogicBlock;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::strobe::Strobe;

pub mod alchitry_cu_icepll;
pub mod alchitry_cu_pulser;
pub mod alchitry_cu_pulser_pll;
pub mod alchitry_cu_pwm;
pub mod alchitry_cu_pwm_vec;
pub mod alchitry_cu_pwm_vec_srom;
pub mod base_tests;
pub mod fifo;
pub mod nested_ports;
pub mod ok_tools;
pub mod opalkelly_xem_6010_blinky;
pub mod opalkelly_xem_6010_wave;
pub mod opalkelly_xem_6010_wire;
pub mod pwm;
pub mod rom;
pub mod snore;
pub mod sync_rom;

make_domain!(Mhz1, 1_000_000);

#[derive(LogicBlock)]
struct UUT {
    strobe: Strobe<Mhz1, 32>,
}

impl Logic for UUT {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.strobe.enable.connect();
        self.strobe.clock.connect();
    }
}

#[test]
fn test_strobe_as_verilog() {
    let mut uut: Strobe<Mhz1, 32> = Strobe::new(10.0);
    uut.enable.connect();
    uut.clock.connect();
    uut.connect_all();
    check_connected(&uut);
    println!("{}", generate_verilog(&uut));
    let vlog = generate_verilog(&uut);
    yosys_validate("strobe", &vlog).unwrap();
}

#[test]
fn test_strobe() {
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut UUT| x.strobe.clock.next = !x.strobe.clock.val());
    sim.add_testbench(|mut sim: Sim<UUT>| {
        let mut x = sim.init()?;
        x.strobe.enable.next = true.into();
        x = sim.wait(10_000_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    let mut uut = UUT {
        strobe: Strobe::new(10.0),
    };
    uut.connect_all();
    sim.run_traced(uut, 100_000, File::create("strobe.vcd").unwrap())
        .unwrap();
}

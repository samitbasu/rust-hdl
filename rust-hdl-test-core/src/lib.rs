use std::fs::File;
use std::time::Duration;

use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use rust_hdl_yosys_synth::yosys_validate;

pub mod base_tests;
pub mod edge_detector;
pub mod expander;
pub mod fader;
pub mod fifo;
pub mod nested_ports;
pub mod pwm;
pub mod ram;
pub mod reducer;
pub mod rom;
pub mod snore;
pub mod spi;
pub mod sync_rom;

const MHZ1: u64 = 1_000_000;

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
fn test_strobe_as_verilog() {
    let mut uut: Strobe<32> = Strobe::new(MHZ1, 10.0);
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
    sim.add_clock(5, |x: &mut Box<UUT>| {
        x.strobe.clock.next = !x.strobe.clock.val()
    });
    sim.add_testbench(|mut sim: Sim<UUT>| {
        let mut x = sim.init()?;
        x.strobe.enable.next = true;
        x = sim.wait(10_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    let mut uut = UUT {
        strobe: Strobe::new(MHZ1, 10.0),
    };
    uut.connect_all();
    sim.run_traced(Box::new(uut), 100_000, File::create("strobe.vcd").unwrap())
        .unwrap();
}

#[test]
fn test_shot() {
    let mut shot: Shot<32> = Shot::new(1_000_000, Duration::from_millis(1));
    shot.trigger.connect();
    shot.clock.connect();
    shot.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<Shot<32>>| x.clock.next = !x.clock.val());
    sim.add_testbench(|mut sim: Sim<Shot<32>>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.trigger.next = true;
        wait_clock_cycle!(sim, clock, x);
        x.trigger.next = false;
        x = sim.watch(|x| x.fired.val(), x)?;
        wait_clock_cycle!(sim, clock, x);
        sim_assert!(sim, !x.fired.val(), x);
        wait_clock_cycle!(sim, clock, x);
        sim.done(x)
    });
    sim.run_traced(
        Box::new(shot),
        10_0000,
        std::fs::File::create("shot.vcd").unwrap(),
    )
    .unwrap();
}

use rust_hdl_macros::LogicBlock;
use rust_hdl_widgets::strobe::Strobe;
use rust_hdl_widgets::shot::Shot;
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::clock::Clock;
use rust_hdl_core::bits::Bit;
use rust_hdl_macros::hdl_gen;
use rust_hdl_core::simulate::{Simulation, Sim};
use std::fs::File;
use rust_hdl_core::block::Block;
use rust_hdl_core::module_defines::generate_verilog;
use rust_hdl_synth::yosys_synthesis;
use std::io::Write;

#[derive(LogicBlock)]
struct Pulser {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<32>,
    shot: Shot<32>,
}

impl Pulser {
    pub fn new(clock_freq: u64, pulse_rate: u64, pulse_duration_clocks: u64) -> Self {
        let strobe = Strobe::new(clock_freq, pulse_rate);
        let shot = Shot::new(pulse_duration_clocks);
        Self {
            clock: Signal::default(),
            enable: Signal::default(),
            pulse: Signal::new_with_default(false),
            strobe,
            shot
        }
    }
}

impl Logic for Pulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.strobe.clock.next = self.clock.val();
        self.shot.clock.next = self.clock.val();
        self.strobe.enable.next = self.enable.val();
        self.shot.trigger.next = self.strobe.strobe.val();
        self.pulse.next = self.shot.active.val();
    }
}

#[test]
fn test_pulser() {
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Pulser|
        x.clock.next = !x.clock.val()
    );
    sim.add_testbench(|mut sim: Sim<Pulser>| {
        let mut x = sim.init()?;
        x.enable.next = true;
        x = sim.wait(10_000_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    let mut uut = Pulser::new(10_000, 100, 10);
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    //sim.run_traced(uut, 100_000, File::create("pulser.vcd").unwrap());
    println!("{}", generate_verilog(&uut));
    let mut file = File::create("pulser.v").unwrap();
    let vlog = generate_verilog(&uut);
    write!(file, "{}", vlog);
    yosys_synthesis("pulser", &vlog).unwrap();
    //sim.run(uut, 1_000_000);
}
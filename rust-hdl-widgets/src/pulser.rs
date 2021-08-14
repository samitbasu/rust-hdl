use crate::shot::Shot;
use crate::strobe::Strobe;
use rust_hdl_core::prelude::*;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct Pulser<const CLOCK: u64> {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<CLOCK, 32>,
    shot: Shot<CLOCK, 32>,
}

impl<const CLOCK: u64> Pulser<CLOCK> {
    pub fn new(pulse_rate_hz: f64, pulse_duration: Duration) -> Self {
        let strobe = Strobe::new(pulse_rate_hz);
        let shot = Shot::new(pulse_duration);
        Self {
            clock: Signal::default(),
            enable: Signal::default(),
            pulse: Signal::new_with_default(false),
            strobe,
            shot,
        }
    }
}

impl<const CLOCK: u64> Logic for Pulser<CLOCK> {
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
fn test_pulser_synthesis() {
    use rust_hdl_synth::yosys_validate;
    let mut uut: Pulser<1_000_000> = Pulser::new(1.0, Duration::from_millis(100));
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pulser", &vlog).unwrap();
}

#[test]
fn test_pulser() {
    let mut sim = Simulation::new();
    const KHZ10: u64 = 10_000;
    sim.add_clock(5, |x: &mut Pulser<KHZ10>| x.clock.next = !x.clock.val());
    sim.add_testbench(|mut sim: Sim<Pulser<KHZ10>>| {
        let mut x = sim.init()?;
        x.enable.next = true;
        x = sim.wait(10_000_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    let mut uut: Pulser<KHZ10> = Pulser::new(100.0, Duration::from_millis(100));
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    sim.run_traced(uut, 100_000, std::fs::File::create("pulser.vcd").unwrap())
        .unwrap();
}

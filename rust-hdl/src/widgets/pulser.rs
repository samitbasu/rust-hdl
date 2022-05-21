use crate::core::prelude::*;
use crate::widgets::shot::Shot;
use crate::widgets::strobe::Strobe;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct Pulser {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<32>,
    shot: Shot<32>,
}

impl Pulser {
    pub fn new(clock_rate_hz: u64, pulse_rate_hz: f64, pulse_duration: Duration) -> Self {
        let strobe = Strobe::new(clock_rate_hz, pulse_rate_hz);
        let shot = Shot::new(clock_rate_hz, pulse_duration);
        Self {
            clock: Signal::default(),
            enable: Signal::default(),
            pulse: Signal::new_with_default(false),
            strobe,
            shot,
        }
    }
}

impl Logic for Pulser {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, strobe, shot);
        self.strobe.enable.next = self.enable.val();
        self.shot.trigger.next = self.strobe.strobe.val();
        self.pulse.next = self.shot.active.val();
    }
}

#[test]
fn test_pulser_synthesis() {
    let mut uut = Pulser::new(1_000_000, 1.0, Duration::from_millis(100));
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pulser", &vlog).unwrap();
}

#[test]
fn test_pulser() {
    let mut sim = Simulation::new();
    const KHZ10: u64 = 10_000;
    sim.add_clock(5, |x: &mut Box<Pulser>| x.clock.next = !x.clock.val());
    sim.add_testbench(|mut sim: Sim<Pulser>| {
        let mut x = sim.init()?;
        x.enable.next = true;
        x = sim.wait(100_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    sim.add_testbench(|mut sim: Sim<Pulser>| {
        let mut x = sim.init()?;
        for _j in 0..20 {
            x = sim.watch(|x| x.pulse.val(), x)?;
            for _i in 0..10 {
                wait_clock_cycle!(sim, clock, x);
                sim_assert!(sim, x.pulse.val(), x);
            }
            wait_clock_cycle!(sim, clock, x);
            sim_assert!(sim, !x.pulse.val(), x);
        }
        sim.done(x)?;
        Ok(())
    });
    let mut uut = Pulser::new(KHZ10, 100.0, Duration::from_millis(1));
    uut.connect_all();
    sim.run(Box::new(uut), 1_000_000).unwrap();
}

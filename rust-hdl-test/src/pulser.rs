use rust_hdl_core::prelude::*;
use rust_hdl_widgets::shot::Shot;
use rust_hdl_widgets::strobe::Strobe;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct Pulser<F: Domain> {
    pub clock: Signal<In, Clock, F>,
    pub enable: Signal<In, Bit, F>,
    pub pulse: Signal<Out, Bit, F>,
    strobe: Strobe<F, 32>,
    shot: Shot<F, 32>,
}

impl<F: Domain> Pulser<F> {
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

impl<F: Domain> Logic for Pulser<F> {
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
    make_domain!(Hz100, 100);
    use rust_hdl_synth::yosys_validate;
    let mut uut: Pulser<Hz100> = Pulser::new(1.0, Duration::from_millis(100));
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pulser", &vlog).unwrap();
}

#[test]
fn test_pulser() {
    make_domain!(Khz10, 10_000);

    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Pulser<Khz10>| x.clock.next = !x.clock.val());
    sim.add_testbench(|mut sim: Sim<Pulser<Khz10>>| {
        let mut x = sim.init()?;
        x.enable.next = true.into();
        x = sim.wait(10_000_000, x)?;
        sim.done(x)?;
        Ok(())
    });
    let mut uut: Pulser<Khz10> = Pulser::new(100.0, Duration::from_millis(100));
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    sim.run_traced(uut, 100_000, std::fs::File::create("pulser.vcd").unwrap())
        .unwrap();
}

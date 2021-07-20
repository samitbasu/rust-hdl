use rust_hdl_core::prelude::*;
use rust_hdl_widgets::shot::Shot;
use rust_hdl_widgets::strobe::Strobe;

#[derive(LogicBlock)]
pub struct Pulser {
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
            shot,
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
fn test_pulser_synthesis() {
    use rust_hdl_synth::yosys_validate;
    let mut uut = Pulser::new(100_000_000, 1, 10_000_000);
    uut.clock.connect();
    uut.enable.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pulser", &vlog).unwrap();
}

#[test]
fn test_pulser() {
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Pulser| x.clock.next = !x.clock.val());
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
    sim.run_traced(uut, 100_000, std::fs::File::create("pulser.vcd").unwrap()).unwrap();
}

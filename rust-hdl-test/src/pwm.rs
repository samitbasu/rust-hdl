use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::pwm::PulseWidthModulator;
use std::fs::File;

make_domain!(Mhz1, 1_000_000);

#[derive(LogicBlock)]
struct PWMTest {
    pub clock: Signal<In, Clock, Mhz1>,
    pub pwm: PulseWidthModulator<Mhz1, 8>,
}

impl Default for PWMTest {
    fn default() -> Self {
        Self {
            clock: Signal::default(),
            pwm: PulseWidthModulator::default(),
        }
    }
}

impl Logic for PWMTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.pwm.clock.next = self.clock.val();
        self.pwm.enable.next = true.into();
        self.pwm.threshold.next = 32_u32.into();
    }
}

#[test]
fn test_pwm_circuit() {
    let mut uut = PWMTest::default();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("pwm", &generate_verilog(&uut)).unwrap();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut PWMTest| x.clock.next = !x.clock.val());
    sim.add_testbench(|mut sim: Sim<PWMTest>| {
        let mut x = sim.init()?;
        let mut accum = 0;
        for _ndx in 0..256 {
            x = sim.wait(10, x)?;
            if x.pwm.active.val().raw() {
                accum += 1;
            }
        }
        sim.done(x)?;
        assert_eq!(accum, 32);
        Ok(())
    });
    sim.run_traced(uut, 512 * 10, File::create("pwm.vcd").unwrap())
        .unwrap();
}

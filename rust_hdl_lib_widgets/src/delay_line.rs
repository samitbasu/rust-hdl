use crate::{dff::DFF, dff_setup};
use array_init::array_init;
use rust_hdl_lib_core::prelude::*;

// A configurable delay line.  Given writes at the input,
// will write those values back to the output N cycles later,
// where N is an input of max bit width W.
#[derive(LogicBlock)]
pub struct DelayLine<D: Synth, const N: usize, const W: usize> {
    pub clock: Signal<In, Clock>,
    pub data_in: Signal<In, D>,
    pub data_out: Signal<Out, D>,
    pub delay: Signal<In, Bits<W>>,
    line: [DFF<D>; N],
}

impl<D: Synth, const N: usize, const W: usize> Default for DelayLine<D, N, W> {
    fn default() -> Self {
        assert!(W >= clog2(N));
        Self {
            clock: Default::default(),
            data_in: Default::default(),
            data_out: Default::default(),
            delay: Default::default(),
            line: array_init(|_| Default::default()),
        }
    }
}

impl<D: Synth, const N: usize, const W: usize> Logic for DelayLine<D, N, W> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock all of the delay lines
        for i in 0..N {
            self.line[i].clock.next = self.clock.val();
        }
        for i in 1..N {
            self.line[i].d.next = self.line[i - 1].q.val();
        }
        // Connect the head to the input signal
        self.line[0].d.next = self.data_in.val();
        // Connect the delay line to the appropriate output
        self.data_out.next = self.data_in.val();
        for i in 0..N {
            if self.delay.val().index() == i + 1 {
                self.data_out.next = self.line[i].q.val();
            }
        }
    }
}

#[cfg(test)]
type DelayLineTest = DelayLine<Bits<8>, 8, 3>;

#[test]
fn test_delay_synthesizes() {
    let mut uut = DelayLineTest::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("delay_line", &vlog).unwrap();
}

#[test]
fn test_delay_operation() {
    let mut uut = DelayLineTest::default();
    uut.connect_all();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<DelayLineTest>| {
        x.clock.next = !x.clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<DelayLineTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        x.delay.next = 0.into();
        x.data_in.next = 0xDE.into();
        wait_clock_false!(sim, clock, x);
        sim_assert!(sim, x.data_out.val() == 0xDE, x);
        wait_clock_true!(sim, clock, x);
        x.data_in.next = 0.into();
        wait_clock_false!(sim, clock, x);
        sim_assert!(sim, x.data_out.val() == 0x0, x);
        wait_clock_true!(sim, clock, x);
        for delay in 1..7 {
            wait_clock_cycles!(sim, clock, x, 4);
            x.delay.next = delay.into();
            x.data_in.next = (0xDE + delay).into();
            wait_clock_cycle!(sim, clock, x);
            x.data_in.next = 0x00.into();
            wait_clock_cycles!(sim, clock, x, delay - 1);
            sim_assert!(sim, x.data_out.val() == (0xDE + delay), x);
        }
        sim.done(x)
    });
    sim.run(Box::new(uut), 2_000).unwrap();
}

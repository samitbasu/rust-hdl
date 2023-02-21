use rust_hdl_private_core::prelude::*;

use crate::{dff::DFF, dff_setup};

/// A [Strobe] generates a periodic pulse train, with a single clock-cycle wide pulse
/// at the prescribed frequency.  The argument [N] of the generic [Strobe<N>] is used
/// to size the counter that stores the internal delay value.  Unfortunately, Rust const
/// generics are currently not good enough to compute [N] on the fly.  However, a compile
/// time assert ensures that the number of clock cycles between pulses does not overflow
/// the [N]-bit wide register inside the [Strobe].
#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const N: usize> {
    /// Set this to true to enable the pulse train.
    pub enable: Signal<In, Bit>,
    /// This is the strobing signal - it will fire for 1 clock cycle such that the strobe frequency is generated.
    pub strobe: Signal<Out, Bit>,
    /// The clock that drives the [Strobe].  All signals are synchronous to this clock.
    pub clock: Signal<In, Clock>,
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    /// Generate a [Strobe] widget that can be used in a RustHDL circuit.
    ///
    /// # Arguments
    ///
    /// * `frequency`: The frequency (in Hz) of the clock signal driving the circuit.
    /// * `strobe_freq_hz`: The desired frequency in Hz of the output strobe.  Note that
    /// the strobe frequency will be rounded to something that can be obtained by dividing
    /// the input clock by an integer.  As such, it may not produce exactly the desired
    /// frequency, unless `frequency`/`strobe_freq_hz` is an integer.
    ///
    /// returns: Strobe<{ N }>
    ///
    /// # Examples
    ///
    /// See [BlinkExample] for an example.
    pub fn new(frequency: u64, strobe_freq_hz: f64) -> Self {
        let clock_duration_femto = freq_hz_to_period_femto(frequency as f64);
        let strobe_interval_femto = freq_hz_to_period_femto(strobe_freq_hz);
        let interval = strobe_interval_femto / clock_duration_femto;
        let threshold = interval.round() as u64;
        assert!((threshold as u128) < (1_u128 << (N as u128)));
        assert!(threshold > 2);
        Self {
            enable: Signal::default(),
            strobe: Signal::default(),
            clock: Signal::default(),
            threshold: Constant::new(threshold.into()),
            counter: Default::default(),
        }
    }
}

impl<const N: usize> Logic for Strobe<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the counter clock to my clock
        dff_setup!(self, clock, counter);
        if self.enable.val() {
            self.counter.d.next = self.counter.q.val() + 1;
        }
        self.strobe.next = self.enable.val() & (self.counter.q.val() == self.threshold.val());
        if self.strobe.val() {
            self.counter.d.next = 1.into();
        }
    }
}

use std::num::Wrapping;

use rust_hdl::prelude::freq_hz_to_period_femto;
use serde::Serialize;

use crate::synchronous::{NoTrace, Synchronous, Tracer};

pub struct StrobeConfig {
    threshold: u32,
}

impl StrobeConfig {
    pub fn new(frequency: u64, strobe_freq_hz: f64) -> Self {
        let clock_duration_femto = freq_hz_to_period_femto(frequency as f64);
        let strobe_interval_femto = freq_hz_to_period_femto(strobe_freq_hz);
        let interval = strobe_interval_femto / clock_duration_femto;
        let threshold = interval.round() as u64;
        assert!((threshold as u128) < (1_u128 << 32));
        assert!(threshold > 2);
        Self {
            threshold: threshold as u32,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub struct StrobeState(u32);

impl Synchronous for StrobeConfig {
    type State = StrobeState;
    type Input = bool;
    type Output = bool;

    fn update(
        &self,
        tracer: impl Tracer,
        state_q: StrobeState,
        enable: bool,
    ) -> (bool, StrobeState) {
        let _module = tracer.module("strobe");
        let counter = if enable {
            (Wrapping(state_q.0) + Wrapping(1)).0
        } else {
            state_q.0
        };
        let strobe = enable & (state_q.0 == self.threshold);
        let state_d = StrobeState(if strobe { 1 } else { counter });
        (strobe, state_d)
    }

    fn default_output(&self) -> Self::Output {
        false
    }
}

#[test]
fn test_strobe() {
    // Final state: Strobe { counter: 1000 }, elapsed time 11678, pulse count 999999
    // We want a strobe every 1000 clock cycles.
    let constants = StrobeConfig { threshold: 1000 };
    let mut state = StrobeState::default();
    let mut num_pulses = 0;
    let mut output = false;
    let now = std::time::Instant::now();
    let tracer = NoTrace {};
    for _cycle in 0..1_000_000_000 {
        (output, state) = constants.update(&tracer, state, true);
        if output {
            num_pulses += 1;
        }
    }
    println!(
        "Final state: {state:?}, elapsed time {}, pulse count {num_pulses}",
        now.elapsed().as_millis()
    );
}

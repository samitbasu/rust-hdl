use std::time::Duration;

use rust_hdl::prelude::{freq_hz_to_period_femto, Bits, NANOS_PER_FEMTO};

use crate::synchronous::Synchronous;

pub struct ShotConfig<const N: usize> {
    duration: Bits<N>,
}

impl<const N: usize> ShotConfig<N> {
    pub fn new(frequency: u64, duration: Duration) -> Self {
        let duration_nanos = duration.as_nanos() as f64 * NANOS_PER_FEMTO; // duration in femtos
        let clock_period_nanos = freq_hz_to_period_femto(frequency as f64);
        let clocks = (duration_nanos / clock_period_nanos).floor() as u64;
        assert!(clocks < (1_u64 << 32));
        Self {
            duration: clocks.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ShotState<const N: usize> {
    counter: Bits<N>,
    state: bool,
}

#[derive(Default)]
pub struct ShotOutputs {
    pub active: bool,
    pub fired: bool,
}

impl<const N: usize> Synchronous for ShotConfig<N> {
    type Input = bool;
    type Output = ShotOutputs;
    type State = ShotState<N>;

    fn update(&self, state_q: ShotState<N>, trigger: bool) -> (ShotOutputs, ShotState<N>) {
        let ShotState {
            counter: counter_q,
            state: state_q,
        } = state_q;
        let mut counter_d = if state_q { counter_q + 1 } else { counter_q };
        let mut outputs: ShotOutputs = Default::default();
        let mut state_d = state_q;
        if state_q && counter_q == self.duration {
            state_d = false;
            outputs.fired = true;
        }
        outputs.active = state_q;
        if trigger {
            state_d = true;
            counter_d = 0.into();
        }
        let state_d = ShotState {
            counter: counter_d,
            state: state_d,
        };
        (outputs, state_d)
    }
}

#[test]
fn test_shot() {
    let shot_config = ShotConfig::<32> {
        duration: 100.into(),
    };
    let mut state = ShotState::default();
    let mut output: ShotOutputs = Default::default();
    let mut shot_on = 0;
    let mut trig_count = 0;
    let now = std::time::Instant::now();
    for clk in 0..1_000_000_000 {
        (output, state) = shot_config.update(state, clk % 1000 == 0);
        if output.active {
            shot_on += 1;
        }
        if output.fired {
            trig_count += 1;
        }
    }
    println!(
        "Final state: {state:?} elapsed time {} shot on {shot_on} trig_count {trig_count}",
        now.elapsed().as_millis()
    )
}

use std::time::Duration;

use serde::Serialize;

use crate::{
    shot::{ShotConfig, ShotState},
    strobe::{StrobeConfig, StrobeState},
    synchronous::{NoTrace, Synchronous, Tracer},
};

pub struct PulserConfig {
    strobe: StrobeConfig,
    shot: ShotConfig<32>,
}

impl PulserConfig {
    pub fn new(clock_rate_hz: u64, pulse_rate_hz: f64, pulse_duration: Duration) -> Self {
        let strobe = StrobeConfig::new(clock_rate_hz, pulse_rate_hz);
        let shot = ShotConfig::new(clock_rate_hz, pulse_duration);
        Self { strobe, shot }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PulserState {
    strobe: StrobeState,
    shot: ShotState<32>,
}

impl Synchronous for PulserConfig {
    type Input = bool;
    type Output = bool;
    type State = PulserState;

    fn update(&self, t: impl Tracer, q: Self::State, enable: bool) -> (Self::Output, Self::State) {
        let _module = t.module("pulser");
        let (strobe_output, d_strobe) = self.strobe.update(&t, q.strobe, enable);
        let (shot_outputs, d_shot) = self.shot.update(&t, q.shot, strobe_output);
        let pulse = shot_outputs.active;
        (
            pulse,
            PulserState {
                strobe: d_strobe,
                shot: d_shot,
            },
        )
    }

    fn default_output(&self) -> Self::Output {
        false
    }
}

#[test]
fn test_pulser() {
    let config = PulserConfig::new(1_000_000_000, 1_00.0, Duration::from_millis(1));
    let mut state = PulserState::default();
    let mut time_high = 0;
    let mut output;
    let now = std::time::Instant::now();
    let tracer = NoTrace {};
    for _cycle in 0..1_000_000_000 {
        (output, state) = config.update(&tracer, state, true);
        if output {
            time_high += 1;
        }
    }
    println!(
        "Final state: {state:?}, elapsed time {}, time high {time_high}",
        now.elapsed().as_millis()
    )
}

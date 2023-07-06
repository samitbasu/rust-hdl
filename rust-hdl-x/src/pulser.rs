use std::time::Duration;

use crate::{
    shot::{ShotConfig, ShotState},
    strobe::{StrobeConfig, StrobeState},
    synchronous::Synchronous,
};

pub struct PulserConfig {
    strobe: StrobeConfig,
    shot: ShotConfig,
}

impl PulserConfig {
    pub fn new(clock_rate_hz: u64, pulse_rate_hz: f64, pulse_duration: Duration) -> Self {
        let strobe = StrobeConfig::new(clock_rate_hz, pulse_rate_hz);
        let shot = ShotConfig::new(clock_rate_hz, pulse_duration);
        Self { strobe, shot }
    }
}

#[derive(Default, Debug)]
pub struct PulserState {
    strobe: StrobeState,
    shot: ShotState,
}

impl Synchronous for PulserConfig {
    type Input = bool;
    type Output = bool;
    type State = PulserState;

    fn update(&self, state: Self::State, enable: bool) -> (Self::Output, Self::State) {
        let PulserState {
            strobe: strobe_q,
            shot: shot_q,
        } = state;
        let (strobe_output, strobe_d) = self.strobe.update(strobe_q, enable);
        let (shot_outputs, shot_d) = self.shot.update(shot_q, strobe_output);
        let pulse = shot_outputs.active;
        (
            pulse,
            PulserState {
                strobe: strobe_d,
                shot: shot_d,
            },
        )
    }
}

#[test]
fn test_pulser() {
    let config = PulserConfig::new(1_000_000_000, 1_00.0, Duration::from_millis(1));
    let mut state = PulserState::default();
    let mut time_high = 0;
    let mut output;
    let now = std::time::Instant::now();
    for _cycle in 0..1_000_000_000 {
        (output, state) = config.update(state, true);
        if output {
            time_high += 1;
        }
    }
    println!(
        "Final state: {state:?}, elapsed time {}, time high {time_high}",
        now.elapsed().as_millis()
    )
}

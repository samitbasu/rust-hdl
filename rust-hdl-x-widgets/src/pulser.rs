use std::time::Duration;

use rust_hdl::prelude::freq_hz_to_period_femto;
use rust_hdl_x::{
    basic_logger_builder::BasicLoggerBuilder, single_clock_simulation, synchronous::Synchronous,
    LogBuilder, Loggable, Logger, TagID,
};

use crate::{
    shot::{ShotConfig, ShotState},
    strobe::{StrobeConfig, StrobeState},
};

pub struct PulserConfig<const N: usize> {
    strobe: StrobeConfig,
    shot: ShotConfig<N>,
    tag_input: TagID<bool>,
    tag_output: TagID<bool>,
}

impl<const N: usize> PulserConfig<N> {
    pub fn new(
        clock_rate_hz: u64,
        pulse_rate_hz: f64,
        pulse_duration: Duration,
        mut builder: impl LogBuilder,
    ) -> Self {
        let strobe = StrobeConfig::new(clock_rate_hz, pulse_rate_hz, builder.scope("strobe"));
        let shot = ShotConfig::new(clock_rate_hz, pulse_duration, builder.scope("shot"));
        let tag_input = builder.tag("enable");
        let tag_output = builder.tag("active");
        Self {
            strobe,
            shot,
            tag_input,
            tag_output,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Loggable, PartialEq)]
pub struct PulserState<const N: usize> {
    strobe: StrobeState,
    shot: ShotState<N>,
}

impl<const N: usize> Synchronous for PulserConfig<N> {
    type Input = bool;
    type Output = bool;
    type State = PulserState<N>;

    fn compute(
        &self,
        mut l: impl Logger,
        enable: bool,
        q: Self::State,
    ) -> (Self::Output, Self::State) {
        l.log(self.tag_input, enable);
        let (strobe_output, d_strobe) = self.strobe.compute(&mut l, enable, q.strobe);
        let (shot_outputs, d_shot) = self.shot.compute(&mut l, strobe_output, q.shot);
        let pulse = shot_outputs.active;
        l.log(self.tag_output, pulse);
        (
            pulse,
            PulserState {
                strobe: d_strobe,
                shot: d_shot,
            },
        )
    }
}

#[test]
fn test_pulser() {
    let mut builder = BasicLoggerBuilder::default();
    let clock_period_fs = freq_hz_to_period_femto(100_000_000.0) as u64;
    builder.add_simple_clock(clock_period_fs);
    let config: PulserConfig<32> = PulserConfig::new(
        100_000_000,
        10_000.0,
        Duration::from_micros(25),
        &mut builder,
    );
    let mut logger = builder.build();
    let mut time_high = 0;
    let now = std::time::Instant::now();
    single_clock_simulation(
        &mut logger,
        config,
        clock_period_fs,
        1_000_000,
        |_cycle, output| {
            if output {
                time_high += 1;
            }
            true
        },
    );
    logger.dump();
    println!(
        "Final state: elapsed time {}, time high {time_high}",
        now.elapsed().as_millis()
    );
    let buf = std::io::BufWriter::new(std::fs::File::create("pulser.vcd").unwrap());
    logger.vcd(buf).unwrap();
}

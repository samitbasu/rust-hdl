use rust_hdl::prelude::{freq_hz_to_period_femto, Bits, NANOS_PER_FEMTO};
use rust_hdl_x::{synchronous::Synchronous, LogBuilder, Logger, TagID};
use rust_hdl_x_macro::Loggable;
use std::time::Duration;

pub struct ShotConfig<const N: usize> {
    duration: Bits<N>,
    tag_input: TagID<bool>,
    tag_output: TagID<ShotOutputs>,
    tag_q: TagID<ShotState<N>>,
    tag_d: TagID<ShotState<N>>,
}

impl<const N: usize> ShotConfig<N> {
    pub fn new(frequency: u64, duration: Duration, mut builder: impl LogBuilder) -> Self {
        let duration_nanos = duration.as_nanos() as f64 * NANOS_PER_FEMTO; // duration in femtos
        let clock_period_nanos = freq_hz_to_period_femto(frequency as f64);
        let clocks = (duration_nanos / clock_period_nanos).floor() as u64;
        assert!(clocks < (1_u64 << 32));
        let tag_input = builder.tag("trigger");
        let tag_output = builder.tag("output");
        let tag_q = builder.tag("q");
        let tag_d = builder.tag("d");
        Self {
            duration: clocks.into(),
            tag_input,
            tag_output,
            tag_q,
            tag_d,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Loggable, PartialEq)]
pub struct ShotState<const N: usize> {
    counter: Bits<N>,
    state: bool,
}

#[derive(Default, Clone, Copy, Loggable, PartialEq)]
pub struct ShotOutputs {
    pub active: bool,
    pub fired: bool,
}

impl<const N: usize> Synchronous for ShotConfig<N> {
    type Input = bool;
    type Output = ShotOutputs;
    type State = ShotState<N>;
    fn compute(
        &self,
        mut logger: impl Logger,
        trigger: Self::Input,
        q: Self::State,
    ) -> (Self::Output, Self::State) {
        logger.log(self.tag_input, trigger);
        logger.log(self.tag_q, q);
        let mut d = q;
        d.counter = if q.state { q.counter + 1 } else { q.counter };
        let mut outputs: ShotOutputs = Default::default();
        if q.state && q.counter == self.duration {
            d.state = false;
            outputs.fired = true;
        }
        outputs.active = q.state;
        if trigger {
            d.state = true;
            d.counter = 0.into();
        }
        logger.log(self.tag_output, outputs);
        logger.log(self.tag_d, d);
        (outputs, d)
    }
}

#[test]
fn test_shot() {
    let mut builder = rust_hdl_x::basic_logger_builder::BasicLoggerBuilder::default();
    let period_in_fs = freq_hz_to_period_femto(100_000_000.0);
    let shot_config = ShotConfig::<32>::new(100_000_000, Duration::from_micros(1), &mut builder);
    let mut logger = builder.build();
    let mut shot_on = 0;
    let mut trig_count = 0;
    let now = std::time::Instant::now();
    rust_hdl_x::single_clock_simulation(
        &mut logger,
        shot_config,
        period_in_fs as u64,
        10_000_000,
        |cycle, output| {
            if output.active {
                shot_on += 1;
            }
            if output.fired {
                trig_count += 1;
            }
            cycle % 1000 == 0
        },
    );
    println!(
        "Final state: elapsed time {} shot on {shot_on} trig_count {trig_count}",
        now.elapsed().as_millis()
    );
    let buf = std::io::BufWriter::new(std::fs::File::create("shot.vcd").unwrap());
    logger.vcd(buf).unwrap();
}

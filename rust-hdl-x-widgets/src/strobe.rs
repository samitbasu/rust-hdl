use rust_hdl::prelude::freq_hz_to_period_femto;
use rust_hdl_x::{synchronous::Synchronous, LogBuilder, Loggable, Logger, TagID};

pub struct StrobeConfig {
    threshold: u32,
    tag_input: TagID<bool>,
    tag_output: TagID<bool>,
}

impl StrobeConfig {
    pub fn new(frequency: u64, strobe_freq_hz: f64, mut builder: impl LogBuilder) -> Self {
        let clock_duration_femto = freq_hz_to_period_femto(frequency as f64);
        let strobe_interval_femto = freq_hz_to_period_femto(strobe_freq_hz);
        let interval = strobe_interval_femto / clock_duration_femto;
        let threshold = interval.round() as u64;
        assert!((threshold as u128) < (1_u128 << 32));
        assert!(threshold > 2);
        let tag_input = builder.tag("enable");
        let tag_output = builder.tag("output");
        Self {
            threshold: threshold as u32,
            tag_input,
            tag_output,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Loggable)]
pub struct StrobeState {
    count: u32,
}

impl Synchronous for StrobeConfig {
    type State = StrobeState;
    type Input = bool;
    type Output = bool;

    fn compute(
        &self,
        mut logger: impl Logger,
        enable: bool,
        q: StrobeState,
    ) -> (bool, StrobeState) {
        logger.log(self.tag_input, enable);
        let mut d = q;
        if enable {
            d.count = q.count + 1;
        }
        let strobe = enable & (q.count == self.threshold);
        if strobe {
            d.count = 1;
        }
        logger.log(self.tag_output, strobe);
        (strobe, d)
    }
}

#[test]
fn test_strobe() {
    // Final state: Strobe { counter: 1000 }, elapsed time 11678, pulse count 999999
    // We want a strobe every 1000 clock cycles.
    let mut builder = rust_hdl_x::basic_logger_builder::BasicLoggerBuilder::default();
    let period_in_fs = freq_hz_to_period_femto(100_000_000.0) as u64;
    let config = StrobeConfig::new(100_000_000, 100_000.0, &mut builder);
    builder.add_simple_clock(period_in_fs);
    let mut num_pulses = 0;
    let now = std::time::Instant::now();
    let mut logger = builder.build();
    let num_cycles = 1_000_000;
    rust_hdl_x::single_clock_simulation(
        &mut logger,
        config,
        period_in_fs,
        num_cycles,
        |_cycle, output| {
            if output {
                num_pulses += 1;
            }
            true
        },
    );
    println!(
        "Final state: elapsed time {}, pulse count {num_pulses}",
        now.elapsed().as_millis()
    );
    let buf = std::io::BufWriter::new(std::fs::File::create("strobe.vcd").unwrap());
    logger.vcd(buf).unwrap();
}

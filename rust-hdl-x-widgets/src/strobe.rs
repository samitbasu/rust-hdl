use crate::log::LogBuilder;
use crate::loggable::Loggable;
use crate::{log::TagID, synchronous::Synchronous};
use rust_hdl::prelude::freq_hz_to_period_femto;
use rust_hdl_x_macro::Loggable;

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
        let tag_input = builder.tag("trigger");
        let tag_output = builder.tag("output");
        Self {
            threshold: threshold as u32,
            tag_input,
            tag_output,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Loggable)]
pub struct StrobeState {
    count: u32,
}

impl Synchronous for StrobeConfig {
    type State = StrobeState;
    type Input = bool;
    type Output = bool;

    fn trace_id(&self) -> Option<TraceID> {
        self.trace_id
    }

    fn setup(&mut self, builder: impl TracerBuilder) {
        self.trace_id = Some(Self::register_trace_types(builder));
    }

    fn compute(&self, _t: impl Tracer, enable: bool, q: StrobeState) -> (bool, StrobeState) {
        let mut d = q;
        if enable {
            d.count = q.count + 1;
        }
        let strobe = enable & (q.count == self.threshold);
        if strobe {
            d.count = 1;
        }
        (strobe, d)
    }
}

#[test]
fn test_strobe() {
    // Final state: Strobe { counter: 1000 }, elapsed time 11678, pulse count 999999
    // We want a strobe every 1000 clock cycles.
    let mut config = StrobeConfig {
        threshold: 1000,
        trace_id: None,
    };
    let mut state = StrobeState::default();
    let mut num_pulses = 0;
    let mut output = false;
    let now = std::time::Instant::now();
    let mut builder = crate::basic_tracer::BasicTracerBuilder::default();
    config.setup(&mut builder);
    let mut tracer = builder.build();
    for _cycle in 0..10_000_000 {
        (output, state) = config.update(&mut tracer, true, state);
        if output {
            num_pulses += 1;
        }
    }
    println!(
        "Final state: {state:?}, elapsed time {}, pulse count {num_pulses}",
        now.elapsed().as_millis()
    );
}

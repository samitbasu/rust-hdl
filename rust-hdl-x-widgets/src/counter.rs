use rust_hdl::prelude::Bits;
use rust_hdl_x::{synchronous::Synchronous, LogBuilder, Logger, TagID};

struct BitCounter<const N: usize> {
    tag_input: TagID<bool>,
    tag_output: TagID<Bits<N>>,
}

impl<const N: usize> BitCounter<N> {
    pub fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        Self {
            tag_input,
            tag_output,
        }
    }
}

impl<const N: usize> Synchronous for BitCounter<N> {
    type State = Bits<N>;
    type Input = bool;
    type Output = Bits<N>;

    fn compute(
        &self,
        mut logger: impl Logger,
        input: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        logger.log(self.tag_input, input);
        let new_state = if input { state + 1 } else { state };
        let output = new_state;
        logger.log(self.tag_output, output);
        (output, new_state)
    }
}

#[cfg(test)]
mod tests {
    use rust_hdl::prelude::freq_hz_to_period_femto;

    use super::*;
    #[test]
    fn test_counter_with_bits_argument() {
        let mut logger_builder = rust_hdl_x::basic_logger_builder::BasicLoggerBuilder::default();
        let clock_period = freq_hz_to_period_femto(1e6) as u64;
        logger_builder.add_simple_clock(clock_period);
        let counter: BitCounter<24> = BitCounter::new(&mut logger_builder);
        let logger = logger_builder.build();
        let mut last_output: Bits<24> = Default::default();
        rust_hdl_x::single_clock_simulation(
            logger,
            counter,
            clock_period,
            100_000_000,
            |cycle, output| {
                last_output = output;
                cycle % 2 == 0
            },
        );
        assert_eq!(last_output, (100_000_000 / 2) & 0xFF_FFFF);
    }
}

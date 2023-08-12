use rust_hdl::prelude::Bits;

use crate::{
    log::{LogBuilder, TagID},
    logger::Logger,
    synchronous::Synchronous,
};

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
    use crate::{basic_logger_builder::BasicLoggerBuilder, log::ClockDetails};

    use super::*;
    #[test]
    fn test_counter_with_bits_argument() {
        let mut logger_builder = BasicLoggerBuilder::default();
        let clock_period = 1_000_000_000;
        logger_builder.add_clock(ClockDetails {
            period_in_fs: clock_period,
            offset_in_fs: 0,
            initial_state: false,
        });
        let counter: BitCounter<24> = BitCounter::new(&mut logger_builder);
        let mut logger = logger_builder.build();
        let mut state: Bits<24> = Default::default();
        let mut last_output = Default::default();
        let mut time = 0;
        for cycle in 0..100_000_000 {
            logger.set_time_in_fs(time);
            time += clock_period;
            let (output, new_state) = counter.compute(&mut logger, cycle % 2 == 0, state);
            state = new_state;
            last_output = output;
            //        println!("{} {}", output, state);
        }
        println!(
            "Last output {last_output:x} (vs) {:x}",
            (100_000_000 / 2) & 0xFF_FFFF
        );
    }
}

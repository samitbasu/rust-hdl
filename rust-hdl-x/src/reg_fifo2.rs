use crate::{basic_logger_builder, single_clock_simulation, synchronous::Synchronous, Logger};
use rust_hdl::prelude::freq_hz_to_period_femto;
use rust_hdl_x_macro::Loggable;

use crate::{
    log::{LogBuilder, TagID},
    loggable::Loggable,
};

// A single register with a FIFO interface
pub struct SingleRegisterFIFO {
    tag_input: TagID<Inputs>,
    tag_output: TagID<Outputs>,
}

#[derive(Loggable, Copy, Clone, PartialEq, Default)]
pub struct Inputs {
    pub input: u32,
    pub write: bool,
    pub read: bool,
}

#[derive(Loggable, Default, Copy, Clone, PartialEq)]
pub struct Outputs {
    pub output: u32,
    pub empty: bool,
    pub full: bool,
    pub error: bool,
}

#[derive(Loggable, Copy, Clone, Default, PartialEq)]
pub struct State {
    pub value: u32,
    pub loaded: bool,
    pub error: bool,
}

impl SingleRegisterFIFO {
    pub fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        Self {
            tag_input,
            tag_output,
        }
    }
}

impl Synchronous for SingleRegisterFIFO {
    type State = State;
    type Input = Inputs;
    type Output = Outputs;

    fn compute(
        &self,
        mut tracer: impl Logger,
        inputs: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        tracer.log(self.tag_input, inputs);
        let mut output = Outputs::default();
        let mut next_state = state;
        if inputs.write {
            if state.loaded {
                next_state.error = true;
            } else {
                next_state.value = inputs.input;
                next_state.loaded = true;
            }
        } else if inputs.read {
            if state.loaded {
                output.output = state.value;
                next_state.loaded = false;
            } else {
                next_state.error = true;
            }
        }
        output.empty = !state.loaded;
        output.full = state.loaded;
        output.error = state.error;
        tracer.log(self.tag_output, output);
        (output, next_state)
    }
}

#[test]
fn test_single_register_fifo() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let period_in_fs = freq_hz_to_period_femto(100.0e6) as u64;
    logger_builder.add_simple_clock(period_in_fs);
    let fifo = SingleRegisterFIFO::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    single_clock_simulation(&mut logger, fifo, period_in_fs, 100_000, |cycle, output| {
        if output.full {
            Inputs {
                input: 0,
                write: false,
                read: true,
            }
        } else {
            Inputs {
                input: cycle as u32,
                write: true,
                read: false,
            }
        }
    });
    let mut vcd = std::fs::File::create("single_register_fifo.vcd").unwrap();
    logger.vcd(&mut vcd).unwrap();
}

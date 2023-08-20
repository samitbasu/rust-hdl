use rust_hdl_x::{synchronous::Synchronous, LogBuilder, Loggable, Logger, TagID};
use rust_hdl_x_macro::Loggable;

// A single register with a FIFO interface
pub struct SingleRegisterFIFO<T: Loggable> {
    tag_input: TagID<Inputs<T>>,
    tag_output: TagID<Outputs<T>>,
}

#[derive(Loggable, Copy, Clone, PartialEq, Default)]
pub struct Inputs<T: Loggable> {
    pub input: T,
    pub write: bool,
    pub read: bool,
}

#[derive(Loggable, Default, Copy, Clone, PartialEq)]
pub struct Outputs<T: Loggable> {
    pub output: T,
    pub empty: bool,
    pub full: bool,
    pub error: bool,
}

#[derive(Loggable, Copy, Clone, Default, PartialEq)]
pub struct State<T: Loggable> {
    pub value: T,
    pub loaded: bool,
    pub error: bool,
}

impl<T: Loggable> SingleRegisterFIFO<T> {
    pub fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        Self {
            tag_input,
            tag_output,
        }
    }
}

impl<T: Loggable> Synchronous for SingleRegisterFIFO<T> {
    type State = State<T>;
    type Input = Inputs<T>;
    type Output = Outputs<T>;

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
        }
        if inputs.read {
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
    let mut logger_builder = rust_hdl_x::basic_logger_builder::BasicLoggerBuilder::default();
    let period_in_fs = rust_hdl::prelude::freq_hz_to_period_femto(100.0e6) as u64;
    logger_builder.add_simple_clock(period_in_fs);
    let fifo = SingleRegisterFIFO::<u32>::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    rust_hdl_x::single_clock_simulation(
        &mut logger,
        fifo,
        period_in_fs,
        100_000,
        |cycle, output| {
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
        },
    );
    logger.dump();
    let mut vcd = std::fs::File::create("single_register_fifo.vcd").unwrap();
    logger.vcd(&mut vcd).unwrap();
}

#[test]
fn test_single_register_fifo_complex_struct() {
    #[derive(Copy, Clone, PartialEq, Default, Loggable)]
    struct Complex {
        a: u32,
        b: u32,
    }

    let mut logger_builder = rust_hdl_x::basic_logger_builder::BasicLoggerBuilder::default();
    let period_in_fs = rust_hdl::prelude::freq_hz_to_period_femto(100.0e6) as u64;
    logger_builder.add_simple_clock(period_in_fs);
    let fifo = SingleRegisterFIFO::<Complex>::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    rust_hdl_x::single_clock_simulation(
        &mut logger,
        fifo,
        period_in_fs,
        100_000,
        |cycle, output| {
            if output.full {
                Inputs {
                    input: Complex::default(),
                    write: false,
                    read: true,
                }
            } else {
                Inputs {
                    input: Complex {
                        a: cycle as u32,
                        b: cycle as u32,
                    },
                    write: true,
                    read: false,
                }
            }
        },
    );
    logger.dump();
    let mut vcd = std::fs::File::create("single_register_fifo_complex_struct.vcd").unwrap();
    logger.vcd(&mut vcd).unwrap();
}

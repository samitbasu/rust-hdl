use std::{
    cell::RefCell,
    fmt::{Display, Formatter, LowerHex},
    io::BufWriter,
    rc::Rc,
};

use crate::log::{ClockDetails, LogBuilder};
use crate::loggable::Loggable;
use log::TagID;
use logger::Logger;
use rust_hdl::prelude::{freq_hz_to_period_femto, Bits};
use rust_hdl_x_macro::Loggable;
use synchronous::Synchronous;

//use synchronous::Synchronous;

//mod bit_iter;
//mod bit_slice;
mod counter;
mod rev_bit_iter;
//mod derive_vcd;
//mod pulser;
//pub mod shot;
//mod spi_controller;
//pub mod strobe;
//mod synchronous;
//pub mod basic_tracer;
//pub mod counter;
//pub mod no_trace;
//pub mod shot;
pub mod synchronous;
//pub mod traceable;
//pub mod tracer;
//pub mod tracer_builder;
//mod vcd;
pub mod basic_logger;
pub mod basic_logger_builder;
pub mod log;
pub mod loggable;
pub mod logger;

pub mod reg_fifo;
pub mod reg_fifo2;

#[ignore]
#[test]
fn bits_benchmark() {
    let tic = std::time::Instant::now();
    let x = rust_hdl::core::bits::Bits::<65>::from(0x12345678);
    let y = rust_hdl::core::bits::Bits::<65>::from(0x1);
    let mut z = rust_hdl::core::bits::Bits::<65>::from(0x0);
    for i in 0..1000000 {
        let _ = x.get_bit(i % 32);
        let _ = y.get_bit(i % 32);
        z = z + y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run bit benchmark: {:?}", toc - tic);
}

/* #[ignore]
#[test]
fn uint_benchmark() {
    let tic = std::time::Instant::now();
    let x = uint!(0x12345678_U65);
    let y = uint!(0x1_U65);
    let mut z = uint!(0x0_U65);
    for i in 0..1000000 {
        let _ = x.bit(i % 32);
        let _ = y.bit(i % 32);
        z += y;
    }
    let toc = std::time::Instant::now();
    println!("Time to run uint benchmark: {:?}", toc - tic);
}
 */

#[derive(Debug)]
struct Bar {
    counter: u16,
    tag_input: TagID<u16>,
    tag_output: TagID<bool>,
    tag_state: TagID<u16>,
    tag_next_state: TagID<u16>,
}

impl Bar {
    fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        let tag_state = builder.tag("state");
        let tag_next_state = builder.tag("next_state");
        Self {
            counter: 0,
            tag_input,
            tag_output,
            tag_state,
            tag_next_state,
        }
    }
}

impl Synchronous for Bar {
    type Input = u16;
    type Output = bool;
    type State = u16;

    fn compute(
        &self,
        mut trace: impl Logger,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        let next_state = state + inputs;
        let output = next_state % 2 == 0;
        trace.log(self.tag_input, inputs);
        trace.log(self.tag_output, output);
        trace.log(self.tag_state, state);
        trace.log(self.tag_next_state, next_state);
        (output, next_state)
    }
}

#[derive(Debug)]
struct Foo {
    sub1: Bar,
    sub2: Bar,
    tag_input: TagID<u16>,
    tag_output: TagID<MoreJunk>,
    tag_state: TagID<u16>,
    tag_next_state: TagID<u16>,
}

impl Foo {
    fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        let tag_state = builder.tag("state");
        let tag_next_state = builder.tag("next_state");
        Self {
            sub1: Bar::new(builder.scope("sub1")),
            sub2: Bar::new(builder.scope("sub2")),
            tag_input,
            tag_output,
            tag_state,
            tag_next_state,
        }
    }
}

impl Synchronous for Foo {
    type Input = u16;
    type Output = MoreJunk;
    type State = u16;

    fn compute(
        &self,
        mut logger: impl Logger,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        // Update the submodules
        logger.log(self.tag_input, inputs);
        logger.log(self.tag_state, state);
        let (sub1_out, sub1_state) = self.sub1.compute(&mut logger, state, inputs);
        let (sub2_out, sub2_state) = self.sub2.compute(&mut logger, state, inputs);
        // Do our own update
        let output = MoreJunk::default();
        let state = sub1_state + sub2_state;
        logger.log(self.tag_output, output);
        logger.log(self.tag_next_state, state);
        (output, state)
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum State {
    #[default]
    Boot,
    Running,
}

impl Loggable for State {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 0)
    }
    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
        match self {
            State::Boot => logger.write_string(tag, "Boot"),
            State::Running => logger.write_string(tag, "Running"),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Loggable, PartialEq)]
struct DeepJunk {
    x: u32,
    y: u16,
}

#[derive(Default, Clone, Copy, Debug, Loggable, PartialEq)]
struct Junk {
    a: bool,
    b: u8,
    c: State,
    d: DeepJunk,
}

#[derive(Default, Copy, Clone, Debug, Loggable, PartialEq)]
struct MoreJunk {
    a: Junk,
    b: Junk,
}

#[test]
fn test_trace_setup() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let foo = Foo::new(&mut logger_builder);
    println!("{}", logger_builder);
    println!("{:#?}", foo);
    let logger = logger_builder.build();
    println!("{}", logger);
    let mut vcd = vec![];
    logger.vcd(&mut vcd).unwrap();
    //    println!("{}", String::from_utf8(vcd).unwrap());
    std::fs::write("empty.vcd", vcd).unwrap();
}

#[test]
fn test_using_address() {
    struct Foo {
        id: usize,
    }

    struct Junk {
        id: usize,
        bar1: Foo,
        bar2: Foo,
    }

    let jnk = Junk {
        id: 0,
        bar1: Foo { id: 1 },
        bar2: Foo { id: 2 },
    };

    println!("{:?}", &jnk as *const Junk);
    println!("{:?}", &jnk.bar1 as *const Foo);
    println!("{:?}", &jnk.bar2 as *const Foo);
}

// Test a simple counter machine.
struct Counter<T: Loggable> {
    tag_enable: TagID<bool>,
    tag_output: TagID<T>,
}

impl<T: Loggable> Counter<T> {
    pub fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("enable");
        let tag_output = builder.tag("output");
        Self {
            tag_enable: tag_input,
            tag_output,
        }
    }
}

impl<
        T: Loggable
            + Default
            + Copy
            + num_traits::ops::wrapping::WrappingAdd
            + num_traits::One
            + LowerHex,
    > Synchronous for Counter<T>
{
    type State = T;
    type Input = bool;
    type Output = T;

    fn compute(
        &self,
        mut tracer: impl Logger,
        enable: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        tracer.log(self.tag_enable, enable);
        let new_state = if enable {
            T::wrapping_add(&state, &T::one())
        } else {
            state
        };
        tracer.log(self.tag_output, new_state);
        (new_state, new_state)
    }
}

fn single_clock_simulation<S: Synchronous>(
    mut logger: impl Logger,
    obj: S,
    period_in_fs: u64,
    num_cycles: u64,
    mut test_bench: impl FnMut(u64, S::Output) -> S::Input,
) -> bool {
    let mut state = S::State::default();
    let mut new_state = S::State::default();
    let mut output = S::Output::default();
    for cycle in 0..num_cycles {
        logger.set_time_in_fs(cycle * period_in_fs);
        let mut input = test_bench(cycle, output);
        loop {
            (output, new_state) = obj.compute(&mut logger, input, state);
            let next_input = test_bench(cycle, output);
            if next_input == input {
                break;
            }
            input = next_input;
        }
        state = new_state;
    }
    true
}

/*
For combinatorial test logic...

Suppose we have a fifo.
inputs:
<write>
<read>
outputs:
<full>
<empty>

Then there is a logic connection between outputs and inputs.
As such, we must first know the outputs before we can decide
what the input should be.

If the design contains combinatorial paths from input to
output, then we can end up in oscillations.

let mut prev_output;
let prev_input = test_bench(clock, prev_output);
let (output, new_state) = compute(logger, prev_input, state);
let next_input = test_bench(clock, output);
while next_input != prev_input {
    let (output, new_state) = compute(logger, next_input, state);
}

*/

#[test]
fn test_counter_with_closures() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let clock = ClockDetails {
        name: "clock".to_string(),
        period_in_fs: freq_hz_to_period_femto(1e6) as u64,
        offset_in_fs: 0,
        initial_state: true,
    };
    let period_in_fs = clock.period_in_fs;
    logger_builder.add_clock(clock);
    let counter: Counter<u32> = Counter::new(&mut logger_builder);
    let now = std::time::Instant::now();
    assert!(single_clock_simulation(
        logger_builder.build(),
        counter,
        period_in_fs,
        10_000_000,
        |cycle, output| cycle % 2 == 0,
    ));
    println!("Elapsed: {:?}", now.elapsed());
}

#[test]
fn test_counter_with_tracing() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let clock = ClockDetails {
        name: "clock".to_string(),
        period_in_fs: freq_hz_to_period_femto(1e6) as u64,
        offset_in_fs: 0,
        initial_state: true,
    };
    assert_eq!(clock.next_edge_after(0), 1_000_000_000 / 2);
    let period_in_fs = clock.period_in_fs;
    logger_builder.add_clock(clock);
    let counter: Counter<u32> = Counter::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    let mut state = 0;
    let mut last_output = 0;
    let now = std::time::Instant::now();
    for cycle in 0..100_000 {
        logger.set_time_in_fs(cycle * period_in_fs);
        let (output, new_state) = counter.compute(&mut logger, cycle % 2 == 0, state);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output}");
    println!("Simulation time: {}", now.elapsed().as_secs_f64());
    let now = std::time::Instant::now();
    // Time serde_json serialization of the logger.
    {
        let foo = bincode::serialize(&logger).unwrap();
        println!(
            "JSON generation time: {} {}",
            now.elapsed().as_secs_f64(),
            foo.len()
        );
        std::fs::write("counter.bcd", foo).unwrap();
    }
    let now = std::time::Instant::now();
    println!("{}", logger);
    let mut vcd = vec![];
    {
        logger.vcd(&mut vcd).unwrap();
        println!("VCD generation time: {}", now.elapsed().as_secs_f64());
    }
    std::fs::write("counter.vcd", vcd).unwrap();
}

#[test]
fn test_counter_with_no_tracing() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let clock = ClockDetails {
        name: "clock".to_string(),
        period_in_fs: freq_hz_to_period_femto(1e6) as u64,
        offset_in_fs: 0,
        initial_state: true,
    };
    logger_builder.add_clock(clock);
    let counter: Counter<u32> = Counter::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    let mut state = 0;
    let mut last_output = 0;
    for cycle in 0..100_000_000 {
        let (output, new_state) = counter.compute((), cycle % 2 == 0, state);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output}");
}

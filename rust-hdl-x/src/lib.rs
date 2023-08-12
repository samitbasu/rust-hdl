use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use crate::log::LogBuilder;
use crate::loggable::Loggable;
use log::TagID;
use logger::Logger;
use rust_hdl::prelude::Bits;
use rust_hdl_x_macro::Loggable;
use synchronous::Synchronous;

use crate::basic_logger::BasicLoggerBuilder;

//use synchronous::Synchronous;

//mod bit_iter;
//mod bit_slice;
//mod counter;
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
pub mod log;
pub mod loggable;
pub mod logger;

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
    fn new(builder: impl LogBuilder) -> Self {
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

#[derive(Default, Debug)]
struct Foo {
    sub1: Bar,
    sub2: Bar,
    tag_input: TagID<u16>,
    tag_output: TagID<MoreJunk>,
    tag_state: TagID<u16>,
    tag_next_state: TagID<u16>,
}

impl Foo {
    fn new(builder: impl LogBuilder) -> Self {
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
        let (sub1_out, sub1_state) = self.sub1.update(&mut logger, state, inputs);
        let (sub2_out, sub2_state) = self.sub2.update(&mut logger, state, inputs);
        // Do our own update
        let output = MoreJunk::default();
        let state = sub1_state + sub2_state;
        logger.log(self.tag_output, output);
        logger.log(self.tag_next_state, state);
        (output, state)
    }
}

#[derive(Default, Clone, Copy, Loggable)]
enum State {
    #[default]
    Boot,
    Running,
}

#[derive(Default, Clone, Copy, Loggable)]
struct Junk {
    a: bool,
    b: u8,
    c: State,
}

#[derive(Default, Copy, Clone, Loggable)]
struct MoreJunk {
    a: Junk,
    b: Junk,
}

#[test]
fn test_trace_setup() {
    let mut tracer_builder = basic_logger::BasicLoggerBuilder::default();
    let mut foo = Foo::new(&mut tracer_builder);
    println!("{}", tracer_builder);
    println!("{:#?}", foo);
    let mut tracer = tracer_builder.build();
    println!("{}", tracer);
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
    tag_input: TagID<bool>,
    tag_output: TagID<T>,
}

impl<T: Loggable> Counter<T> {
    fn new(builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        Self {
            tag_input,
            tag_output,
        }
    }
}

impl<T: Loggable + Default + Copy + num_traits::ops::wrapping::WrappingAdd + num_traits::One>
    Synchronous for Counter<T>
{
    type State = T;
    type Input = bool;
    type Output = T;

    fn compute(
        &self,
        mut tracer: impl Logger,
        input: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        tracer.log(self.tag_input, input);
        let new_state = if input {
            T::wrapping_add(&state, &T::one())
        } else {
            state
        };
        tracer.log(self.tag_output, new_state);
        (new_state, new_state)
    }
}

#[test]
fn test_counter_with_tracing() {
    let mut logger_builder = BasicLoggerBuilder::default();
    let counter = Counter::new::<u32>(&mut logger_builder);
    let mut logger = logger_builder.build();
    let mut state = 0;
    let mut last_output = 0;
    let mut new_state = 0;
    for cycle in 0..10_000_000 {
        let (output, new_state) = counter.compute(&mut logger, cycle % 2 == 0, state);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output}");
    println!("{}", logger);
}

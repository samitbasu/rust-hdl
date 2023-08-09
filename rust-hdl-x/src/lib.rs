use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use traceable::Traceable;
use tracer::{TraceID, TraceTag, TraceType, Tracer};
use tracer_builder::TracerBuilder;

use crate::basic_tracer::BasicTracerBuilder;

//use synchronous::Synchronous;

//mod bit_iter;
//mod bit_slice;
//mod counter;
//mod derive_vcd;
//mod pulser;
//mod shot;
//mod spi_controller;
//mod strobe;
//mod synchronous;
pub mod basic_tracer;
pub mod no_trace;
pub mod traceable;
pub mod tracer;
pub mod tracer_builder;
//mod vcd;

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

pub trait Synchronous {
    type Input: Copy + Traceable;
    type Output: Copy + Traceable + Default;
    type State: Copy + Default + Traceable;
    // Must be derived
    fn setup(&mut self, trace: impl TracerBuilder);
    // User provided or derived
    fn trace_id(&self) -> Option<TraceID>;
    // User provided
    fn compute(
        &self,
        tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State);
    // Always fixed
    fn update(
        &self,
        mut tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        if let Some(id) = self.trace_id() {
            tracer.set_context(id);
            tracer.set_tag(TraceTag::Input);
            inputs.record(&mut tracer);
            tracer.set_tag(TraceTag::StateQ);
            state.record(&mut tracer);
        }
        let (output, state) = self.compute(&mut tracer, state, inputs);
        if let Some(id) = self.trace_id() {
            tracer.set_context(id);
            tracer.set_tag(TraceTag::Output);
            output.record(&mut tracer);
            tracer.set_tag(TraceTag::StateD);
            state.record(&mut tracer);
        }
        (output, state)
    }
    // Always fixed
    fn register_trace_types(mut builder: impl TracerBuilder) -> TraceID {
        builder.set_kind(TraceType::Input);
        Self::Input::register_trace_type(&mut builder);
        builder.set_kind(TraceType::Output);
        Self::Output::register_trace_type(&mut builder);
        builder.set_kind(TraceType::State);
        Self::State::register_trace_type(&mut builder);
        builder.trace_id()
    }
}

#[derive(Default, Debug)]
struct Bar {
    counter: u16,
    trace_id: Option<TraceID>,
}

impl Synchronous for Bar {
    type Input = u16;
    type Output = bool;
    type State = u16;

    fn setup(&mut self, tracer: impl TracerBuilder) {
        self.trace_id = Some(Self::register_trace_types(tracer));
    }
    fn trace_id(&self) -> Option<TraceID> {
        self.trace_id
    }
    fn compute(
        &self,
        trace: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        todo!()
    }
}

#[derive(Default, Debug)]
struct Foo {
    sub1: Bar,
    sub2: Bar,
    trace_id: Option<TraceID>,
}

impl Synchronous for Foo {
    type Input = u16;
    type Output = MoreJunk;
    type State = u16;

    fn setup(&mut self, mut builder: impl TracerBuilder) {
        self.trace_id = Some(Self::register_trace_types(&mut builder));
        // Set up the submodules
        self.sub1.setup(builder.scope("sub1"));
        self.sub2.setup(builder.scope("sub2"));
    }
    fn trace_id(&self) -> Option<TraceID> {
        self.trace_id
    }
    fn compute(
        &self,
        mut tracer: impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        // Update the submodules
        let (sub1_out, sub1_state) = self.sub1.update(&mut tracer, state, inputs);
        let (sub2_out, sub2_state) = self.sub2.update(&mut tracer, state, inputs);
        // Do our own update
        let output = MoreJunk::default();
        let state = sub1_state + sub2_state;
        (output, state)
    }
}

#[derive(Default, Clone, Copy)]
enum State {
    #[default]
    Boot,
    Running,
}

impl Traceable for State {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(0);
    }
    fn record(&self, mut tracer: impl Tracer) {
        match self {
            State::Boot => tracer.write_string("Boot"),
            State::Running => tracer.write_string("Running"),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Junk {
    a: bool,
    b: u8,
    c: State,
}

impl Traceable for Junk {
    fn register_trace_type(tracer: impl TracerBuilder) {
        bool::register_trace_type(tracer.namespace("a"));
        u8::register_trace_type(tracer.namespace("b"));
        State::register_trace_type(tracer.namespace("c"));
    }
    fn record(&self, mut tracer: impl Tracer) {
        self.a.record(&mut tracer);
        self.b.record(&mut tracer);
        self.c.record(&mut tracer);
    }
}

#[derive(Default, Copy, Clone)]
struct MoreJunk {
    a: Junk,
    b: Junk,
}

impl Traceable for MoreJunk {
    fn register_trace_type(tracer: impl TracerBuilder) {
        Junk::register_trace_type(tracer.namespace("a"));
        Junk::register_trace_type(tracer.namespace("b"));
    }
    fn record(&self, mut tracer: impl Tracer) {
        self.a.record(&mut tracer);
        self.b.record(&mut tracer);
    }
}

#[test]
fn test_trace_setup() {
    let mut tracer_builder = BasicTracerBuilder::default();
    let mut foo = Foo::default();
    foo.setup(&mut tracer_builder);
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
struct Counter<T> {
    trace_id: Option<TraceID>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Traceable + Default + Copy + num_traits::ops::wrapping::WrappingAdd + num_traits::One>
    Synchronous for Counter<T>
{
    type State = T;
    type Input = bool;
    type Output = T;

    fn setup(&mut self, builder: impl TracerBuilder) {
        self.trace_id = Some(Self::register_trace_types(builder));
    }
    fn trace_id(&self) -> Option<TraceID> {
        self.trace_id
    }
    fn compute(
        &self,
        _tracer: impl Tracer,
        state: Self::State,
        input: Self::Input,
    ) -> (Self::Output, Self::State) {
        let new_state = if input {
            T::wrapping_add(&state, &T::one())
        } else {
            state
        };
        (new_state, new_state)
    }
}

#[test]
fn test_counter_with_tracing() {
    let mut counter = Counter {
        trace_id: None,
        _phantom: std::marker::PhantomData::<u32>,
    };
    let mut tracer_builder = BasicTracerBuilder::default();
    counter.setup(&mut tracer_builder);
    let mut tracer = tracer_builder.build();
    let mut state = 0;
    let mut last_output = 0;
    let mut new_state = 0;
    for cycle in 0..10_000_000 {
        let (output, new_state) = counter.update(&mut tracer, state, cycle % 2 == 0);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output}");
    println!("{}", tracer);
}

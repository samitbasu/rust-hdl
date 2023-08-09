use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

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
//mod tracer;
//mod vcd;

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

/*

More ideas

// Use stack to preserve the path.

fn update() -> () {
    let id = t.get_id();
    t.set_id(Hash(id,Self::ID));
    // Do stuff
    t.set_id(id)
    return (output, state)
}

This prevents a memory allocation or any other heap related operations in each cycle.


What about using a generic argument for the trace itself?
struct Foo<T: TracerBuilder> {
    trace: T::Tracer,
}

This does not guarantee that subfields will also be generic over the same argument.
It also does not guarantee that the tracer will be the same type as the tracer for the subfields.


How about:
1. Use a traceID in the struct
2. Have the TracerBuilder hand out traceIDs
3. Write a setup pass that registers the input output and state types with the tracerbuilder
4. Have the tracer be passed in, and use the traceID to set the context.
 */

// Latest idea - make a trace handle part of the struct.

// TODO - add some kind of token to make sure we do not call compute directly, but only via update.

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TraceID(usize);

impl Default for TraceID {
    fn default() -> Self {
        Self(!0)
    }
}

pub trait Tracer {
    fn set_context(&mut self, id: TraceID);
    fn set_tag(&mut self, tag: TraceTag);
    fn write_bool(&mut self, val: bool);
    fn write_small(&mut self, val: u64);
    fn write_large(&mut self, val: &[bool]);
    fn write_string(&mut self, val: &'static str);
}

impl Tracer for () {
    fn set_context(&mut self, _id: TraceID) {}
    fn set_tag(&mut self, _tag: TraceTag) {}
    fn write_bool(&mut self, _val: bool) {}
    fn write_small(&mut self, _val: u64) {}
    fn write_large(&mut self, _val: &[bool]) {}
    fn write_string(&mut self, _val: &'static str) {}
}

impl<T: Tracer> Tracer for &mut T {
    fn set_context(&mut self, id: TraceID) {
        (**self).set_context(id)
    }
    fn set_tag(&mut self, tag: TraceTag) {
        (**self).set_tag(tag)
    }
    fn write_bool(&mut self, val: bool) {
        (**self).write_bool(val)
    }
    fn write_small(&mut self, val: u64) {
        (**self).write_small(val)
    }
    fn write_large(&mut self, val: &[bool]) {
        (**self).write_large(val)
    }
    fn write_string(&mut self, val: &'static str) {
        (**self).write_string(val)
    }
}

pub trait Synchronous {
    type Input: Copy + Traceable;
    type Output: Copy + Traceable + Default;
    type State: Copy + Default + Traceable;
    // Must be derived
    fn setup(&mut self, trace: impl TracerBuilder);
    // User provided or derived
    fn trace_id(&self) -> TraceID;
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
        tracer.set_context(self.trace_id());
        tracer.set_tag(TraceTag::Input);
        inputs.record(&mut tracer);
        tracer.set_tag(TraceTag::StateQ);
        state.record(&mut tracer);
        let (output, state) = self.compute(&mut tracer, state, inputs);
        tracer.set_tag(TraceTag::Output);
        output.record(&mut tracer);
        tracer.set_tag(TraceTag::StateD);
        state.record(&mut tracer);
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
    trace_id: TraceID,
}

impl Synchronous for Bar {
    type Input = u16;
    type Output = bool;
    type State = u16;

    fn setup(&mut self, tracer: impl TracerBuilder) {
        self.trace_id = Self::register_trace_types(tracer);
    }
    fn trace_id(&self) -> TraceID {
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
    trace_id: TraceID,
}

impl Synchronous for Foo {
    type Input = u16;
    type Output = MoreJunk;
    type State = u16;

    fn setup(&mut self, mut builder: impl TracerBuilder) {
        self.trace_id = Self::register_trace_types(&mut builder);
        // Set up the submodules
        self.sub1.setup(builder.scope("sub1"));
        self.sub2.setup(builder.scope("sub2"));
    }
    fn trace_id(&self) -> TraceID {
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

pub trait TracerBuilder {
    type SubBuilder: TracerBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder;
    fn trace_id(&self) -> TraceID;
    fn set_kind(&mut self, kind: TraceType);
    fn register(&self, width: usize);
    fn namespace(&self, name: &str) -> Self::SubBuilder;
}

impl<T: TracerBuilder> TracerBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder {
        (**self).scope(name)
    }
    fn trace_id(&self) -> TraceID {
        (**self).trace_id()
    }
    fn register(&self, width: usize) {
        (**self).register(width)
    }
    fn namespace(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace(name)
    }
    fn set_kind(&mut self, kind: TraceType) {
        (**self).set_kind(kind)
    }
}

pub trait TraceTarget {}

pub trait Traceable {
    fn register_trace_type(tracer: impl TracerBuilder);
    fn record(&self, tracer: impl Tracer);
}

impl Traceable for bool {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(1);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_bool(*self);
    }
}

impl Traceable for u8 {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(8);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_small(*self as u64);
    }
}

impl Traceable for u16 {
    fn register_trace_type(tracer: impl TracerBuilder) {
        tracer.register(16);
    }
    fn record(&self, mut tracer: impl Tracer) {
        tracer.write_small(*self as u64);
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

#[derive(Debug, Clone)]
enum TraceValues {
    Short(Vec<u64>),
    Long(Vec<Vec<bool>>),
    Enum(Vec<&'static str>),
}

#[derive(Debug, Clone)]
struct TraceSignal {
    name: String,
    width: usize,
    values: TraceValues,
}

impl TraceSignal {
    fn new(name: &str, width: usize) -> TraceSignal {
        TraceSignal {
            name: name.to_string(),
            width,
            values: if width == 0 {
                TraceValues::Enum(vec![])
            } else if width <= 64 {
                TraceValues::Short(vec![])
            } else {
                TraceValues::Long(vec![])
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum TraceTag {
    #[default]
    Input,
    Output,
    StateD,
    StateQ,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TraceType {
    Input,
    Output,
    State,
}

#[derive(Debug, Clone)]
struct ScopeRecord {
    name: String,
    inputs: Vec<TraceSignal>,
    outputs: Vec<TraceSignal>,
    state_q: Vec<TraceSignal>,
    state_d: Vec<TraceSignal>,
}

impl Display for ScopeRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for input in &self.inputs {
            writeln!(f, "{}::input::{} [{}]", self.name, input.name, input.width)?;
        }
        for output in &self.outputs {
            writeln!(
                f,
                "{}::output::{} [{}]",
                self.name, output.name, output.width
            )?;
        }
        for state_q in &self.state_q {
            writeln!(
                f,
                "{}::state_q::{} [{}]",
                self.name, state_q.name, state_q.width
            )?;
        }
        for state_d in &self.state_d {
            writeln!(
                f,
                "{}::state_d::{} [{}]",
                self.name, state_d.name, state_d.width
            )?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct BasicTracer {
    scopes: Vec<ScopeRecord>,
    scope_index: usize,
    field_index: usize,
    tag: TraceTag,
}

impl BasicTracer {
    fn signal(&mut self) -> &mut TraceSignal {
        let scope = &mut self.scopes[self.scope_index];
        match self.tag {
            TraceTag::Input => &mut scope.inputs[self.field_index],
            TraceTag::Output => &mut scope.outputs[self.field_index],
            TraceTag::StateD => &mut scope.state_d[self.field_index],
            TraceTag::StateQ => &mut scope.state_q[self.field_index],
        }
    }
}

impl Tracer for BasicTracer {
    fn write_bool(&mut self, value: bool) {
        if let TraceValues::Short(ref mut values) = self.signal().values {
            values.push(value as u64);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_small(&mut self, value: u64) {
        if let TraceValues::Short(ref mut values) = self.signal().values {
            values.push(value);
        } else {
            panic!("Wrong type");
        }
    }
    fn write_large(&mut self, val: &[bool]) {
        if let TraceValues::Long(ref mut values) = self.signal().values {
            values.push(val.to_vec());
        } else {
            panic!("Wrong type");
        }
    }
    fn write_string(&mut self, val: &'static str) {
        if let TraceValues::Enum(ref mut values) = self.signal().values {
            values.push(val);
        } else {
            panic!("Wrong type");
        }
    }

    fn set_context(&mut self, id: TraceID) {
        self.scope_index = id.0;
    }

    fn set_tag(&mut self, tag: TraceTag) {
        self.field_index = 0;
        self.tag = tag;
    }
}

impl Display for BasicTracer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.scopes[self.scope_index].fmt(f)
    }
}

// I don't like the use of interior mutability here.
// I need to redesign the API so it is not required.
#[derive(Clone, Debug)]
struct BasicTracerBuilder {
    scopes: Rc<RefCell<Vec<ScopeRecord>>>,
    current_scope: usize,
    current_kind: Option<TraceType>,
    path: Vec<String>,
}

impl Display for BasicTracerBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.scopes.borrow().iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for BasicTracerBuilder {
    fn default() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(vec![ScopeRecord {
                name: "root".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                state_q: Vec::new(),
                state_d: Vec::new(),
            }])),
            current_scope: 0,
            current_kind: None,
            path: vec![],
        }
    }
}

impl TracerBuilder for BasicTracerBuilder {
    type SubBuilder = Self;
    fn scope(&self, name: &str) -> Self {
        let name = format!(
            "{}::{}",
            self.scopes.borrow()[self.current_scope].name,
            name
        );
        self.scopes.borrow_mut().push(ScopeRecord {
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            state_q: Vec::new(),
            state_d: Vec::new(),
        });
        Self {
            scopes: self.scopes.clone(),
            current_scope: self.scopes.borrow().len() - 1,
            current_kind: None,
            path: vec![],
        }
    }

    fn trace_id(&self) -> TraceID {
        TraceID(self.current_scope)
    }

    fn set_kind(&mut self, kind: TraceType) {
        self.current_kind = Some(kind);
    }

    fn register(&self, width: usize) {
        let name = self.path.join("::");
        let signal = TraceSignal::new(&name, width);
        let kind = self.current_kind.unwrap();
        match kind {
            TraceType::Input => {
                self.scopes.borrow_mut()[self.current_scope]
                    .inputs
                    .push(signal);
            }
            TraceType::Output => {
                self.scopes.borrow_mut()[self.current_scope]
                    .outputs
                    .push(signal);
            }
            TraceType::State => {
                self.scopes.borrow_mut()[self.current_scope]
                    .state_q
                    .push(signal.clone());
                self.scopes.borrow_mut()[self.current_scope]
                    .state_d
                    .push(signal);
            }
        }
    }

    fn namespace(&self, name: &str) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        Self {
            scopes: self.scopes.clone(),
            current_scope: self.current_scope,
            current_kind: self.current_kind,
            path: new_path,
        }
    }
}

impl BasicTracerBuilder {
    pub fn build(self) -> BasicTracer {
        BasicTracer {
            scopes: self.scopes.take(),
            scope_index: 0,
            field_index: 0,
            tag: TraceTag::Input,
        }
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

/*
Consider the possibility of separating the trace function from the update function so that
they are completely different.  This means that you cannot, for example, easily write multiple
intermediate results to the trace.

There are a few options:
1.  The trace contains only the state vector.  That is the easiest, but not particularly helpful.
2.  A new struct is created that contains the states and the inputs and outputs.

*/

// We then provide a derive macro to add the TraceHandle storage into the parent struct.

/*

#[add_trace_support]
struct MyThing {
   a: Sub1,
   b: Sub2,
   c: u16
   handle: TraceHandle, // <- inserted by the attribute macro
}

impl FooTrace for MyThing {
    fn set_handle(&mut self, handle: TraceHandle) {
        self.handle = handle;
    }
    fn get_handle(&self) -> TraceHandle {
        self.handle
    }
}

Adds a generated field to hold the trace handle and a couple of accessor methods:



struct MyThing {

}


*/

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

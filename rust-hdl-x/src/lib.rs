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


 */

// Latest idea - make a trace handle part of the struct.

pub trait Tracer {}

pub trait Synchronous {
    type Input: Copy + Traceable2;
    type Output: Copy + Traceable2 + Default;
    type State: Copy + Default + Traceable2;
    fn setup(&mut self, trace: &mut impl TracerBuilder);
    fn update(
        &self,
        tracer: &mut impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State);
    fn register_trace_types(tracer: &mut impl TracerBuilder) -> TraceHandle {
        Self::Input::register_trace_type(tracer, TraceType::Input);
        Self::Output::register_trace_type(tracer, TraceType::Output);
        Self::State::register_trace_type(tracer, TraceType::StateD);
        Self::State::register_trace_type(tracer, TraceType::StateQ);
        tracer.handle()
    }
}

#[derive(Default, Debug)]
struct Bar {
    counter: u16,
    trace: TraceHandle,
}

impl Synchronous for Bar {
    type Input = u16;
    type Output = bool;
    type State = u16;

    fn setup(&mut self, tracer: &mut impl TracerBuilder) {
        self.trace = Self::register_trace_types(tracer);
    }

    fn update(
        &self,
        tracer: &mut impl Tracer,
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
    trace: TraceHandle,
}

impl Synchronous for Foo {
    type Input = u16;
    type Output = MoreJunk;
    type State = u16;

    fn setup(&mut self, tracer: &mut impl TracerBuilder) {
        self.trace = Self::register_trace_types(tracer);
        // Set up the submodules
        self.sub1.setup(&mut tracer.scope("sub1"));
        self.sub2.setup(&mut tracer.scope("sub2"));
    }

    fn update(
        &self,
        tracer: &mut impl Tracer,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        // Update the submodules
        let (sub1_out, sub1_state) = self.sub1.update(tracer, state, inputs);
        let (sub2_out, sub2_state) = self.sub2.update(tracer, state, inputs);
        // Do our own update
        let output = MoreJunk::default();
        let state = sub1_state + sub2_state;
        (output, state)
    }
}

// In the constructor for Foo, we set up our tracing

// The tracehandle points to the entry in the tracer
// table that contains information for the current
// instance of the struct.

#[derive(Clone, Copy, Debug)]
pub struct TraceHandle(usize);

impl Default for TraceHandle {
    fn default() -> Self {
        Self(!0)
    }
}

pub trait TracerBuilder {
    fn scope(&self, name: &str) -> Self;
    fn handle(&self) -> TraceHandle;
    fn register(&mut self, width: usize, kind: TraceType);
    fn namespace(&self, name: &str) -> Self;
}

pub trait Traceable2 {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType);
}

impl Traceable2 for bool {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        tracer.register(1, kind);
    }
}

impl Traceable2 for u8 {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        tracer.register(8, kind);
    }
}

impl Traceable2 for u16 {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        tracer.register(16, kind);
    }
}

#[derive(Default, Clone, Copy)]
enum State {
    #[default]
    Boot,
    Running,
}

impl Traceable2 for State {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        tracer.register(0, kind);
    }
}

#[derive(Default, Clone, Copy)]
struct Junk {
    a: bool,
    b: u8,
    c: State,
}

impl Traceable2 for Junk {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        bool::register_trace_type(&mut tracer.namespace("a"), kind);
        u8::register_trace_type(&mut tracer.namespace("b"), kind);
        State::register_trace_type(&mut tracer.namespace("c"), kind);
    }
}

#[derive(Default, Copy, Clone)]
struct MoreJunk {
    a: Junk,
    b: Junk,
}

impl Traceable2 for MoreJunk {
    fn register_trace_type(tracer: &mut impl TracerBuilder, kind: TraceType) {
        Junk::register_trace_type(&mut tracer.namespace("a"), kind);
        Junk::register_trace_type(&mut tracer.namespace("b"), kind);
    }
}

#[derive(Debug, Clone)]
enum TraceValues {
    Short(Vec<u64>),
    Long(Vec<Vec<bool>>),
    Enum(Vec<String>),
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TraceType {
    Input,
    Output,
    StateQ,
    StateD,
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

#[derive(Clone, Debug)]
struct BasicTracer {
    scopes: Rc<RefCell<Vec<ScopeRecord>>>,
    current: TraceHandle,
    path: Vec<String>,
}

impl Display for BasicTracer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for scope in self.scopes.borrow().iter() {
            writeln!(f, "{}", scope)?;
        }
        Ok(())
    }
}

impl Default for BasicTracer {
    fn default() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(vec![ScopeRecord {
                name: "root".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                state_q: Vec::new(),
                state_d: Vec::new(),
            }])),
            current: TraceHandle(0),
            path: vec![],
        }
    }
}

impl TracerBuilder for BasicTracer {
    fn scope(&self, name: &str) -> Self {
        let name = format!("{}::{}", self.scopes.borrow()[self.current.0].name, name);
        self.scopes.borrow_mut().push(ScopeRecord {
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            state_q: Vec::new(),
            state_d: Vec::new(),
        });
        Self {
            scopes: self.scopes.clone(),
            current: TraceHandle(self.scopes.borrow().len() - 1),
            path: vec![],
        }
    }

    fn handle(&self) -> TraceHandle {
        self.current
    }

    fn register(&mut self, width: usize, kind: TraceType) {
        let name = self.path.join("::");
        let signal = TraceSignal::new(&name, width);
        match kind {
            TraceType::Input => {
                self.scopes.borrow_mut()[self.current.0].inputs.push(signal);
            }
            TraceType::Output => {
                self.scopes.borrow_mut()[self.current.0]
                    .outputs
                    .push(signal);
            }
            TraceType::StateQ => {
                self.scopes.borrow_mut()[self.current.0]
                    .state_q
                    .push(signal);
            }
            TraceType::StateD => {
                self.scopes.borrow_mut()[self.current.0]
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
            current: self.current,
            path: new_path,
        }
    }
}

#[test]
fn test_trace_setup() {
    let mut tracer = BasicTracer::default();
    let mut foo = Foo::default();
    foo.setup(&mut tracer);
    println!("{}", tracer);
    println!("{:#?}", foo);
}

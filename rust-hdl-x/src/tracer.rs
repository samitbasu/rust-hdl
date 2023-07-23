// For tracing, we would like to add a tracer to the update function.
// That means the trait needs to include a tracer

// trait Synchronous {
//     type Input;
//     type Output;
//     type State;
//     type Tracer;
//     fn update(&self, tracer: &mut Trace, q: Self::State, trigger: Self::Input) -> (Self::Output, Self::State);
//     fn default_output(&self) -> Self::Output;
// }

// Then, in the update function, we can trace the various signals.
//
// For example, in the shot test, we would like to trace the trigger, the state,
// and the counter.

// Some additional thoughts on the issue of tracing.  We really only need to store variable length
// bit strings with each path (or static strings).  So our log messages look like this:

use std::cell::RefCell;

use ruint::Uint;
use rust_hdl_x_macro::BitSerialize;

#[derive(Debug, Clone)]
enum TraceMessage {
    EnterModule(&'static str),
    EnterStruct(&'static str),
    Bool(&'static str, bool),
    Short(&'static str, usize, u64),
    Vector(&'static str, usize, Vec<u64>),
    String(&'static str, &'static str),
    ExitStruct(),
    ExitModule(),
}

// The tracing interface is then

trait Tracer {
    fn enter_module(&self, name: &'static str);
    fn log(&self, name: &'static str, value: impl BitSerialize);
    fn exit_module(&self);
}

trait BitSerialize {
    fn serialize(&self, tag: &'static str, serializer: impl BitSerializer);
}

trait BitSerializer {
    fn enter_struct(&self, name: &'static str);
    fn bool(&self, tag: &'static str, value: bool);
    fn short(&self, tag: &'static str, bits: usize, value: u64);
    fn long(&self, tag: &'static str, bits: usize, values: &[u64]);
    fn string(&self, tag: &'static str, value: &'static str);
    fn exit_struct(&self);
}

impl<const N: usize, const LIMBS: usize> BitSerialize for Uint<N, LIMBS> {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        if N == 1 {
            serializer.bool(tag, self.to());
        } else if N <= 64 {
            serializer.short(tag, N, self.to());
        } else {
            serializer.long(tag, N, self.as_limbs().as_slice());
        }
    }
}

impl BitSerialize for bool {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.bool(tag, *self);
    }
}

impl BitSerialize for u8 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 8, *self as u64);
    }
}

impl BitSerialize for u16 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 16, *self as u64);
    }
}

impl BitSerialize for u32 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 32, *self as u64);
    }
}

impl BitSerialize for u64 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 64, *self);
    }
}

#[derive(BitSerialize)]
struct TwoBits {
    bit_1: bool,
    bit_2: bool,
}

#[derive(BitSerialize)]
struct NoWorky {
    bit_1: bool,
    bit_2: bool,
    part_3: u8,
    nibble_4: u16,
}

// A simple in memory tracer is then
struct InMemoryTracer {
    messages: RefCell<Vec<TraceMessage>>,
}

impl InMemoryTracer {
    fn new() -> Self {
        Self {
            messages: RefCell::new(Vec::new()),
        }
    }
}

impl Tracer for InMemoryTracer {
    fn enter_module(&self, name: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::EnterModule(name));
    }
    fn log(&self, name: &'static str, value: impl BitSerialize) {
        value.serialize(name, self);
    }
    fn exit_module(&self) {
        self.messages.borrow_mut().push(TraceMessage::ExitModule());
    }
}

impl BitSerializer for InMemoryTracer {
    fn enter_struct(&self, name: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::EnterStruct(name));
    }
    fn bool(&self, tag: &'static str, value: bool) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Bool(tag, value));
    }
    fn short(&self, tag: &'static str, bits: usize, value: u64) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Short(tag, bits, value));
    }
    fn long(&self, tag: &'static str, bits: usize, values: &[u64]) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Vector(tag, bits, values.to_vec()));
    }
    fn string(&self, tag: &'static str, value: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::String(tag, value));
    }
    fn exit_struct(&self) {
        self.messages.borrow_mut().push(TraceMessage::ExitStruct());
    }
}

impl<T: BitSerializer> BitSerializer for &T {
    fn enter_struct(&self, name: &'static str) {
        (*self).enter_struct(name);
    }
    fn bool(&self, tag: &'static str, value: bool) {
        (*self).bool(tag, value);
    }
    fn short(&self, tag: &'static str, bits: usize, value: u64) {
        (*self).short(tag, bits, value);
    }
    fn long(&self, tag: &'static str, bits: usize, values: &[u64]) {
        (*self).long(tag, bits, values);
    }
    fn string(&self, tag: &'static str, value: &'static str) {
        (*self).string(tag, value);
    }
    fn exit_struct(&self) {
        (*self).exit_struct();
    }
}

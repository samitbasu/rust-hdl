use crate::atom::AtomKind;

#[doc(hidden)]
pub trait Direction: Clone {
    const KIND: AtomKind;
}

/// This direction marker is used for a [Signal] that is
/// an input with respect to a circuit.  That means
/// that we do not expect to write to the input, but that
/// the value will be set by external components to the
/// circuit.
/// ```rust
/// use rust_hdl_core::prelude::*;
///
/// struct Foo {
///     pub x: Signal<In, Bit>,     // <--- This is a single bit input
///     pub y: Signal<In, Bits<8>>, // <--- This is a multi-bit input signal
/// }
/// ```
#[derive(Default, Clone, Debug)]
pub struct In {}

/// This direction marker is used for a [Signal] that
/// leaves a circuit as an output.  That means we expect this
/// circuit to drive the signal using its internal logic.
/// It is an error in RustHDL to leave an output undriven.
#[derive(Default, Clone, Debug)]
pub struct Out {}

#[derive(Default, Clone, Debug)]
pub struct Local {}

#[derive(Default, Clone, Debug)]
pub struct InOut {}

impl Direction for In {
    const KIND: AtomKind = AtomKind::InputParameter;
}

impl Direction for Out {
    const KIND: AtomKind = AtomKind::OutputParameter;
}

impl Direction for Local {
    const KIND: AtomKind = AtomKind::LocalSignal;
}

impl Direction for InOut {
    const KIND: AtomKind = AtomKind::InOutParameter;
}

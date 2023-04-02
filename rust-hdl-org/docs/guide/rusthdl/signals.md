//! ### Signal Type
//!
//! *Signals are software abstractions to represent physical wires*.  The [Signal](core::signal::Signal)
//!type is generic over a couple of parameters.  The first is meant to indicate the driver of the wire.
//! In RustHDL, every wire must have exactly one driver.  It is the hardware equivalent of the
//! single writer principle.  You can have as many readers as you want, but only one writer.  Unfortunately,
//! there are some subtleties here, and declaring ownership of a wire using the type system is
//! imperfect.  Instead, we settle for a signalling mechanism of _intent_.  So you mark a
//! signal as how you intend to use it in your logic.  For example, consider the following circuit:
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! pub struct My8BitAdder {
//!    pub input_1: Signal<In, Bits<8>>,
//!    pub input_2: Signal<In, Bits<8>>,
//!    pub output: Signal<Out, Bits<8>>,
//! }
//! ```
//! In this case, the fields of the adder circuit are marked as `pub` so they can be accessed from
//! outside the circuit.  The [Direction](core::signal::Direction) argument to the [Signal] indicates
//! how the given circuit intends to utilize the various wires.  In this case, `input_1` and `input_2`
//! should be considered inputs, and `output` is, obviously, an output.  As such, `My8BitAdder` is
//! promising you that it will drive the `output` wire.  If it fails to actually do so (by leaving
//! it undriven), then you will get an error when you try to use it in a design.
//!
//! *RustHDL does not allow undriven nets*.  They are treated similarly to uninitialized memory in Rust.
//! You _must_ drive every net in the design.  Furthermore, you can have only one driver for each
//! net.  These two principles are core to RustHDL!
//!
//! The second part of a [Signal] is that it is _typed_.  In general, the type signature is meant
//! to convey something about the nature of the data being stored or passed.  In the case of
//! `My8BitAdder`, it doesn't say much - only that the input is an unsigned 8-bit value.  But
//! the types can be more complicated, including collections of signals running in multiple
//! directions (as is typical for a bus or other complex interface).
//!
//! Signals can also be bidirectional with the [InOut](core::signal::Direction::InOut) designation.
//! But you typically can only use these types of signals at the edge of your device.  More on that
//! elsewhere.
//!
//! The definition of [Signal] also indicates how it should be used.  [Signal]'s cannot be
//! assigned to using usual semantics.
//! ```
//! # use rust_hdl::prelude::*;
//!
//! #[derive(Clone, Debug)]
//! pub struct Signal<D: Direction, T: Synth> {
//!     pub next: T,
//!     pub changed: bool,
//!     // Internal details omitted
//!#     dir: std::marker::PhantomData<D>,
//! }
//!```
//!
//! To change (drive) the value of a signal, you assign to the `next` field.  To read the
//! value of the signal (i.e. to get it's current state without driving it), you use the `val()` method.
//! This is in keeping with the idea that you treat the signal differently if you want to drive
//! it to some value, versus if you want to read it, as in hardware, these are different functions.
//! In most cases, you will read from `val()` of the input signals, and write to the `.next` of the
//! output signal.  For example, in the `My8BitAdder` example, you would read from `input_1.val()`
//! and from `input_2.val()`, and write to `output.next`.  Like this:
//!
//! ```
//! # use rust_hdl::prelude::*;
//!
//! pub struct My8BitAdder {
//!    pub input_1: Signal<In, Bits<8>>,
//!    pub input_2: Signal<In, Bits<8>>,
//!    pub output: Signal<Out, Bits<8>>,
//! }
//!
//! impl Logic for My8BitAdder {
//!    fn update(&mut self) {
//!        self.output.next = self.input_1.val() + self.input_2.val();
//!    }
//! }
//!
//! ```
//!
//! In general, this is the pattern to follow.  However, there are some exceptions.  Sometimes,
//! you will want a "scratchpad" for holding intermediate results in a longer expression.
//! For example, suppose you want to logically OR a bunch of values together, but want to
//! logically shift them into different positions before doing so.  Let us assume you have
//! a logical block that looks like this:
//!
//! ```
//! # use rust_hdl::prelude::*;
//!
//! pub struct OrStuff {
//!    pub val1: Signal<In, Bit>,
//!    pub val2: Signal<In, Bits<4>>,
//!    pub val3: Signal<In, Bits<2>>,
//!    pub val4: Signal<In, Bit>,
//!    pub combined: Signal<Out, Bits<8>>,
//!    pad: Signal<Local, Bits<8>>,
//! }
//! ```
//!
//! In this case, the `pad` field (which is private to the logic) has a direction of `Local`,
//! which means it can be used to write and read from in the same circuit, as _long as you write first_!
//! Hence, you can do something like this in the `update` method
//!
//! ```
//! # use rust_hdl::prelude::*;
//!
//!# pub struct OrStuff {
//!#    pub val1: Signal<In, Bit>,
//!#    pub val2: Signal<In, Bits<4>>,
//!#    pub val3: Signal<In, Bits<2>>,
//!#    pub val4: Signal<In, Bit>,
//!#    pub combined: Signal<Out, Bits<8>>,
//!#    pad: Signal<Local, Bits<8>>,
//!# }
//!
//! impl Logic for OrStuff {
//!    fn update(&mut self) {
//!       self.pad.next = 0.into(); // Write first
//!       self.pad.next = self.pad.val() | bit_cast::<8,1>(self.val1.val().into()); // Now we can read and write to it
//!       self.pad.next = self.pad.val() | (bit_cast::<8,4>(self.val2.val()) << 1);
//!       self.pad.next = self.pad.val() | (bit_cast::<8,2>(self.val3.val()) << 5);
//!       self.pad.next = self.pad.val() | (bit_cast::<8,1>(self.val4.val().into()) << 7);
//!       self.combined.next = self.pad.val();
//!    }
//! }
//! ```
//!
//! You can understand this behavior by either "folding" all of the expressions into a single
//! long expression (i.e., by eliminating `self.pad` altogether) and just assigning the output
//! to an expression consisting of the various inputs OR-ed together.  Nonetheless, it is
//! handy to be able to compute intermediate values and read them back elsewhere in the code.
//!
//! * Note that `.next` should _never_ appear on the right hand side of an expression! *
//!
//! The following code will fail to compile, because once we try to derive HDL from the result,
//! RustHDL realizes it makes no sense.
//!
//! ```compile_fail
//! # use rust_hdl::prelude::*;
//!# pub struct OrStuff {
//!#    pub val1: Signal<In, Bit>,
//!#    pub val2: Signal<In, Bits<4>>,
//!#    pub val3: Signal<In, Bits<2>>,
//!#    pub val4: Signal<In, Bit>,
//!#    pub combined: Signal<Out, Bits<8>>,
//!#    pad: Signal<Local, Bits<8>>,
//!# }
//! impl Logic for OrStuff {
//!    #[hdl_gen]
//!    fn update(&mut self) {
//!       self.pad.next = 0.into(); // Write first
//!       self.pad.next = self.pad.next | bit_cast::<8,1>(self.val1.val().into()); // Fails!  Can only write to .next
//!       self.combined.next = self.pad.val();
//!    }
//! }
//!```
//!
//! Detecting the case in which you fail to write to a signal before reading from it is more complicated
//! and must be done a run time.  The macro processor is not sophisticated enough to detect that case at the moment.
//! However, it can be found when your logic is checked for correctness by the static analyzer.
//!
//! Normally, the Verilog code generator or the Simulation engine will statically check your design for you.
//! However, you can also check the design yourself using the [check_all](core::check_error::check_all)
//! function.  Here is an example of that check being run on a logic block that attempts to write
//! to an input signal being passed into the block.  The example panics because
//!
//!```should_panic
//! # use rust_hdl::prelude::*;
//!
//! #[derive(LogicBlock, Default)]
//! struct BadActor {
//!   pub in1: Signal<In, Bit>,
//!   pub in2: Signal<In, Bit>,
//!   pub out1: Signal<Out, Bit>,
//! }
//!
//! impl Logic for BadActor {
//!   #[hdl_gen]
//!   fn update(&mut self) {
//!        // This is definitely not OK
//!        self.in1.next = true;
//!        // This is fine
//!        self.out1.next = self.in2.val();
//!    }
//! }
//!
//! // This will panic with an error of CheckError::WritesToInputs, pointing to self.in1
//! check_all(&BadActor::default()).unwrap()
//! ```

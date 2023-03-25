//! ** Write FPGA Firmware using Rust! **
//!
//!
//! RustHDL is a crate that allows you to write FPGA firmware using Rust!
//! Specifically, `rust-hdl` compiles a subset of Rust down to Verilog so that
//! you can synthesize firmware for your FPGA using standard tools.  It also
//! provides tools for simulation, verification, and analysis along with strongly
//! typed interfaces for making sure your design works before heading to the bench.
//! The workflow is very similar to GPU programming.  You write everything in Rust,
//! including an update `kernel` that compiles down onto the hardware.  You can simulate
//! and verify everything from the safety and comfort of your Rust environment, and
//! then head over to standard synthesis tools to get files that program your FPGA.
//!
//! ## Links
//!
//! You may want:
//!
//! - [API Documentation](https://docs.rs/rust-hdl/latest/rust_hdl/)
//! - [GitHub](https://github.com/samitbasu/rust-hdl)
//! - [Home Page](https://github.com/samitbasu/rust-hdl)
//!
//! ## Features
//! * Safe - have Rust check the validity of your firmware with
//! strongly typed interfaces at **compile** time, as well as at
//! run time, synthesis, and on the device.
//! * Fast - Run simulations of your designs straight from your
//! Rust code, with pretty good simulation performance.
//! * Readable - RustHDL outputs Verilog code for synthesis and
//! implementation, and goes through some effort to make sure that
//! code is readable and understandable, in case you need to resolve
//! timing issues or other conflicts.
//! * Reusable - RustHDL supports templated firmware for parametric
//! use, as well as a simple composition model based on structs.
//! * Batteries Included - RustHDL includes a set of basic firmware
//! widgets that provide FIFOs, RAMs and ROMs, Flip flops, SPI components,
//! PWMs etc, so you can get started quickly.
//! * Free - Although you can use RustHDL to wrap existing IP cores,
//! all of the RustHDL code and firmware is open source and free to use (as in speech and beer).
//! * Tested - RustHDL has been used to write firmware that is shipping in commercial products.
//! This includes quite complicated designs that use nearly all of a moderately sized FPGA,
//! and take advantage of specialized hardware in the FPGAs themselves.
//!
//! ## Quickstart
//!
//! The definitive example in FPGA firmware land is a simple LED blinker.  This typically
//! involves a clock that is fed to the FPGA with a pre-defined frequency, and an output
//! signal that can control an LED.  Because we don't know what FPGA we are using, we will
//! do this in simulation first.  We want a blink that is 250 msec long every second, and
//! our clock speed is (a comically slow) 10kHz.  Here is a minimal working Blinky! example:
//!
//! ```
//! use std::time::Duration;
//! use rust_hdl::prelude::*;
//!
//! const CLOCK_SPEED_HZ : u64 = 10_000;
//!
//! #[derive(LogicBlock)]  // <- This turns the struct into something you can simulate/synthesize
//! struct Blinky {
//!     pub clock: Signal<In, Clock>, // <- input signal, type is clock
//!     pulser: Pulser,               // <- sub-circuit, a widget that generates pulses
//!     pub led: Signal<Out, Bit>,    // <- output signal, type is single bit
//! }
//!
//! impl Default for Blinky {
//!    fn default() -> Self {
//!        Self {
//!          clock: Default::default(),
//!          pulser: Pulser::new(CLOCK_SPEED_HZ, 1.0, Duration::from_millis(250)),
//!          led: Default::default(),
//!        }
//!     }
//! }
//!
//! impl Logic for Blinky {
//!     #[hdl_gen] // <- this turns the update function into an HDL Kernel that can be turned into Verilog
//!     fn update(&mut self) {
//!        // v-- write to the .next member     v-- read from .val() method
//!        self.pulser.clock.next = self.clock.val();
//!        self.pulser.enable.next = true.into();
//!        self.led.next = self.pulser.pulse.val();
//!     }
//! }
//!
//! fn main() {
//!     // v--- build a simple simulation (1 testbench, single clock)
//!     let mut sim = simple_sim!(Blinky, clock, CLOCK_SPEED_HZ, ep, {
//!         let mut x = ep.init()?;
//!         wait_clock_cycles!(ep, clock, x, 4*CLOCK_SPEED_HZ);
//!         ep.done(x)
//!     });
//!
//!     // v--- construct the circuit
//!     let mut uut = Blinky::default();
//!     // v--- run the simulation, with the output traced to a .vcd file
//!     sim.run_to_file(Box::new(uut), 5 * SIMULATION_TIME_ONE_SECOND, "blinky.vcd").unwrap();
//!     vcd_to_svg("/tmp/blinky.vcd","images/blinky_all.svg",&["uut.clock", "uut.led"], 0, 4_000_000_000_000).unwrap();
//!     vcd_to_svg("/tmp/blinky.vcd","images/blinky_pulse.svg",&["uut.clock", "uut.led"], 900_000_000_000, 1_500_000_000_000).unwrap();
//! }
//! ```
//!
//! Running the above (a release run is highly recommended) will generate a `vcd` file (which is
//! a trace file for FPGAs and hardware in general).  You can open this using e.g., `gtkwave`.
//! If you have, for example, an Alchitry Cu board you can generate a bitstream for this exampling
//! with a single call.  It's a little more involved, so we will cover that in the detailed
//! documentation.  It will also render that `vcd` file into an `svg` you can view with an ordinary
//! web browser.  This is the end result showing the entire simulation:
//! ![blinky_all](https://github.com/samitbasu/rust-hdl/raw/main/rust-hdl/images/blinky_all.svg)
//! Here is a zoom in showing the pulse to the LED
//! ![blinky_pulse](https://github.com/samitbasu/rust-hdl/raw/main/rust-hdl/images/blinky_pulse.svg)
//!
//! The flow behind RustHDL is the following:
//!
//! - Circuits are modelled using simple `struct`s, composed of other circuit elements and
//! signal wires that interconnect them.
//! - A `#[derive(LogicBlock)]` annotation on the struct adds autogenerated code needed by
//! RustHDL.
//! - You `impl Logic` on your `struct`, and provide the `fn update(&mut self)` method, which
//! is the HDL update kernel.
//! - That gets annotated with a `#[hdl_gen]` attribute to generate HDL from the Rust code
//! - You can then simulate and synthesize your design - either in software, or by using an
//! appropriate BSP and toolchain.
//!
//! The rest is detail.  Some final things to keep in mind.
//!
//! - RustHDL is a strict subset of Rust.  The `rustc` compiler must be satisfied with your
//! design first.  That means types, exhaustive enum matching, etc.
//! - The goal is to eliminate a class of mistakes that are easy to make in other HDLs with
//! checks taking place at compile time, via static analysis at run time, and then with
//! testbenches.
//! - Although the performance can always be improved, RustHDL is pretty fast, especially in
//! release mode.
//!
//! ## Types
//!
//! There are a couple of key types you should be comfortable with to use RustHDL.  The first
//! is the [Bits](core::bits::Bits) type, which provides a compile-time arbitrary width bit vector.
//!
//! ### Representing bits
//!
//! The [Bits](core::bits::Bits) type is a `Copy` enabled type that you can construct from integers,
//! from the `Default` trait, or from other `Bits`.    Mostly, it is meant to stay out of your way
//! and behave like a `u32`.
//!
//! ```
//! # use rust_hdl::prelude::*;
//! let x: Bits<50> = Default::default();
//! ```
//! This will construct a length 50 bit vector that is initialized to all `0`.
//!
//! You can also convert from literals into bit vectors using the [From] and [Into] traits,
//! provided the literals are of the `u64` type.
//!
//! ```
//! # use rust_hdl::prelude::*;
//! let x: Bits<50> = 0xBEEF.into();
//! ```
//!
//! In some cases, Rust complains about literals, and you may need to provide a suffix:
//! ```
//! # use rust_hdl::prelude::*;
//! let x: Bits<50> = 0xDEAD_BEEF_u64.into();
//! ```
//! However, in most cases, you can leave literals suffix-free, and Rust will automatically
//! determine the type from the context.
//!
//! You can construct a larger constant using the [bits] function.  If you have a literal of up to
//! 128 bits, it provides a functional form
//! ```
//! # use rust_hdl::prelude::*;
//! let x: Bits<200> = bits(0xDEAD_BEEE); // Works for up to 128 bit constants.
//! ```
//!
//! There is also the [ToBits] trait, which is implemented on the basic unsigned integer types.
//! This trait allows you to handily convert from different integer values
//!
//! ```
//! # use rust_hdl::prelude::*;
//! let x: Bits<10> = 32_u8.to_bits();
//! ```
//!
//! ### Operations
//!
//! The [Bits](core::bits::Bits) type supports a subset of operations that can be synthesized in
//! hardware.  You can perform
//!
//! * Addition between `Bits` of the same size using the `+` operator
//! * Subtraction between `Bits` of the same size using the `-` operator
//! * Bitwise logical `AND` between `Bits` of the same size using the `&` operator
//! * Bitwise logical `OR` between `Bits` of the same size using the `|` operator
//! * Bitwise logical `XOR` (Exclusive Or) between `Bits` of the same size using the `^` operator
//! * Bitwise comparisons for equality between `Bits` of the same size using `==` and `!=` operators
//! * Unsigned comparisons (e.g., `>,>=,<,<=`) between `Bits` of the same size - these are
//! always treated as unsigned values for comparison purposes.
//! * Shift left using the `<<` operator
//! * Shift right (no sign extension!) using the '>>' operator
//! * Bitwise logical `NOT` using the `!` prefix operator
//!
//! These should feel natural when using RustHDL, as expressions follow Rust's rules (and not Verilog's).
//! For example:
//! ```rust
//! # use rust_hdl::prelude::*;
//! let x: Bits<32> = 0xDEAD_0000_u32.to_bits();
//! let y: Bits<32> = 0x0000_BEEF_u32.to_bits();
//! let z = x + y;
//! assert_eq!(z, 0xDEAD_BEEF_u32.to_bits());
//! ```
//!
//! You can, of course, construct expressions of arbitrary complexity using parenthesis, etc.
//! The only real surprise may be at synthesis time, when you try to fit the expression onto hardware.
//!
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
//!
//! ## Traits
//!
//! There is only one trait that you typically need to implement to get things to work in RustHDL
//! with the simulation and synthesis frameworks.  That is the [Logic](core::logic::Logic) trait.
//! Although you will rarely (if ever) need to implement the methods themselves, here is the
//! full definition of the trait:
//!```
//! # use rust_hdl::prelude::*;
//!
//! pub trait Logic {
//!     fn update(&mut self);
//!     fn connect(&mut self) {}
//!     fn hdl(&self) -> Verilog {
//!         Verilog::Empty
//!     }
//!     fn timing(&self) -> Vec<TimingInfo> {
//!         vec![]
//!     }
//! }
//! ```
//!
//! The methods are quite simple:
//!
//! * `update` - this updates the state of the logical block based on the inputs and internal state.
//! In general, this is where the action of the logical block takes place.
//! * `connect` - this is where we claim whatever signals we drive, by calling `connect` on them.
//! * `hdl` - this method returns the Verilog description for our logical block in the form of
//! an [Verilog](core::ast::Verilog) enum.
//! * `timing` - this is where specific timing exceptions or requirements are expressed for the
//! logical block.
//!
//! In almost all cases, you will use the `#[derive(LogicBlock)]` macro to derive all of the traits from
//! your own `update` method, written in Rust.  If we revisit the `Blinky` example, note that
//! we only provided the `update` method, with an attribute of `#[hdl_gen]`, which in turn
//! generated the remaining trait implementations:
//! ```
//! # use std::time::Duration;
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::docs::vcd2svg::vcd_to_svg;
//!
//! # const CLOCK_SPEED_HZ : u64 = 10_000;
//!
//! #[derive(LogicBlock)]
//! struct Blinky {
//!     pub clock: Signal<In, Clock>,
//!     pulser: Pulser,
//!     pub led: Signal<Out, Bit>,
//! }
//!
//! # impl Default for Blinky {
//! #   fn default() -> Self {
//! #       Self {
//! #         clock: Default::default(),
//! #         pulser: Pulser::new(CLOCK_SPEED_HZ, 1.0, Duration::from_millis(250)),
//! #         led: Default::default(),
//! #       }
//! #    }
//! # }
//!
//! impl Logic for Blinky {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!        self.pulser.clock.next = self.clock.val();
//!        self.pulser.enable.next = true.into();
//!        self.led.next = self.pulser.pulse.val();
//!     }
//! }  
//!```
//!
//! There are a couple of other traits that RustHDL uses that you should be aware of.
//!
//! - `Synth` - this trait is provided on types that can be represented in hardware, i.e. as
//! a set of bits.  You will probably not need to implement this trait yourself, but if you
//! need some special type representation `Foo`, and `impl Synth for Foo`, then RustHDL will
//! be able to generate Verilog code for it.
//! - `Block` - this trait is needed on any `struct` that is a composition of circuit elements
//! (pretty much every struct used to model a circuit).  This should be auto-derived.
//! - `Logic` - Sometimes, you will need to override the default implementations of the `Logic`
//!trait.  In those cases, (when you are providing a custom simulation model, or wrapping a
//!black box Verilog routine), you will need to `impl` the other methods.
//!
//! ## The Synthesizable Subset of Rust and the HDL Kernel
//!
//! RustHDL uses procedural macros to define a subset of the Rust language that can be used to
//! describe actual hardware.  That subset is known as the synthesizable subset of Rust.  It is
//! quite limited because the end result is translated into Verilog and ultimately into hardware
//! configuration for the FPGA.
//!
//! - The HDL kernel must be valid Rust!  If you remove the `#[hdl_gen]` attribute, the code
//! must still be accepted by `rustc`!  That means you must satisfy the type constraints, the
//! private nature of the struct fields, etc.  This is one of the major benefits of RustHDL.  It
//! takes code that is already been checked by `rustc` and then converts it into HDL.
//!
//! So this will _clearly_ fail to compile.
//!
//! ```compile_fail
//! # use rust_hdl::prelude::*;
//!
//! struct Foo {
//!  bar: Signal<Out, Bits<4>>
//! }
//!
//! impl Logic for Foo {
//!    #[hdl_gen]
//!    fn update(&mut self) {
//!       self.bar.next = "Oy!"; // Type issue here...
//!    }
//! }
//! ```
//!
//! - The `#[hdl_gen]` attribute can only be applied to a function (aka HDL Kernel) that
//! takes `&mut self` as an argument. In almost all cases, you will write something like:
//!
//! ```
//!# use rust_hdl::prelude::*;
//!
//! struct Foo {}
//!
//! impl Logic for Foo {
//!   #[hdl_gen]
//!   fn update(&mut self) {
//!      // Put your synthesizable subset of Rust here...
//!   }
//! }
//! ```
//!
//! - The body of the `update` function must be a single block, consisting of statements.
//! Local definitions and items are not allowed in HDL kernels.  The following, for example, will
//!fail.  This is an example of valid Rust that is not allowed in an HDL kernel.
//!
//!```compile_fail
//! # use rust_hdl::prelude::*;
//!
//! struct Foo {}
//!
//! impl Logic for Foo {
//!    #[hdl_gen]
//!    fn update (&mut self) {
//!      // Fails because local items are not allowed in HDL kernels.
//!      let x = 32;
//!    }
//! }
//!```
//!
//! - Assignments are allowed as long as you follow the rules about signals.  Types are
//! still enforced by Rust.
//!     - Indexed assignments are currently not supported
//!     - Signal assignments must be to either `.next` or `.next.field` if the signal is struct based.
//!
//! So valid assignments will be of the form `self.<signal>.next = <expr>`, or for structure-valued
//! signals.
//!
//! - Expressions support accessing fields of a signal
//! - Binary operations supported are `+`, `-`, `*`, `&&`, `||`, `^`, `&`, `|`, `<<`, `>>`, `==`, `<`, `<=`, `!=`, `>`, `>=`
//! In general, binary operations require that both arguments are of the same type (e.g. bitwidth) or one of the
//! arguments will be a literal.
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! struct Foo {
//!    pub sig1: Signal<In, Bits<4>>,
//!    pub sig2: Signal<In, Bits<4>>,
//!    pub sig3: Signal<Out, Bits<4>>,
//! }
//!
//! impl Logic for Foo {
//!    #[hdl_gen]
//!    fn update(&mut self) {
//!       self.sig3.next = self.sig1.val() + 4; // Example of binop with a literal
//!       self.sig3.next = self.sig1.val() ^ self.sig2.val(); // Example of a binop with two bitvecs
//!    }
//! }
//! ```
//!
//! - Unary operations supported are `-` and `!`
//! The `-` operator is only supported for `Signed` types.  Otherwise, it makes no sense.  If
//! you want to compute the 2's complement of an unsigned value, you need to do so explicitly.
//! The `!` operator will flip all of the bits in the bitvector.
//! - Conditionals (`if`) are supported
//!```rust
//! # use rust_hdl::prelude::*;
//!
//! struct Foo {
//!     pub sig1: Signal<In, Bit>,
//!     pub sig2: Signal<Out, Bits<2>>,
//!     pub sig3: Signal<In, Bits<2>>,
//!     pub sig4: Signal<Out, Bits<2>>,
//! }
//!
//! impl Logic for Foo {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!         self.sig2.next = 0.into(); // Latch prevention!
//!         // Straight `if`s are supported, but beware of latches!
//!         // This `if` statement would generate a latch if not for
//!         // the unconditional assign to `sig2`
//!         if self.sig1.val() {
//!            self.sig2.next = 1.into();
//!         }
//!         // You can use `else` clauses also
//!         if self.sig1.val() {
//!            self.sig2.next = 1.into();
//!         } else {
//!            self.sig2.next = 2.into();
//!         }
//!         // Nesting and chaining are also fine
//!         if self.sig3.val() == 0 {
//!            self.sig4.next = 3.into();
//!         } else if self.sig3.val() == 1 {
//!            self.sig4.next = 2.into();
//!         } else {
//!            self.sig4.next = 0.into();   // <- Fall through else prevents latch
//!         }
//!     }
//! }
//! ```
//! - Literals (provided they implement the `Synth` trait) are supported.  In most cases, you
//! can used un-suffixed literals (like `1` or `0xDEAD`) as add `.into()`.
//! - Function calls - RustHDL kernels support a very limited number of function calls, all of
//!    which are ignored in HDL at the moment (they are provided to make `rustc` happy)
//!     - `bit_cast`
//!     - `signed_bit_cast`
//!     - `unsigned_cast`
//!     - `bits`
//!     - `Bits`
//!     - `Type::join` and `Type::link` used to link and join logical interfaces...
//! - Method calls - Kernels support the following limited set of method calls
//!     - `get_bits` - extract a (fixed width) set of bits from a bit vector
//!     - `get_bit` - extract a single bit from a bit vector
//!     - `replace_bit` - replace a single bit in a bit vector
//!     - `all` - true if all the bits in the bit vector are true
//!     - `any` - true if any of the bits in the bit vector are true
//!     - `xor` - true if the number of ones in the bit vector is odd
//!     - `val`, `into`, `index`, `to_bits` - ignored in HDL kernels
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! struct Foo {
//!     pub sig1: Signal<In, Bits<8>>,
//!     pub sig_index: Signal<In, Bits<3>>,
//!     pub sig2: Signal<Out, Bit>,
//!     pub sig3: Signal<Out, Bits<3>>,
//!     pub sig4: Signal<Out, Bit>,
//! }
//!
//! impl Logic for Foo {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!         self.sig2.next = self.sig1.val().get_bit(self.sig_index.val().index()); // <- Selects specified bit out of sig1
//!         self.sig3.next = self.sig1.val().get_bits::<3>(self.sig_index.val().index()); // Selects 3 bits starting at index `sig_index`
//!         // Notice that here we have an output on both the left and right side of the assignment
//!         // That is fine as long we we write to `.next` before we read from `.val`.
//!         self.sig4.next = self.sig3.val().all(); // True if sig3 is all true
//!     }
//! }
//! ```
//! - Matches - Kernels support matching with literals or identifiers
//! Matches are used for state machines and implementing ROMs.  
//! For now, `match` is a statement, not an expression!  Maybe that will be fixed in a future
//! version of RustHDL, but for now, the value of the `match` is ignored.
//! Here is an example of a `match` for a state machine:
//! ```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State {
//!     Idle,
//!     Running,
//!     Paused,
//! }
//!
//!
//! struct Foo {
//!     pub start: Signal<In, Bit>,
//!     pub pause: Signal<In, Bit>,
//!     pub stop: Signal<In, Bit>,
//!     pub clock: Signal<In, Clock>,
//!     state: DFF<State>,
//! }
//!
//! impl Logic for Foo {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!        dff_setup!(self, clock, state); // <- setup the DFF
//!        match self.state.q.val() {
//!            State::Idle =>
//!                   if self.start.val() {
//!                      self.state.d.next = State::Running;
//!                   }
//!            State::Running =>
//!                   if self.pause.val() {
//!                      self.state.d.next = State::Paused;
//!                   }
//!            State::Paused =>
//!                   if !self.pause.val() {
//!                      self.state.d.next = State::Running;
//!                   }
//!        }
//!        if self.stop.val() {
//!            self.state.d.next = State::Idle;
//!        }
//!     }
//! }
//! ```
//! - Macros - some macros are supported in kernels
//!     - `println` - this is converted into a comment in the generated HDL
//!     - `comment` - also a comment
//!     - `assert` - converted to a comment
//!     - `dff_setup` - setup a DFF - this macro is converted into the appropriate HDL
//!     - `clock` - clock a set of components - this macro is also converted into the appropriate HDL
//! - Loops - `for` loops are supported for code generation
//!     - In software parlance, all `for` loops are unrolled at compile time, so they must be of the form `for <ident> in <const>..<const>`.
//! A simple example to consider is a parameterizable mux.
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! // Mux from N separate signals, using A address bits
//! // For fun, it's also generic over the width of the
//! // signals being muxed.  So there are 3 generics here:
//! //    - D - the type of those signals
//! //    - N - the number of signals being muxed
//! //    - A - the number of address bits (check that 2^A >= N)
//! struct Mux<D: Synth, const N: usize, const A: usize> {
//!    pub input_lines: [Signal<In, D>; N],
//!    pub select: Signal<In, Bits<A>>,
//!    pub outsig: Signal<Out, D>,
//!    fallback: Constant<D>,
//! }
//!
//! // The impl for this requires a for loop
//! impl<D: Synth, const N: usize, const A: usize> Logic for Mux<D, N, A> {
//!   #[hdl_gen]
//!   fn update(&mut self) {
//!        self.outsig.next = self.fallback.val();
//!        for i in 0..N {
//!           if self.select.val().index() == i {
//!              self.outsig.next = self.input_lines[i].val();
//!           }
//!        }
//!    }
//! }
//! ```
//! RustHDL is still pretty restrictive about arrays and loops.  You can still do great stuff though.
//!
//! Since an example is instructive, here is the HDL kernel for a nontrivial circuit (the `SPIMaster`),
//! annotated to demonstrate the various valid bits of syntax.  It's been heavily redacted to make
//! it easier to read.
//!
//! ```
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!# #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//!# enum SPIState {
//!#     Idle,
//!#     Dwell,
//!#     LoadBit,
//!#     MActive,
//!#     SampleMISO,
//!#     MIdle,
//!#     Finish,
//!# }
//!# #[derive(Copy, Clone)]
//!# pub struct SPIConfig {
//!#     pub clock_speed: u64,
//!#     pub cs_off: bool,
//!#     pub mosi_off: bool,
//!#     pub speed_hz: u64,
//!#     pub cpha: bool,
//!#     pub cpol: bool,
//!# }
//!# #[derive(LogicInterface, Default)]
//!# #[join = "SPIWiresSlave"]
//!# pub struct SPIWiresMaster {
//!#     pub mosi: Signal<Out, Bit>,
//!#     pub miso: Signal<In, Bit>,
//!#     pub msel: Signal<Out, Bit>,
//!#     pub mclk: Signal<Out, Bit>,
//!# }
//!# #[derive(LogicInterface, Default)]
//!# #[join = "SPIWiresMaster"]
//!# pub struct SPIWiresSlave {
//!#     pub mosi: Signal<In, Bit>,
//!#     pub miso: Signal<Out, Bit>,
//!#     pub msel: Signal<In, Bit>,
//!#     pub mclk: Signal<In, Bit>,
//!# }
//! // Note - you can use const generics in HDL definitions and kernels!
//! #[derive(LogicBlock)]
//! struct SPIMaster<const N: usize> {
//!     // The `pub` members are the ones you can access from other circuits.
//!     // These form the official interface of the circuit
//!     pub clock: Signal<In, Clock>,
//!     pub bits_outbound: Signal<In, Bits<16>>,
//!     pub data_outbound: Signal<In, Bits<N>>,
//!     // snip...
//!#     pub data_inbound: Signal<Out, Bits<N>>,
//!#     pub start_send: Signal<In, Bit>,
//!#     pub transfer_done: Signal<Out, Bit>,
//!#     pub continued_transaction: Signal<In, Bit>,
//!#     pub busy: Signal<Out, Bit>,
//!#     pub wires: SPIWiresMaster,   // <-- This is a LogicInterface type
//!     // These are private, so they can only be accessed by internal code
//!     register_out: DFF<Bits<N>>,
//!     register_in: DFF<Bits<N>>,
//!     state: DFF<SPIState>,
//!     strobe: Strobe<32>,
//!     pointer: DFF<Bits<16>>,
//!      // snip...
//!#     pointerm1: Signal<Local, Bits<16>>,
//!#     clock_state: DFF<Bit>,
//!#     done_flop: DFF<Bit>,
//!#     msel_flop: DFFWithInit<Bit>,
//!#     mosi_flop: DFF<Bit>,
//!#     continued_save: DFF<Bit>,
//!     // Computed constants need to be stored in a special Constant field member
//!     cs_off: Constant<Bit>,
//!     mosi_off: Constant<Bit>,
//!#     cpha: Constant<Bit>,
//!#     cpol: Constant<Bit>,
//! }
//!
//!impl<const N: usize> Logic for SPIMaster<N> {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!         // Setup the internals - for Latch avoidance, each digital flip flop
//!         // requires setup - it needs to be clocked, and it needs to connect
//!         // the output and input together, so that the input is driven.
//!         // This macro simply declutters the code a bit and makes it easier to read.
//!         dff_setup!(
//!             self,
//!             clock,
//!             //   | equivalent to `self.register_out.clock.next = self.clock.val();`
//!             // v--               `self.register_out.d.next = self.register_out.q.val();`
//!             register_out,
//!             register_in,
//!             state,
//!             pointer,
//!#             clock_state,
//!#             done_flop,
//!#             msel_flop,
//!#             mosi_flop,
//!#             continued_save
//!         );
//!         // This macro is shorthand for `self.strobe.next = self.clock.val();`
//!         clock!(self, clock, strobe);
//!         // These are just standard assignments... Nothing too special.
//!         // Note that `.next` is on the LHS, and `.val()` on the right...
//!         self.strobe.enable.next = true;
//!         self.wires.mclk.next = self.clock_state.q.val();
//!#         self.wires.mosi.next = self.mosi_flop.q.val();
//!         self.wires.msel.next = self.msel_flop.q.val();
//!         self.data_inbound.next = self.register_in.q.val();
//!#         self.transfer_done.next = self.done_flop.q.val();
//!#         self.done_flop.d.next = false;
//!         self.pointerm1.next = self.pointer.q.val() - 1;
//!#         self.busy.next = true;
//!         // The `match` is used to model state machines
//!         match self.state.q.val() {
//!             SPIState::Idle => {
//!                 self.busy.next = false;
//!                 self.clock_state.d.next = self.cpol.val();
//!                 if self.start_send.val() {
//!                     // Capture the outgoing data in our register
//!                     self.register_out.d.next = self.data_outbound.val();
//!                     self.state.d.next = SPIState::Dwell; // Transition to the DWELL state
//!                     self.pointer.d.next = self.bits_outbound.val(); // set bit pointer to number of bit to send (1 based)
//!                     self.register_in.d.next = 0.into(); // Clear out the input store register
//!                     self.msel_flop.d.next = !self.cs_off.val(); // Activate the chip select
//!                     self.continued_save.d.next = self.continued_transaction.val();
//!                 } else {
//!                     if !self.continued_save.q.val() {
//!                         self.msel_flop.d.next = self.cs_off.val(); // Set the chip select signal to be "off"
//!                     }
//!                 }
//!                 self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
//!             }
//!             SPIState::Dwell => {
//!                 if self.strobe.strobe.val() {
//!                     // Dwell timeout has reached zero
//!                     self.state.d.next = SPIState::LoadBit; // Transition to the loadbit state
//!                 }
//!             }
//!             SPIState::LoadBit => {
//!                 // Note in this statement that to use the pointer register as a bit index
//!                 // into the `register_out` DFF, we need to convert it with `index()`.
//!                 if self.pointer.q.val().any() {
//!                     // We have data to send
//!                     self.mosi_flop.d.next = self
//!                         .register_out
//!                         .q
//!                         .val()
//!                         .get_bit(self.pointerm1.val().index()); // Fetch the corresponding bit out of the register
//!#                     self.pointer.d.next = self.pointerm1.val(); // Decrement the pointer
//!                     self.state.d.next = SPIState::MActive; // Move to the hold mclock low state
//!                     self.clock_state.d.next = self.cpol.val() ^ self.cpha.val();
//!                 } else {
//!                     self.mosi_flop.d.next = self.mosi_off.val(); // Set the mosi signal to be "off"
//!                     self.clock_state.d.next = self.cpol.val();
//!                     self.state.d.next = SPIState::Finish; // No data, go back to idle
//!                 }
//!             }
//!             SPIState::MActive => {
//!                 if self.strobe.strobe.val() {
//!                     self.state.d.next = SPIState::SampleMISO;
//!                 }
//!             }
//!#           SPIState::SampleMISO => {}
//!#           SPIState::MIdle => {}
//!#           SPIState::Finish => {}
//!        }
//!     }
//!}
//! ```
//!
//! ## Enums
//!
//! In keeping with Rust's strongly typed model, you can use enums (not sum types) in your HDL,
//! provided you derive the `LogicState` trait for them.  This makes your code much easier to
//! read and debug, and `rustc` will make sure you don't do anything illegal with your
//! enums.
//!  
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State {
//!     Idle,
//!     Running,
//!     Paused,
//! }
//! ```
//!
//! Using enums for storing things like state has several advantages:
//! - RustHDL will automatically calculate the minimum number of bits needed to store the
//! enum in e.g., a register.
//!
//! For example, we can create a Digital Flip Flop (register) of value `State` from the next
//! example, and RustHDL will convert this into a 2 bit binary register.  
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State {
//!     Idle,
//!     Sending,
//!     Receiving,
//!     Done,
//! }
//!
//! struct Foo {
//!     dff: DFF<State>,  // <-- This is a 2 bit DFF
//! }
//! ```
//!
//! Now imagine we add another state in the future to our state machine - say `Pending`:
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State {
//!     Idle,
//!     Sending,
//!     Receiving,
//!     Pending,
//!     Done,
//! }
//!
//! struct Foo {
//!     dff: DFF<State>,  // <-- This is now a 3 bit DFF!
//! }
//! ```
//! RustHDL will _automatically_ choose a 3-bit representation.  
//!
//! - RustHDL will ensure that assignments to `enum`-valued signals are valid at all times
//!
//! The strong type guarantees ensure you cannot assign arbitrary values to `enum` valued
//! signals, and the namespaces ensure that there is no ambiguity in assignment.  This example
//! won't compile, since `On` without the name of the `enum` means nothing, and `State1` and
//! `State2` are separate types.  They cannot be assigned to one another.
//!
//! ```compile_fail
//! # use rust_hdl::prelude::*;
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State1 {
//!      On,
//!      Off,
//! }
//!
//! #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! enum State2 {
//!      Off,
//!      On,
//! }
//!
//! struct Foo {
//!     pub sig_in: Signal<In, State1>,
//!     pub sig_out: Signal<Out, State2>,
//! }
//!
//! impl Logic for Foo {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!         self.sig_out.next = On; // << This won't work either.
//!         self.sig_out.next = self.sig_in.val(); // << Won't compile
//!     }
//! }
//! ```
//!
//! If for some reason, you needed to translate between enums, use a `match`:
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! # #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! # enum State1 {
//! #     On,
//! #     Off,
//! # }
//! # #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
//! # enum State2 {
//! #     Off,
//! #     On,
//! # }
//! #
//! # struct Foo {
//! #     pub sig_in: Signal<In, State1>,
//! #     pub sig_out: Signal<Out, State2>,
//! # }
//! #
//! impl Logic for Foo {
//!    #[hdl_gen]
//!    fn update(&mut self) {
//!       match self.sig_in.val() {
//!           State1::On => self.sig_out.next = State2::On,
//!           State1::Off => self.sig_out.next = State2::Off,
//!       }
//!    }
//! }
//! ```
//!
//! ## Interfaces
//!
//! One area you will encouter as your circuits become more complex is that the interfaces
//! to those circuits will become increasingly complicated.  To demonstrate, suppose you
//! have a circuit that consumes a sequence of 16-bit integers via a FIFO interface.  The
//! circuit has some flow control signals because it cannot consume them every clock
//! cycle (For Reasons).  Suppose also that you have a data producer circuit that will
//! produce 16-bit integers and you want to connect these two together.  A natural
//! FIFO interface would look like this
//!
//! ```rust
//!# use rust_hdl::prelude::*;
//!  struct MyFIFO {
//!      pub data_to_fifo: Signal<In, Bits<16>>,
//!      pub write: Signal<In, Bits<16>>,
//!      pub full: Signal<Out, Bit>,
//!      pub overflow: Signal<Out, Bit>,
//!  }
//!
//!  struct DataWidget {
//!      pub data_to_fifo: Signal<Out, Bits<16>>,
//!      pub write: Signal<Out, Bits<16>>,
//!      pub full: Signal<In, Bit>,
//!      pub overflow: Signal<In, Bit>,
//!  }
//!
//!  struct Foo {
//!     producer: DataWidget,
//!     consumer: MyFIFO,
//!  }
//! ```
//!
//! Now, we want to connect the output of the DataWidget (all 4 signals!) to the corresponding
//! signals on `MyFIFO`.  Keep in mind that the order of assignment is irrelevant, but which
//! signal appears on the LHS vs RHS _is_ important.  In the `impl Logic` block for `Foo`,
//! our HDL kernel will look like this:
//!```rust
//!# use rust_hdl::prelude::*;
//!#  struct MyFIFO {
//!#      pub data_to_fifo: Signal<In, Bits<16>>,
//!#      pub write: Signal<In, Bits<16>>,
//!#      pub full: Signal<Out, Bit>,
//!#      pub overflow: Signal<Out, Bit>,
//!#  }
//!#
//!#  struct DataWidget {
//!#      pub data_to_fifo: Signal<Out, Bits<16>>,
//!#      pub write: Signal<Out, Bits<16>>,
//!#      pub full: Signal<In, Bit>,
//!#      pub overflow: Signal<In, Bit>,
//!#  }
//!#
//!#  struct Foo {
//!#     producer: DataWidget,
//!#     consumer: MyFIFO,
//!#  }
//!impl Logic for Foo {
//!   #[hdl_gen]
//!   fn update(&mut self) {
//!      self.consumer.data_to_fifo.next = self.producer.data_to_fifo.val();
//!      self.consumer.write.next = self.producer.write.val();
//!      self.producer.full.next = self.consumer.full.val();
//!      self.producer.overflow.next = self.consumer.overflow.val();
//!   }
//!}
//!```
//! This is basically boilerplate at this point, and typing that in and getting it right
//! is error prone and tedious.  Fortunately, RustHDL can help!  RustHDL includes the
//! concept of an `Interface`, which is basically a bus.  An `Interface` is generally a
//! pair of structs that contain signals of complementary directions and a `#[derive]`
//! macro that autogenerates a bunch of boilerplate.  To continue on with our previous
//! example, we could define a pair of `struct`s for the write interface of the FIFO
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//! #[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
//! #[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
//! struct MyFIFOWriteReceiver {
//!     pub data_to_fifo: Signal<In, Bits<16>>,
//!     pub write: Signal<In, Bit>,
//!     pub full: Signal<Out, Bit>,
//!     pub overflow: Signal<Out, Bit>,
//! }
//!
//! #[derive(LogicInterface)]       // <- Also here
//! #[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
//! struct MyFIFOWriteSender {
//!    pub data_to_fifo: Signal<Out, Bits<16>>,
//!    pub write: Signal<Out, Bit>,
//!    pub full: Signal<In, Bit>,
//!    pub overflow: Signal<In, Bit>
//! }
//! ```
//!The names of the fields must match, the types of the fields must also match, and the directions
//! of the signals must be complementary.  So in general:
//!
//! - Each field in struct `A` must have a matching named field in struct `B`
//! - The types of those fields must match
//! - The direction of those signals must be opposite
//! - Order of the fields is immaterial
//! - The `join` attribute tells the compiler which interface to mate to this one.
//!
//! So what can we do with our shiny new interfaces?  Plenty of stuff.  First, lets
//! rewrite our FIFO circuit and data producer to use our new interfaces.
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//!#  #[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
//!#  #[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteReceiver {
//!#      pub data_to_fifo: Signal<In, Bits<16>>,
//!#      pub write: Signal<In, Bit>,
//!#      pub full: Signal<Out, Bit>,
//!#      pub overflow: Signal<Out, Bit>,
//!#  }
//!#  #[derive(LogicInterface)]       // <- Also here
//!#  #[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteSender {
//!#     pub data_to_fifo: Signal<Out, Bits<16>>,
//!#     pub write: Signal<Out, Bit>,
//!#     pub full: Signal<In, Bit>,
//!#     pub overflow: Signal<In, Bit>
//!#  }
//! struct MyFIFO {
//!     // The write interface to the FIFO - now only one line!
//!     pub write_bus: MyFIFOWriteReceiver,
//! }
//!
//! struct DataWidget {
//!     // The output interface from the DataWidget!
//!     pub data_out: MyFIFOWriteSender,
//! }
//! ```
//!
//! That is significantly less verbose!  So what happens to our
//! `impl Logic for Foo`?  Well, RustHDL autogenerates 2 methods for each `LogicInterface`.  The first
//! one is called `join`.  And it, well, joins the interfaces.
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//!#  #[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
//!#  #[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteReceiver {
//!#      pub data_to_fifo: Signal<In, Bits<16>>,
//!#      pub write: Signal<In, Bit>,
//!#      pub full: Signal<Out, Bit>,
//!#      pub overflow: Signal<Out, Bit>,
//!#  }
//!#  #[derive(LogicInterface)]       // <- Also here
//!#  #[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteSender {
//!#     pub data_to_fifo: Signal<Out, Bits<16>>,
//!#     pub write: Signal<Out, Bit>,
//!#     pub full: Signal<In, Bit>,
//!#     pub overflow: Signal<In, Bit>
//!#  }
//!#  struct MyFIFO {
//!#      // The write interface to the FIFO - now only one line!
//!#      pub write_bus: MyFIFOWriteReceiver,
//!#  }
//!#  struct DataWidget {
//!#      pub data_out: MyFIFOWriteSender,
//!#  }
//!#  struct Foo {
//!#     producer: DataWidget,
//!#     consumer: MyFIFO,
//!#  }
//! impl Logic for Foo {
//!    #[hdl_gen]
//!    fn update(&mut self) {
//!       // Excess verbosity eliminated!!
//!       MyFIFOWriteSender::join(&mut self.producer.data_out, &mut self.consumer.write_bus);
//!    }
//! }
//! ```
//!
//! This is exactly equivalent to our previous 4 lines of hand crafted code, but is now automatically
//! generated _and_ synthesizable.  But wait!  There is more.  RustHDL also generates a `link`
//! method, which allows you to _forward_ a bus from one point to another.  If you think in terms
//! gendered cables, a `join` is a cable with a Male connector on one end and a Female connector
//! on the other.  A `link` is a cable that is either Male to Male or Female to Female.  Links
//! are useful when you want to forward an interface to an interior component of a circuit, but
//! hide that interior component from the outside world.  For example, lets suppose that
//! `DataWidget` doesn't actually produce the 16-bit samples.  Instead, some other FPGA component
//! or circuit generates the 16-bit samples, and `DataWidget` just wraps it along with some
//! other control logic.  So in fact, our `DataWidget` has an internal representation that looks
//! like this
//!```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//! # struct MyFIFOWriteSender{}
//! struct DataWidget {
//!    pub data_out: MyFIFOWriteSender,
//!    secret_guy: CryptoGenerator,
//!    running: DFF<Bit>,
//! }
//!
//! struct CryptoGenerator {
//!    pub data_out: MyFIFOWriteSender,
//!    // secret stuff!
//! }
//!```  
//!
//! In this example, the `DataWidget` wants to present the outside world that it is a `MyFIFOWriteSender`
//! interface, and that it can produce 16-bit data values.  But the real work is being done internally
//! by the `secret_guy`.  The manual way to do this would be to connect up the signals manually.  Again,
//! paying attention to which signal is an input (for `DataWidget`), and which is an output.
//!
//! ```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!#  #[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
//!#  #[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteReceiver {
//!#      pub data_to_fifo: Signal<In, Bits<16>>,
//!#      pub write: Signal<In, Bit>,
//!#      pub full: Signal<Out, Bit>,
//!#      pub overflow: Signal<Out, Bit>,
//!#  }
//!#  #[derive(LogicInterface)]       // <- Also here
//!#  #[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteSender {
//!#     pub data_to_fifo: Signal<Out, Bits<16>>,
//!#     pub write: Signal<Out, Bit>,
//!#     pub full: Signal<In, Bit>,
//!#     pub overflow: Signal<In, Bit>
//!#  }
//!#  struct DataWidget {
//!#     pub data_out: MyFIFOWriteSender,
//!#     secret_guy: CryptoGenerator,
//!#     running: DFF<Bit>,
//!#  }
//!#  struct CryptoGenerator {
//!#     pub data_out: MyFIFOWriteSender,
//!#     // secret stuff!
//!#  }
//! impl Logic for DataWidget {
//!    #[hdl_gen]
//!     fn update(&mut self) {
//!        // Yawn...
//!        self.data_out.data_to_fifo.next = self.secret_guy.data_out.data_to_fifo.val();
//!        self.data_out.write.next = self.secret_guy.data_out.write.val();
//!        self.secret_guy.data_out.full.next = self.data_out.full.val();
//!        self.secret_guy.data_out.overflow.next = self.data_out.overflow.val();
//!     }
//! }
//! ```
//!
//! In these instances, you can use the `link` method instead.  The syntax is
//! `Interface::link(&mut self.outside, &mut self.inside)`, where `outside` is the
//! side of the interface going out of the circuit, and `inside` is the side of the interface
//! inside of the circuit.  Hence, our interface can be `forwarded` or `linked` with a single line
//! like so:
//! ```rust
//! # use rust_hdl::prelude::*;
//! # use rust_hdl::widgets::prelude::*;
//!#  #[derive(LogicInterface)]     // <- Note the LogicInterface, not LogicBlock
//!#  #[join = "MyFIFOWriteSender"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteReceiver {
//!#      pub data_to_fifo: Signal<In, Bits<16>>,
//!#      pub write: Signal<In, Bit>,
//!#      pub full: Signal<Out, Bit>,
//!#      pub overflow: Signal<Out, Bit>,
//!#  }
//!#  #[derive(LogicInterface)]       // <- Also here
//!#  #[join = "MyFIFOWriteReceiver"] // <- Name of the "mating" interface
//!#  struct MyFIFOWriteSender {
//!#     pub data_to_fifo: Signal<Out, Bits<16>>,
//!#     pub write: Signal<Out, Bit>,
//!#     pub full: Signal<In, Bit>,
//!#     pub overflow: Signal<In, Bit>
//!#  }
//!#  struct DataWidget {
//!#     pub data_out: MyFIFOWriteSender,
//!#     secret_guy: CryptoGenerator,
//!#     running: DFF<Bit>,
//!#  }
//!#  struct CryptoGenerator {
//!#     pub data_out: MyFIFOWriteSender,
//!#     // secret stuff!
//!#  }
//! impl Logic for DataWidget {
//!    #[hdl_gen]
//!     fn update(&mut self) {
//!        // Tada!
//!        MyFIFOWriteSender::link(&mut self.data_out, &mut self.secret_guy.data_out);
//!     }
//! }
//! ```
//!
//! As a parting note, you can make interfaces generic across types.  Here, for example
//! is the FIFO interface used in the High Level Synthesis library in RustHDL:
//! ```rust
//! # use rust_hdl::prelude::*;
//!#[derive(Clone, Debug, Default, LogicInterface)]
//! #[join = "FIFOWriteResponder"]
//! pub struct FIFOWriteController<T: Synth> {
//!     pub data: Signal<Out, T>,
//!     pub write: Signal<Out, Bit>,
//!     pub full: Signal<In, Bit>,
//!     pub almost_full: Signal<In, Bit>,
//! }
//!
//! #[derive(Clone, Debug, Default, LogicInterface)]
//! #[join = "FIFOWriteController"]
//! pub struct FIFOWriteResponder<T: Synth> {
//!     pub data: Signal<In, T>,
//!     pub write: Signal<In, Bit>,
//!     pub full: Signal<Out, Bit>,
//!     pub almost_full: Signal<Out, Bit>,
//! }
//! ```
//!
//! You can then use any synthesizable type for the data bus, and keep the control signals
//! as single bits!  Neat, eh? 🦑
//!
//! ## Simulation
//!
//! Now that you have a shiny new circuit implemented as a struct, what do you do with it?
//! Typically, in hardware design, the first thing you do (after static analysis) is to simulate
//! the circuit.  Simulation allows you to verify the proper behavior of the circuit in software
//! _before_ heading over to the bench to test on the physical hardware.  There is a saying
//! in hardware design "success in simulation is necessary, but not sufficient for correct operation".
//! Or something like that.
//!
//! In any case, RustHDL makes it easy to simulate your designs by allowing you to create and write
//! complex test benches in Rust instead of in an HDL like Verilog or VHDL.  Furthermore, the
//! simulator is built in, so you do not need to use external tools for simulation.  Occasionally,
//! you may need to or want to simulate using external tools.  Currently, RustHDL can't help
//! much there.  You can convert your design to Verilog and then import it into standard
//! simulation tools, but the testbench won't go with the design.  Maybe in the future...
//!
//! The simulator that is built into RustHDL is pretty basic, and easy to use.  To use it, you
//! need a circuit to simulate.  Let's create a basic 8 bit adder with a clocked register for
//! the output (and no carry):
//! ```rust
//! use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions
//!
//! //        v--- Required by RustHDL
//! #[derive(LogicBlock, Default)]
//! struct MyAdder {
//!     pub sig_a: Signal<In, Bits<8>>,
//!     pub sig_b: Signal<In, Bits<8>>,
//!     pub sig_sum: Signal<Out, Bits<8>>,
//!     pub clock: Signal<In, Clock>,
//!     my_reg: DFF<Bits<8>>,
//! }
//!
//! impl Logic for MyAdder {
//!   #[hdl_gen]  // <--- don't forget this
//!   fn update(&mut self) {
//!        // Setup the DFF.. this macro is handy to prevent latches
//!        dff_setup!(self, clock, my_reg);
//!        self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
//!        self.sig_sum.next = self.my_reg.q.val();
//!    }
//! }
//! ```
//!
//! At this point, we can convert `MyAdder` into Verilog and use a standard toolchain to generate
//! a bitfile.  However, we want to verify that the circuit operates properly.   The simplest way
//! to do that would be to feed it a vector of random inputs, and make sure that the output
//! matches the sum of the inputs.  Setting up a simulation can be a little verbose, so there
//! is a handy macro [simple_sim!] that works if you have only a single (top level) clock,
//! and only need one test bench.
//!
//! ** An aside on ownership **
//! We haven't talked about the borrow checker much.  And that is because by and large, RustHDL
//! does not use references.  So how do the testbenches work?  The key points for those of you
//! familiar with Rust are:
//!    - The circuit must be [Send].  All RustHDL components are [Send].
//!    - The simulation uses a [Box] to hold the circuit.
//!    - Each testbench runs in it's own thread.
//!    - The circuit is moved to each testbench as it runs via the endpoint.
//!    - The testbench then updates the circuit inputs, and checks outputs.  It is the
//!      sole owner of the circuit at this point.  
//!    - The techbench then passes the circuit back to the simulation (moves) along with some
//!      indication of when it needs to see it again.
//!    - If a testbench is complete, it signals that it does not need to see the circuit again.
//!    - When all testbenches are complete (or any of them report an error), the simulation
//!      halts.
//!
//! It takes a little getting used to, but the design allows you to write concurrent testbenches
//! without worrying about shared mutable state.
//!
//! So back to our adder.  The testbench should look something like this
//!  - set input A to some known value x
//!  - set input B to some known value y
//!  - wait a clock cycle
//!  - check that the output matches the sum x + y
//!  - loop until complete.
//!
//! Here is a complete example:
//! ```rust
//!# use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions
//!#
//!# //        v--- Required by RustHDL
//!# #[derive(LogicBlock, Default)]
//!# struct MyAdder {
//!#     pub sig_a: Signal<In, Bits<8>>,
//!#     pub sig_b: Signal<In, Bits<8>>,
//!#     pub sig_sum: Signal<Out, Bits<8>>,
//!#     pub clock: Signal<In, Clock>,
//!#     my_reg: DFF<Bits<8>>,
//!# }
//!#
//!# impl Logic for MyAdder {
//!#   #[hdl_gen]  // <--- don't forget this
//!#   fn update(&mut self) {
//!#        // Setup the DFF.. this macro is handy to prevent latches
//!#        dff_setup!(self, clock, my_reg);
//!#        self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
//!#        self.sig_sum.next = self.my_reg.q.val();
//!#    }
//!# }
//!    use rand::{thread_rng, Rng};
//!    use std::num::Wrapping;
//!    // Build a set of test cases for the circuit.  Use Wrapping to emulate hardware.
//!    let test_cases = (0..512)
//!        .map(|_| {
//!            let a_val = thread_rng().gen::<u8>();
//!            let b_val = thread_rng().gen::<u8>();
//!            let c_val = (Wrapping(a_val) + Wrapping(b_val)).0;
//!            (
//!                a_val.to_bits::<8>(),
//!                b_val.to_bits::<8>(),
//!                c_val.to_bits::<8>(),
//!            )
//!        })
//!        .collect::<Vec<_>>();
//!    // The clock speed doesn't really matter here. So 100MHz is fine.
//!    let mut sim = simple_sim!(MyAdder, clock, 100_000_000, ep, {
//!        let mut x = ep.init()?; // Get the circuit
//!        for test_case in &test_cases {
//!            // +--  This should look familiar.  Same rules as for HDL kernels
//!            // v    Write to .next, read from .val()
//!            x.sig_a.next = test_case.0;
//!            x.sig_b.next = test_case.1;
//!            // Helpful macro to delay the simulate by 1 clock cycle (to allow the output to update)
//!            wait_clock_cycle!(ep, clock, x);
//!            // You can use any standard Rust stuff inside the testbench.
//!            println!(
//!                "Test case {:x} + {:x} = {:x} (check {:x})",
//!                test_case.0,
//!                test_case.1,
//!                x.sig_sum.val(),
//!                test_case.2
//!            );
//!            // The `sim_assert_eq` macro stops the simulation gracefully.
//!            sim_assert_eq!(ep, x.sig_sum.val(), test_case.2, x);
//!        }
//!        // When the testbench is complete, call done on the endpoint, and pass the circuit back.
//!        ep.done(x)
//!    });
//!    // Run the simulation - needs a boxed circuit, and a maximum duration.
//!    sim.run(MyAdder::default().into(), sim_time::ONE_MILLISECOND)
//!        .unwrap();
//! ```
//!
//! The above should write the following to your console (your numbers will be different)
//!
//!```bash
//!Test case 5d + 98 = f5 (check f5)
//!Test case 3b + 44 = 7f (check 7f)
//!Test case 5d + b0 = 0d (check 0d)
//!Test case f8 + 38 = 30 (check 30)
//!Test case 73 + b5 = 28 (check 28)
//!Test case 1b + e5 = 00 (check 00)
//!Test case c1 + 89 = 4a (check 4a)
//! etc...
//!```
//!
//! You can also generate a trace of the circuit using the `vcd` (Value Change Dump) format, and
//! read the output using `gtkwave` or some other `vcd` viewer.  RustHDL includes a simple
//! `vcd` renderer for convenience, but its pretty basic, and mostly for creating documentation
//! examples.  It does have the advantage of being callable directly from your testbench in case
//! you need some visual verification of your circuit.  
//!
//! We can make a one line change to our previous example, and generate a `vcd`.
//!
//! ```rust
//!# use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions
//!#
//!# //        v--- Required by RustHDL
//!# #[derive(LogicBlock, Default)]
//!# struct MyAdder {
//!#     pub sig_a: Signal<In, Bits<8>>,
//!#     pub sig_b: Signal<In, Bits<8>>,
//!#     pub sig_sum: Signal<Out, Bits<8>>,
//!#     pub clock: Signal<In, Clock>,
//!#     my_reg: DFF<Bits<8>>,
//!# }
//!#
//!# impl Logic for MyAdder {
//!#   #[hdl_gen]  // <--- don't forget this
//!#   fn update(&mut self) {
//!#        // Setup the DFF.. this macro is handy to prevent latches
//!#        dff_setup!(self, clock, my_reg);
//!#        self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
//!#        self.sig_sum.next = self.my_reg.q.val();
//!#    }
//!# }
//!#    use rand::{thread_rng, Rng};
//!#    use std::num::Wrapping;
//!#    // Build a set of test cases for the circuit.  Use Wrapping to emulate hardware.
//!#    let test_cases = (0..512)
//!#        .map(|_| {
//!#            let a_val = thread_rng().gen::<u8>();
//!#            let b_val = thread_rng().gen::<u8>();
//!#            let c_val = (Wrapping(a_val) + Wrapping(b_val)).0;
//!#            (
//!#                a_val.to_bits::<8>(),
//!#                b_val.to_bits::<8>(),
//!#                c_val.to_bits::<8>(),
//!#            )
//!#        })
//!#        .collect::<Vec<_>>();
//!#    // The clock speed doesn't really matter here. So 100MHz is fine.
//!#    let mut sim = simple_sim!(MyAdder, clock, 100_000_000, ep, {
//!#        let mut x = ep.init()?; // Get the circuit
//!#        for test_case in &test_cases {
//!#            // +--  This should look familiar.  Same rules as for HDL kernels
//!#            // v    Write to .next, read from .val()
//!#            x.sig_a.next = test_case.0;
//!#            x.sig_b.next = test_case.1;
//!#            // Helpful macro to delay the simulate by 1 clock cycle (to allow the output to update)
//!#            wait_clock_cycle!(ep, clock, x);
//!#            // You can use any standard Rust stuff inside the testbench.
//!#            println!(
//!#                "Test case {:x} + {:x} = {:x} (check {:x})",
//!#                test_case.0,
//!#                test_case.1,
//!#                x.sig_sum.val(),
//!#                test_case.2
//!#            );
//!#            // The `sim_assert_eq` macro stops the simulation gracefully.
//!#            sim_assert_eq!(ep, x.sig_sum.val(), test_case.2, x);
//!#        }
//!#        // When the testbench is complete, call done on the endpoint, and pass the circuit back.
//!#        ep.done(x)
//!#    });
//!    // Run the simulation - needs a boxed circuit, and a maximum duration.
//!    sim.run_to_file(
//!        MyAdder::default().into(),
//!        sim_time::ONE_MILLISECOND,
//!        &vcd_path!("my_adder.vcd"),
//!    )
//!    .unwrap();
//!    vcd_to_svg(
//!        &vcd_path!("my_adder.vcd"),
//!        "images/my_adder.svg",
//!        &[
//!            "uut.clock",
//!            "uut.sig_a",
//!            "uut.sig_b",
//!            "uut.my_reg.d",
//!            "uut.my_reg.q",
//!            "uut.sig_sum",
//!        ],
//!        0,
//!        100 * sim_time::ONE_NANOSECOND,
//!    )
//!    .unwrap()
//! ```
//! The result of that simulation is here.
//! ![my_adder_sim](https://github.com/samitbasu/rust-hdl/raw/main/rust-hdl/images/my_adder.svg)
//! Note that the digital flip flop copies it's input from `d` to `q` on the leading edge of the clock.
//!
//! ## Generating Verilog
//!
//! At some point, you will want to generate Verilog so you can send your design to other
//! tools.  This is pretty simple.  You call [generate_verilog] and pass it a reference
//! to the circuit in question.  The [generate_verilog] function will check your circuit,
//! and then return a string that contains the Verilog equivalent.  It's pretty simple.
//!
//! Here is an example.  We will reuse the `MyAdder` circuit from the testbench section,
//! but this time, generate the Verilog for the circuit instead of simulating it.
//!
//! ```rust
//! use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions
//!
//! //        v--- Required by RustHDL
//! #[derive(LogicBlock, Default)]
//! struct MyAdder {
//!     pub sig_a: Signal<In, Bits<8>>,
//!     pub sig_b: Signal<In, Bits<8>>,
//!     pub sig_sum: Signal<Out, Bits<8>>,
//!     pub clock: Signal<In, Clock>,
//!     my_reg: DFF<Bits<8>>,
//! }
//!
//! impl Logic for MyAdder {
//!   #[hdl_gen]  // <--- don't forget this
//!   fn update(&mut self) {
//!        // Setup the DFF.. this macro is handy to prevent latches
//!        dff_setup!(self, clock, my_reg);
//!        self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
//!        self.sig_sum.next = self.my_reg.q.val();
//!    }
//! }
//!
//! let mut uut = MyAdder::default();
//! uut.connect_all();
//! println!("{}", generate_verilog(&uut));
//! ```
//!
//! You should get the following generated code in your console:
//! ```verilog
//! module top(sig_a,sig_b,sig_sum,clock);
//!
//!     // Module arguments
//!     input wire  [7:0] sig_a;
//!     input wire  [7:0] sig_b;
//!     output reg  [7:0] sig_sum;
//!     input wire  clock;
//!
//!     // Stub signals
//!     reg  [7:0] my_reg$d;
//!     wire  [7:0] my_reg$q;
//!     reg  my_reg$clock;
//!
//!     // Sub module instances
//!     top$my_reg my_reg(
//!         .d(my_reg$d),
//!         .q(my_reg$q),
//!         .clock(my_reg$clock)
//!     );
//!
//!     // Update code
//!     always @(*) begin
//!         my_reg$clock = clock;
//!         my_reg$d = my_reg$q;
//!         my_reg$d = sig_a + sig_b;
//!         sig_sum = my_reg$q;
//!     end
//!
//! endmodule // top
//!
//!
//! module top$my_reg(d,q,clock);
//!
//!     // Module arguments
//!     input wire  [7:0] d;
//!     output reg  [7:0] q;
//!     input wire  clock;
//!
//!     // Update code (custom)
//!     initial begin
//!        q = 8'h0;
//!     end
//!
//!     always @(posedge clock) begin
//!        q <= d;
//!     end
//!
//! endmodule // top$my_reg
//! ```
//!
//! A few things about the Verilog generated.
//!   - It is hierarchical (scoped) by design.  The scopes mimic the scopes inside the RustHDL circuit.
//!  That makes it easy to map the Verilog back to the RustHDL code if needed when debugging.
//!   - The code is readable and formatted.
//!   - The names correspond to the names in RustHDL, which makes it easy to see the details of the logic.
//!   - RustHDL (at least for this trivial example) is a pretty thin wrapper around Verilog.  That's
//! good for compatibility with tooling.
//!
//! While most FPGAs will require you to use a proprietary and closed source toolchain to synthesize
//! your design, you can use the open source [Yosys] compiler (if you have it installed) to
//! check your designs.  For that, you can use the [yosys_validate] function, which runs the Verilog
//! through some checks and reports on potential errors.  At the moment, [Yosys] is far more
//! thorough in it's checking than RustHDL, so I highly recommend you install it and use the
//! [yosys_validate] function on your generated Verilog.
//!
//! ## Struct valued signals
//!
//! We have seen how Enums and Interfaces can help make your code more compact and readable.  There
//! is another abstraction you can use to simplify your code.  Interfaces allow you to group together
//! signals that are logically related into a named bundle (like a bus).  You can also group
//! together `bits` into a logically related bundle that can be treated as a single entity.  
//! While this is supported in RustHDL, it's not frequently that useful.  Nonetheless.
//!
//! Suppose you have a set of signals to your circuit that all travel in the same direction,
//! but are different widths.  Any maybe some of the elements are enums.  Something like this
//!
//! ```rust
//!# use rust_hdl::prelude::*;
//! struct Foo {
//!    pub in_red: Signal<In, Bits<5>>,
//!    pub in_green: Signal<In, Bits<8>>,
//!    pub in_blue: Signal<In, Bits<8>>,
//!    pub in_alpha: Signal<In, Bits<6>>,
//! }
//! ```
//!
//! Instead, we can define a struct and annotate it with [LogicStruct], which makes it into a
//! type that can be used for a signal.
//! ```rust
//!# use rust_hdl::prelude::*;
//!    #[derive(Default, PartialEq, LogicStruct, Copy, Clone, Debug)]
//!    struct Color {
//!        pub red: Bits<5>,
//!        pub green: Bits<8>,
//!        pub blue: Bits<8>,
//!        pub alpha: Bits<6>,
//!    }
//!
//!    struct Foo {
//!        pub in_color: Signal<In, Color>,
//!        pub local_color: Signal<Local, Color>,
//!        pub out_color: Signal<Out, Color>,
//!    }
//!
//!    impl Logic for Foo {
//!        #[hdl_gen]
//!        fn update(&mut self) {
//!            self.local_color.next = self.in_color.val(); // Copy the struct as a single atom
//!                                     // v-- Extract a single field using standard `.field` notation
//!            if self.local_color.val().alpha.get_bit(4) {
//!                self.local_color.next.red = 16.into(); // Assign to a single field of the struct
//!            }
//!            self.out_color.next = self.local_color.val();
//!        }
//!    }
//! ```
//!
//! From within the HDL kernel, you can access the fields of the struct as you normally would.  You can
//! also assign entire structs to one another, as well as individual fields of a struct.  The generated
//! Verilog is messy, and I don't use struct valued signals much.  But if you need to use them they are
//! there.
//!
//! ## Loops and Arrays
//!
//! A frequently useful feature of hardware is to be able to handle a variable number of
//! inputs or outputs based on some parameter.  Examples might include:
//!  - A processing stage with a variable number of passes
//!  - A mux with a variable number of inputs
//!  - A bank of identical state machines, where the number of banks is variable
//!
//! In all of these cases, the tool to reach for is an array in RustHDL.  Including an array
//! of subcircuits is pretty simple.  You simply use a static sized array (via a `const generic`
//! parameter) or a `vec`.  Here is an example of a circuit that contains a configurable number
//! of subcircuits, each of which is an instance of the `Pulser` circuit (a standard RustHDL
//! widget)
//! ```rust
//! # use rust_hdl::prelude::*;
//!
//! struct PulserSet<const N: usize> {
//!     pub outs: Signal<Out, Bits<N>>,
//!     pub clock: Signal<In, Clock>,
//!     pulsers: [Pulser; N]
//! }
//! ```
//!
//! In this case, as long as the members of the array implement `Block` (i.e., are circuits),
//! everything will work as expected, including simulation and synthesis.  
//!
//! Frequently, though, having an array of subcircuits means you need a way to loop over them
//! in order to do something useful with their inputs or outputs.  Loops are were software-centric
//! thinking can get you into trouble very quickly.  In hardware, it's best to think of loops
//! in terms of unrolling.  A `for` loop in RustHDL does not actually loop over anything in
//! the hardware.  Rather it is a way of repeating a block of code multiple times with a varying
//! parameter.
//!
//! So the `impl Logic` HDL kernel of the [PulserSet] example above might look something like this:
//! ```
//! # use rust_hdl::prelude::*;
//! # struct PulserSet<const N: usize> {
//! #  pub outs: Signal<Out, Bits<N>>,
//! #  pub clock: Signal<In, Clock>,
//! #  pulsers: [Pulser; N]    
//! # }
//! impl<const N: usize> Logic for PulserSet<N> {
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!         // Connect all the clocks & enable them all
//!         for i in 0..N {
//!            self.pulsers[i].clock.next = self.clock.val();
//!            self.pulsers[i].enable.next = true.into();
//!         }
//!         // Connect the outputs...
//!         self.outs.next = 0.into();
//!         for i in 0..N {
//!            self.outs.next = self.outs.val().replace_bit(i, self.pulsers[i].pulse.val());
//!         }
//!     }
//! }
//! ```
//!
//! Note that we are both reading and writing from `self.outs` in the same kernel, but we write
//! first, which makes it OK.  Reading first would make this latching behavior, and RustHDL (or
//! `yosys`) would throw a fit.
//!
//! You can do some basic manipulations with the index (like using `3*i+4`, for example), but
//! don't get carried away.  Those expressions are evaluated by the HDL kernel generator and
//! it has a limited vocab.
//!
//! ## High Level Synthesis
//!
//! RustHDL supports it's own version of High Level Synthesis (HLS).  Normally, this is some kind
//! of strange drag-and-drop based entry or visual programming paradigm.  Worse, still, it could
//! be some kind of macro meta-language that you build complex designs out of using a graphical
//! editor.  
//!
//! But that is not the case here!  RustHDL's HLS approach is much simpler.  Essentially,
//! even though [Interfaces] are so great, you may not want to use them.  So the core widgets,
//! like the [AsynchronousFIFO] do not use Interfaces.  That leads to some pretty gnarly
//! circuit definitions.  Here is the [AsynchronousFIFO] for example:
//! ```rust
//! # use rust_hdl::prelude::*;
//! pub struct AsynchronousFIFO<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
//!     // Read interface
//!     pub read: Signal<In, Bit>,
//!     pub data_out: Signal<Out, D>,
//!     pub empty: Signal<Out, Bit>,
//!     pub almost_empty: Signal<Out, Bit>,
//!     pub underflow: Signal<Out, Bit>,
//!     pub read_clock: Signal<In, Clock>,
//!     pub read_fill: Signal<Out, Bits<NP1>>,
//!     // Write interface
//!     pub write: Signal<In, Bit>,
//!     pub data_in: Signal<In, D>,
//!     pub full: Signal<Out, Bit>,
//!     pub almost_full: Signal<Out, Bit>,
//!     pub overflow: Signal<Out, Bit>,
//!     pub write_clock: Signal<In, Clock>,
//!     pub write_fill: Signal<Out, Bits<NP1>>,
//!     // Internals ommitted...
//! }
//! ```
//! Using an [AsynchronousFIFO] requires up to 14 separate signals!  With mixed directions and types!
//! Fortunately, there is an HLS wrapper type you can use instead.  It's _much_ simpler
//! ```rust
//! # use rust_hdl::prelude::*;
//! pub struct AsyncFIFO<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
//!     pub bus_write: FIFOWriteResponder<T>,
//!     pub write_clock: Signal<In, Clock>,
//!     pub bus_read: FIFOReadResponder<T>,
//!     pub read_clock: Signal<In, Clock>,
//!     fifo: AsynchronousFIFO<T, N, NP1, BLOCK_SIZE>,
//! }
//!```
//!
//! In this case, it has only 4 signals, and using it is also much easier.  You can use the
//! [FIFOWriteResponder] and [FIFOWriteController] busses to send and receive data from the
//! asynchronous fifo.  Even better is the fact that this HLS construct is just a thin wrapper
//! around the [AsynchronousFIFO], so when you synthesize it, or plot signals, there is nothing
//! extra under the hood.
//!
//! The HLS library also includes a sort of System-on-chip model in case you want to use it.
//! It allows you to connect a set of widgets to a single controller, and route data to them
//! over a fixed bus using a very simple protocol.  It won't replace AXI or WishBone, but it
//! can be used to build some pretty complicated designs and remain readable.  The test cases
//! are a good place to look for some runnable examples of the different SoC widgets.
//!
//! ## Wrapping IP Cores
//!
//! Occasionally in RustHDL, you will need to wrap an external IP core or logic primitive supported
//! by your hardware, but that is not supported directly in RustHDL.  There best method for wrapping
//! Verilog code is to use the [Wrapper](core::ast::Wrapper) struct and provide your own implementation
//! of the `hdl` method for your logic.
//!
//! Here is a minimal example of a clock buffer primitive (that takes a differential clock input
//! and provides a single ended clock output).  The Verilog module declaration for the clock buffer is
//! simply:
//! ```verilog
//! module IBUFDS(I, B, O);
//!    input I;
//!    input B;
//!    output O;
//! endmodule
//! ```
//!
//! Since the implementation of this device is built into the FPGA (it is a hardware primitive),
//! the module definition is enough for the toolchain to construct the device.  Here is a
//! complete example of a wrapped version of this for use in RustHDL.
//!
//!```rust
//! # use rust_hdl::prelude::*;
//!
//! #[derive(LogicBlock, Default)]
//! pub struct ClockDriver {
//!   pub clock_p: Signal<In, Clock>,
//!   pub clock_n: Signal<In, Clock>,
//!   pub sys_clock: Signal<Out, Clock>,
//! }
//!
//! impl Logic for ClockDriver {
//!     // Our simulation simply forwards the positive clock to the system clock
//!     fn update(&mut self) {
//!         self.sys_clock.next = self.clock_p.val();
//!     }
//!     // RustHDL cannot determine what signals are driven based on the declaration
//!     // alone.  This method identifies `sys_clock` as being driven by the internal
//!     // logic of the device.
//!     fn connect(&mut self) {
//!         self.sys_clock.connect();
//!     }
//!     // Normally the `hdl` method is generated by the `derive` macro.  But in this
//!     // case we implement it ourselves to wrap the Verilog code.
//!      fn hdl(&self) -> Verilog {
//!         Verilog::Wrapper(Wrapper {
//!           code: r#"
//!     // This is basically arbitrary Verilog code that lives inside
//!     // a scoped module generated by RustHDL.  Whatever IP cores you
//!     // use here must have accompanying core declarations in the
//!     // cores string, or they will fail verification.
//!     //
//!     // In this simple case, we remap the names here
//!     IBUFDS ibufds_inst(.I(clock_p), .B(clock_n), .O(sys_clock));
//!
//! "#.into(),
//!     // Some synthesis tools (like [Yosys] need a blackbox declaration so they
//!     // can process the Verilog if they do not have primitives in their
//!     // libraries for the device.  Other toolchains will strip these out.
//!           cores: r#"
//! (* blackbox *)
//! module IBUFDS(I, B, O);
//!   input I;
//!   input B;
//!   output O;
//! endmodule"#.into(),
//!         })
//!      }
//! }
//! ```
//!

#![warn(missing_docs)]

///! Tools for documenting RustHDL designs, including the generation of SVGs from simulation waveforms.
pub mod docs;
///! A series of High Level Synthesis blocks used to build System-on-Chip designs quickly.
pub use rust_hdl_hls as hls;
///! Prelude module defines common symbols to make importing RustHDL easier.
pub mod prelude;
///! The core RustHDL module.  Defines variable width bits, signals, logical blocks, etc.
pub use rust_hdl_core as core;
///! A set of routines for dealing with FPGA specific pieces.  Either tools for synthesis, or
/// logic circuits that are specific to an FPGA family.
#[cfg(feature = "fpga")]
pub use rust_hdl_fpga_support as fpga;
///! Support for the OpalKelly devices (including HDL components and the FrontPanel API)
#[cfg(feature = "ok")]
pub use rust_hdl_ok_core as ok;
#[cfg(feature = "ok")]
pub use rust_hdl_ok_frontpanel_sys as frontpanel;
///! Module that contains all code related to simulating RustHDL designs in Rust (i.e., without
///! an external Verilog simulator).
pub use rust_hdl_sim as sim;
///! A set of core widgets useful for FPGA based designs, all written in RustHDL.  This includes
///! elements such as Digital Flip Flops, Block RAMs, ROMs, FIFOs, SDRAM controllers, SPI controllers
///! I2C controllers, FIR filters, etc.
pub use rust_hdl_widgets as widgets;

#[test]
fn doc_sim() {
    use crate::prelude::*; // <- shorthand to bring in all definitions
    use rand::{thread_rng, Rng};
    use std::num::Wrapping;

    //        v--- Required by RustHDL
    #[derive(LogicBlock, Default)]
    struct MyAdder {
        pub sig_a: Signal<In, Bits<8>>,
        pub sig_b: Signal<In, Bits<8>>,
        pub sig_sum: Signal<Out, Bits<8>>,
        pub clock: Signal<In, Clock>,
        my_reg: DFF<Bits<8>>,
    }

    impl Logic for MyAdder {
        #[hdl_gen] // <--- don't forget this
        fn update(&mut self) {
            // Setup the DFF.. this macro is handy to prevent latches
            dff_setup!(self, clock, my_reg);
            self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
            self.sig_sum.next = self.my_reg.q.val();
        }
    }

    let test_cases = (0..512)
        .map(|_| {
            let a_val = thread_rng().gen::<u8>();
            let b_val = thread_rng().gen::<u8>();
            let c_val = (Wrapping(a_val) + Wrapping(b_val)).0;
            (
                a_val.to_bits::<8>(),
                b_val.to_bits::<8>(),
                c_val.to_bits::<8>(),
            )
        })
        .collect::<Vec<_>>();
    let mut sim = simple_sim!(MyAdder, clock, 100_000_000, ep, {
        let mut x = ep.init()?; // Get the circuit
        for test_case in &test_cases {
            x.sig_a.next = test_case.0;
            x.sig_b.next = test_case.1;
            wait_clock_cycle!(ep, clock, x);
            println!(
                "Test case {:x} + {:x} = {:x} (check {:x})",
                test_case.0,
                test_case.1,
                x.sig_sum.val(),
                test_case.2
            );
            sim_assert_eq!(ep, x.sig_sum.val(), test_case.2, x);
        }
        ep.done(x)
    });
    /*    sim.run(MyAdder::default().into(), sim_time::ONE_MILLISECOND)
    .unwrap();*/
    sim.run_to_file(
        MyAdder::default().into(),
        sim_time::ONE_MILLISECOND,
        &vcd_path!("my_adder.vcd"),
    )
    .unwrap();
    vcd_to_svg(
        &vcd_path!("my_adder.vcd"),
        "/tmp/my_adder.svg",
        &[
            "uut.clock",
            "uut.sig_a",
            "uut.sig_b",
            "uut.my_reg.d",
            "uut.my_reg.q",
            "uut.sig_sum",
        ],
        0,
        100 * sim_time::ONE_NANOSECOND,
    )
    .unwrap()
}

#[test]
fn doc_vlog() {
    use crate::prelude::*;

    #[derive(Copy, Clone, PartialEq, Debug, LogicState)]
    enum State {
        Idle,
        Sending,
        Receiving,
        Done,
    }

    #[derive(LogicBlock, Default)]
    struct Foo {
        clock: Signal<In, Clock>,
        dff: DFF<State>, // <-- This is a 2 bit DFF
    }

    impl Logic for Foo {
        #[hdl_gen]
        fn update(&mut self) {
            dff_setup!(self, clock, dff);
            match self.dff.q.val() {
                State::Idle => {
                    self.dff.d.next = State::Sending;
                }
                State::Sending => {
                    self.dff.d.next = State::Receiving;
                }
                State::Receiving => {
                    self.dff.d.next = State::Done;
                }
                State::Done => {
                    self.dff.d.next = State::Idle;
                }
            }
        }
    }

    let mut uut = Foo::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
}

#[test]
fn doc_struct_valued() {
    use crate::prelude::*;
    #[derive(Default, PartialEq, LogicStruct, Copy, Clone, Debug)]
    struct Color {
        pub red: Bits<5>,
        pub green: Bits<8>,
        pub blue: Bits<8>,
        pub alpha: Bits<6>,
    }

    struct Foo {
        pub in_color: Signal<In, Color>,
        pub local_color: Signal<Local, Color>,
        pub out_color: Signal<Out, Color>,
    }

    impl Logic for Foo {
        #[hdl_gen]
        fn update(&mut self) {
            self.local_color.next = self.in_color.val(); // Copy the struct as a single atom
                                                         // v-- Extract a single field using standard `.field` notation
            if self.local_color.val().alpha.get_bit(4) {
                self.local_color.next.red = 16.into(); // Assign to a single field of the struct
            }
            self.out_color.next = self.local_color.val();
        }
    }
}

#[test]
fn doc_verilog_demo() {
    use crate::prelude::*; // <- shorthand to bring in all definitions

    //        v--- Required by RustHDL
    #[derive(LogicBlock, Default)]
    struct MyAdder {
        pub sig_a: Signal<In, Bits<8>>,
        pub sig_b: Signal<In, Bits<8>>,
        pub sig_sum: Signal<Out, Bits<8>>,
        pub clock: Signal<In, Clock>,
        my_reg: DFF<Bits<8>>,
    }

    impl Logic for MyAdder {
        #[hdl_gen] // <--- don't forget this
        fn update(&mut self) {
            // Setup the DFF.. this macro is handy to prevent latches
            dff_setup!(self, clock, my_reg);
            self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
            self.sig_sum.next = self.my_reg.q.val();
        }
    }

    let mut uut = MyAdder::default();
    uut.connect_all();
    println!("{}", generate_verilog(&uut));
}

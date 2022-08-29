//! Write FPGA Firmware using Rust
//!
//! `rust-hdl` is a crate that allows you to write FPGA firmware using Rust!
//! Specifically, `rust-hdl` compiles a subset of Rust down to Verilog so that
//! you can synthesize firmware for your FPGA using standard tools.  It also
//! provides tools for simulation, verification, and analysis along with strongly
//! typed interfaces for making sure your design works before heading to the bench.
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
//! use rust_hdl::core::prelude::*;
//! use rust_hdl::docs::vcd2svg::vcd_to_svg;
//! use rust_hdl::widgets::prelude::*;
//!
//! const CLOCK_SPEED_HZ : u64 = 10_000;
//!
//!
//! #[derive(LogicBlock)]
//! struct Blinky {
//!     pub clock: Signal<In, Clock>,
//!     pulser: Pulser,
//!     pub led: Signal<Out, Bit>,
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
//!     #[hdl_gen]
//!     fn update(&mut self) {
//!        self.pulser.clock.next = self.clock.val();
//!        self.pulser.enable.next = true.into();
//!        self.led.next = self.pulser.pulse.val();
//!     }
//! }
//!
//! fn main() {
//!     let mut sim = simple_sim!(Blinky, clock, CLOCK_SPEED_HZ, ep, {
//!         let mut x = ep.init()?;
//!         wait_clock_cycles!(ep, clock, x, 4*CLOCK_SPEED_HZ);
//!         ep.done(x)
//!     });
//!
//!     let mut uut = Blinky::default();
//!     uut.connect_all();
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
//! web browser.  This is the end result
//!
//! ![](images/blinky_all.svg)
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
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = Default::default();
//! ```
//! This will construct a length 50 bit vector that is initialized to all `0`.
//!
//! You can also convert from literals into bit vectors using the [From] and [Into] traits,
//! provided the literals are of the `u64` type.
//!
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = 0xBEEF.into();
//! ```
//!
//! In some cases, Rust complains about literals, and you may need to provide a suffix:
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<50> = 0xDEAD_BEEF_u64.into();
//! ```
//! However, in most cases, you can leave literals suffix-free, and Rust will automatically
//! determine the type from the context.
//!
//! You can construct a larger constant using the [bits] function.  If you have a literal of up to
//! 128 bits, it provides a functional form
//! ```
//! # use rust_hdl::core::prelude::*;
//! let x: Bits<200> = bits(0xDEAD_BEEE); // Works for up to 128 bit constants.
//! ```
//!
//! There is also the [ToBits] trait, which is implemented on the basic unsigned integer types.
//! This trait allows you to handily convert from different integer values
//!
//! ```
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::constraint::PinConstraint;
//! # use rust_hdl::core::direction::Direction;
//! # use rust_hdl::core::synth::Synth;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
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
//! # use rust_hdl::core::prelude::*;
//! # use rust_hdl::core::timing::TimingInfo;
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
//! # use rust_hdl::core::prelude::*;
//! # use rust_hdl::docs::vcd2svg::vcd_to_svg;
//! # use rust_hdl::widgets::prelude::*;
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
//! ## The Synthesizable Subset of Rust
//!
//! ## Interfaces
//!
//! ## Struct valued signals
//!
//! ## Simulation
//!
//! ## Generating Verilog
//!
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
//!```
//! # use rust_hdl::core::prelude::*;
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
//!
#![warn(missing_docs)]

/// The core RustHDL module.  Defines variable width bits, signals, logical blocks, etc.
pub mod core;
/// Tools for documenting RustHDL designs, including the generation of SVGs from simulation waveforms.
pub mod docs;
/// A series of High Level Synthesis blocks used to build System-on-Chip designs quickly.
pub mod hls;
/// Module that contains all code related to simulating RustHDL designs in Rust (i.e., without
/// an external Verilog simulator).
pub mod sim;
/// A set of core widgets useful for FPGA based designs, all written in RustHDL.  This includes
/// elements such as Digital Flip Flops, Block RAMs, ROMs, FIFOs, SDRAM controllers, SPI controllers
/// I2C controllers, FIR filters, etc.
pub mod widgets;

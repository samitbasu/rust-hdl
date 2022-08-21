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

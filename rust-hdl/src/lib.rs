//! Write FPGA Firmware using Rust
//!
//! This crate allows you to write FPGA firmware using Rust!  There
//! are a number of advantages to writing firmware in Rust vs.
//! other approaches:
//! - Safe - have Rust check the validity of your firmware with
//! strongly typed interfaces at **compile** time, as well as at
//! run time, synthesis, and on the device.
//! - Fast - Run simulations of your designs straight from your
//! rust code, with pretty good simulation performance.
//! - Readable - RustHDL outputs Verilog code for synthesis and
//! implementation, and goes through some effort to make sure that
//! code is readable and understandable, in case you need to resolve
//! timing issues or other conflicts.
//! - Reusable - RustHDL supports templated firmware for parametric
//! use, as well as a simple composition model based on structs.
//! - Batteries Included - RustHDL includes a set of basic firmware
//! widgets that provide FIFOs, RAMs and ROMs, Flip flops, SPI components,
//! PWMs etc, so you can get started quickly.
//! - Free - Although you can use RustHDL to wrap existing IP cores,
//! all of the RustHDL code and firmware is open source and free to use.
//!
//! This crate is the top level crate, and provides a single installation
//! point for a number of the component crates that make up RustHDL.
//! These crates are focused on different types of functionality, and provide
//! a different subset of the functionality of RustHDL.  Here are the
//! component crates
//! - Core - this crate provides the core functionality of RustHDL.  It includes
//! the base data types and traits needed to construct and simulate firmware.
//! - Macros - this crate provides the procedural macro support that allows you
//! to use the Rust syntax to write firmware.
//! - Widgets - this crate provides a set of basic firmware widgets that can be
//! used to compose designs.  These are all written in Rust, and do not require
//! third party IP cores.
//! - Yosys-Synth - this crate provides access to `yosys`, which is used as the
//! synthesis and validation tool for RustHDL.  Generated Verilog can be checked
//! using `yosys` for correctness.
//! - Toolchain - There are several toolchain crates that cover different toolchain
//! options for RustHDL.  You can use your own toolchain as well.  Toolchain
//! support is very complicated, and currently, only ISE, Vivado, IceStorm and
//! Project Trellis are known to work.
//! - Board Support Package - the BSP crates provide details specific to a single
//! board type.  These crates provide details about clock inputs, output pins
//! and synthesis flags needed to generate downloadable firmware.
//! - Sim Chips - a few "simulated chips" - RustHDL simplified models for some
//! physical chips that might prove useful for testing your firmware against.
//! These don't provide the equivalent functionality as the real chip, but
//! give you a way to test your firmware in a completely simulated environment.
//! - OK Core - Wrappers for the OpalKelly FrontPanel firmware components. If
//! you used devices from OpalKelly, this may prove useful to you.
//! - Test - some various test infrastructure firmware and tools for testing.
//!

#![warn(missing_docs)]

pub mod toolchain;

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

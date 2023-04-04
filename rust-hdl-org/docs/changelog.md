---
sidebar_position: 99
---

# Change Log

## v0.44.0

- A bunch of internal fixes for dealing with some stale crates on crates.io that
I couldn't do anything about.  So I renamed the internal crates and split them
up into `rust_hdl_lib_*`.  As an end-user it should be transparent, and ideally
you only need to deal with the `rust-hdl` top-level crate unless you need a 
BSP for a specific FPGA board.

## v0.43.0

- Add support for right and left shifts by different bit width amounts.  For example,
the following construct used to not be allowed, since RustHDL would insist that the
two arguments to both the left and right shift operators be the same bitwidth.

```rust
#[derive(LogicBlock)]
struct Foo {
    pub sig1: Signal<In, Bits<4>>,
    pub sig2: Signal<In, Bits<2>>,
    pub sig3: Signal<Out, Bits<4>>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
        self.sig3.next = self.sig1.val() << self.sig2.val(); // <-- used to require a bitcast - now it doesn't
    }
}
```

## v0.42.0

- Support for comparison operations on signed arguments.  Support for signed quantities is
still improving in RustHDL.  Now you can do:

```rust
#[derive(LogicBlock)]
struct Foo {
    pub sig1: Signal<In, Signed<4>>,
    pub sig2: Signal<In, Signed<4>>,
    pub sig3: Signal<Out, Bit>,
}

impl Logic for Foo {
    #[hdl_gen]
    fn update(&mut self) {
        self.sig3.next = self.sig1.val() < self.sig2.val(); // <-- used to not work at all.  Now it works
    }
}
```

The generated Verilog should have the proper `$signed` and `$unsigned` decorations, but it's good to double
check the output to make sure it looks reasonable.

## v0.41.0

No changes.

## v0.40.0

- Refactored the I2C bus to make it slightly easier to use.  Now the bus looks like this:

```rust
#[derive(LogicInterface, Default)]
#[join = "I2CBusReceiver"]
pub struct I2CBusDriver {
    pub sda: OpenDrainDriver,
    pub scl: OpenDrainDriver,
}
```

The `OpenDrainDriver` is also a bus definition:

```rust
#[derive(LogicInterface, Default)]
#[join = "OpenDrainReceiver"]
pub struct OpenDrainDriver {
    pub drive_low: Signal<Out, Bit>,
    pub line_state: Signal<In, Bit>,
}
```

This constrains (through the types) the operations that can be legally completed on an open drain pin.

## v0.39.0

- There are some issues with synthesis tools that do not properly route analog signals into the design (no names...)
So to be safe you should move your tristate buffers as close to the top of your design hierarchy as possible.  
This release removed the I2C tristate buffers from the controller and moved them out.

## v0.38.0

- Clean up some of the doc tests, and remove some debug statements.  Update the homepage in the Cargo manifest to 
point to the [website](https://www.rust-hdl.org).

## v0.37.1

- Removed the `yosys`-based input-write detection, which erroneously flagged a legitimate case
involving a tristate signal.

## v0.37.0

- Updated the simualted ADS8688 to be closer to the behavior described in the datasheet.

## v0.36.0

- Documentation improvements.
- Add logic to the analysis pass to look for writes to the input signals.

## v0.35.1

- Added some additional tests for SPI modes.

## v0.35.0

- Added the simulated ADS8688 chip.

## v0.34.0

- No new features - Manifest fixup.

## v0.33.0

- Fix XEM6010 support
- Crate cleanup
- Split the test code back out put the BSPs back into their own crates.
- Update the itnegration tests for the Alchitry Cu
- Moved the toolchain files to the `fpga-support` crate.

# Older versions

- Unfortunately, previous releases were not done with the `cargo release` tool
- So I can't correlate the releases to the precise commits easily.




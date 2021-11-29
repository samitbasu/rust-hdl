# Chapter 1 - Quickstart

It's traditional in the FPGA world to have the simplest possible example of
firmware do something totally trivial, like blink an LED at a fixed rate.  This
is the "Hello World" equivalent for FPGAs.  Unfortunately, it's a bit more
complex than writing "Hello World" in C or Rust, because we need to know enough
about the FPGA we are using to generate firmware for it.  For this first
example, I will use the Alchitry CU board.  This board is based on the Lattice
ICE40 FPGA, and works well with the open source toolchain IceStorm.  

I'm assuming you have installed `icestorm` already.  You will also need `yosys`,
which is an open source toolchain that converts the Verilog output from 
RustHDL into other forms we will look at shortly.

Each circuit in RustHDL maps to a `struct`.  The struct itself is pretty simple
(and meant to be straight-foward to use).  Here is an example of a blinker 
for the Alchitry CU:

```rust
#[derive(LogicBlock)]
pub struct AlchitryCuPulser {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
}
```

This circuit consists of 3 elements.  

- `pulser`: This is a circuit that actually pulses the LED.  It's part of the
base widgets provided by RustHDL.  We will look at it in a moment.
- 'clock': This is the clock that drives the FPGA.  All FPGAs will have at least
one external clock source.  Out logic is synchronous (clock based), and so we
must have a clock source for it to work.
- 'leds': There are physical LEDs (8 of them) connected to pins on the Alchitry Cu
FPGA.  To change their state, we will need to be able to change the logic levels
on those pins.  

We will go into more detail on these pieces later.  For now, lets continue
on with our example.  We have declared the composition of our circuit.  The
next step is to describe how the elements are routed.

```rust
impl Logic for AlchitryCuPulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        self.pulser.clock.next = self.clock.val();
        self.leds.next = 0x00_u32.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA_u32.into();
        }
    }
}
```

Again, this is a quickstart, so there isn't a lot of detail yet.  The `Logic` trait
is used by RustHDL to signify that a struct describes a logical circuit.  The use
the `#[hdl_gen]` attribute since we want RustHDL to autogenerate the Verilog for us.
Finally we provide the *behavioral* part of the circuit.  This is the `update`
function.  It takes a mutable reference to `self`, and calculates how the data moves
through the circuit.   In this case, we enable the `pulser` circuit, connect the
clock to its inputs, and then use it's output to control the LEDs.  Don't worry if
it looks mysterious.  It's actually not.  But let's move on to constructing the
circuit.

The last stage is construction of the circuit.  Here, we need to instantiate the 
components, provide them with the required details, etc.  For our example, we need
to actually connect the `clock` and `leds` to their hardware pins.  This is provided
by the "board support package" or "bsp".  Every FPGA board you use will probably need some 
kind of BSP.  There is one for the Alchitry Cu as part of RustHDL.  So we can just
use that.  The BSP will provide mappings for I/O as well as settings for the toolchain,
etc.

Here is the construction of the Pulser:
```rust
impl Default for AlchitryCuPulser {
    fn default() -> Self {
        let pulser = Pulser::new(rust_hdl_bsp_alchitry_cu::pins::CLOCK_SPEED_100MHZ, 1.0, Duration::from_millis(250));
        Self {
            pulser,
            clock: rust_hdl_bsp_alchitry_cu::pins::clock(),
            leds: rust_hdl_bsp_alchitry_cu::pins::leds(),
        }
    }
}
```

Note that we use the `Default` trait pretty regularly in RustHDL.  Whenever a
circuit/part/widget does not require customization or parameterization, use
the `Default` trait to make construction simpler.

If you have a basic grasp of Rust, this should look pretty ordinary.  We will
look under the hood later.  Still trying to get that blinking LED!

Now you need to compile the example.  There are a few ways to do this.  When you
have a big, standalone project, it makes sense to spin a binary crate for your
firmware.  For something simple like this, it may be easier to use the test 
runner functionality in Rust.  Here's what that looks like:

```rust
#[test]
fn synthesize_alchitry_cu_pulser() {
    let uut = AlchitryCuPulser::default();
    generate_bitstream(uut, target_path!("alchitry_cu/pulser"));
}
```

If you run this test function (using `cargo test`, for example), and you 
have installed the prerequisites (`yosys`, `icestorm`), you should end up
with a firmware image that you can program onto the device.  

That's it!  A first example firmware written in Rust.  The next few chapters
will cover some of the structures and classes involved in more detail.


For the `Pulser`, circuit, we can look it up.  It's in the `rust-hdl` widget 
library (I prefer the term `widget` over `core`, since `core` can mean many
many things, and is often associated with closed or proprietary components.)
The `Pulser` declaration is:

```rust
#[derive(LogicBlock)]
pub struct Pulser {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<32>,
    shot: Shot<32>,
}
```

Again, this looks pretty straightforward, and not magical.  We have a pulser
circuit that has a clock and enable input, a pulse output, and some more internal
circuits.  The constructor for a `Pulser` tells us what we need to know to use it

```rust
impl Pulser {
    pub fn new(clock_rate_hz: u64, pulse_rate_hz: f64, pulse_duration: Duration) -> Self 
#    {
#    Self
#}
}
```

In a nutshell, the Pulser takes some parameters in its constructor.  These describe
the clock speed it is provided, the rate (in Hertz) that you want a pulse to occur,
and a duration for the pulse to stay on each time.  

For the Alchitry Cu, 



Before we continue, let's examine the types of these elements.  First, the `Signal`
class.  The `Signal` class basically looks like this:

```rust
#[derive(Clone, Debug)]
pub struct Signal<D: Direction, T: Synth> {
    pub next: T,
    val: T,
    prev: T,
    pub changed: bool,
    claimed: bool,
    id: usize,
    constraints: Vec<PinConstraint>,
    dir: std::marker::PhantomData<D>,
}
```

It is one of the fundamental classes in RustHDL, 
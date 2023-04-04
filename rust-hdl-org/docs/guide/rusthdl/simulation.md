# Simulation

Now that you have a shiny new circuit implemented as a struct, what do you do with it?
Typically, in hardware design, the first thing you do (after static analysis) is to simulate
the circuit.  Simulation allows you to verify the proper behavior of the circuit in software
_before_ heading over to the bench to test on the physical hardware.  There is a saying
in hardware design "success in simulation is necessary, but not sufficient for correct operation".
Or something like that.

In any case, RustHDL makes it easy to simulate your designs by allowing you to create and write
complex test benches in Rust instead of in an HDL like Verilog or VHDL.  Furthermore, the
simulator is built in, so you do not need to use external tools for simulation.  Occasionally,
you may need to or want to simulate using external tools.  Currently, RustHDL can't help
much there.  You can convert your design to Verilog and then import it into standard
simulation tools, but the testbench won't go with the design.  Maybe in the future...

The simulator that is built into RustHDL is pretty basic, and easy to use.  To use it, you
need a circuit to simulate.  Let's create a basic 8 bit adder with a clocked register for
the output (and no carry):

```rust
use rust_hdl::prelude::*;   // <- shorthand to bring in all definitions

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
  #[hdl_gen]  // <--- don't forget this
  fn update(&mut self) {
       // Setup the DFF.. this macro is handy to prevent latches
       dff_setup!(self, clock, my_reg);
       self.my_reg.d.next = self.sig_a.val() + self.sig_b.val();
       self.sig_sum.next = self.my_reg.q.val();
   }
}
```

At this point, we can convert `MyAdder` into Verilog and use a standard toolchain to generate
a bitfile.  However, we want to verify that the circuit operates properly.   The simplest way
to do that would be to feed it a vector of random inputs, and make sure that the output
matches the sum of the inputs.  Setting up a simulation can be a little verbose, so there
is a handy macro [simple_sim!] that works if you have only a single (top level) clock,
and only need one test bench.

:::info ** An aside on ownership **
We haven't talked about the borrow checker much.  And that is because by and large, RustHDL
does not use references.  So how do the testbenches work?  The key points for those of you
familiar with Rust are:
   - The circuit must be [Send].  All RustHDL components are [Send].
   - The simulation uses a [Box] to hold the circuit.
   - Each testbench runs in it's own thread.
   - The circuit is moved to each testbench as it runs via the endpoint.
   - The testbench then updates the circuit inputs, and checks outputs.  It is the
     sole owner of the circuit at this point.  
   - The techbench then passes the circuit back to the simulation (moves) along with some
     indication of when it needs to see it again.
   - If a testbench is complete, it signals that it does not need to see the circuit again.
   - When all testbenches are complete (or any of them report an error), the simulation
     halts.
It takes a little getting used to, but the design allows you to write concurrent testbenches
without worrying about shared mutable state.
:::


So back to our adder.  The testbench should look something like this
 - set input A to some known value x
 - set input B to some known value y
 - wait a clock cycle
 - check that the output matches the sum x + y
 - loop until complete.

Here is a complete example:
```rust
   use rand::{thread_rng, Rng};
   use std::num::Wrapping;
   // Build a set of test cases for the circuit.  Use Wrapping to emulate hardware.
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
   // The clock speed doesn't really matter here. So 100MHz is fine.
   let mut sim = simple_sim!(MyAdder, clock, 100_000_000, ep, {
       let mut x = ep.init()?; // Get the circuit
       for test_case in &test_cases {
           // +--  This should look familiar.  Same rules as for HDL kernels
           // v    Write to .next, read from .val()
           x.sig_a.next = test_case.0;
           x.sig_b.next = test_case.1;
           // Helpful macro to delay the simulate by 1 clock cycle (to allow the output to update)
           wait_clock_cycle!(ep, clock, x);
           // You can use any standard Rust stuff inside the testbench.
           println!(
               "Test case {:x} + {:x} = {:x} (check {:x})",
               test_case.0,
               test_case.1,
               x.sig_sum.val(),
               test_case.2
           );
           // The `sim_assert_eq` macro stops the simulation gracefully.
           sim_assert_eq!(ep, x.sig_sum.val(), test_case.2, x);
       }
       // When the testbench is complete, call done on the endpoint, and pass the circuit back.
       ep.done(x)
   });
   // Run the simulation - needs a boxed circuit, and a maximum duration.
   sim.run(MyAdder::default().into(), sim_time::ONE_MILLISECOND)
       .unwrap();
```

The above should write the following to your console (your numbers will be different)

```bash
Test case 5d + 98 = f5 (check f5)
Test case 3b + 44 = 7f (check 7f)
Test case 5d + b0 = 0d (check 0d)
Test case f8 + 38 = 30 (check 30)
Test case 73 + b5 = 28 (check 28)
Test case 1b + e5 = 00 (check 00)
Test case c1 + 89 = 4a (check 4a)
etc...
```

You can also generate a trace of the circuit using the `vcd` (Value Change Dump) format, and
read the output using `gtkwave` or some other `vcd` viewer.  RustHDL includes a simple
`vcd` renderer for convenience, but its pretty basic, and mostly for creating documentation
examples.  It does have the advantage of being callable directly from your testbench in case
you need some visual verification of your circuit.  

We can make a one line change to our previous example, and generate a `vcd`.

```rust
   // Run the simulation - needs a boxed circuit, and a maximum duration.
   sim.run_to_file(
       MyAdder::default().into(),
       sim_time::ONE_MILLISECOND,
       &vcd_path!("my_adder.vcd"),
   )
   .unwrap();
   vcd_to_svg(
       &vcd_path!("my_adder.vcd"),
       "images/my_adder.svg",
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
```
The result of that simulation is here.
![my_adder_sim](https://github.com/samitbasu/rust-hdl/raw/main/rust-hdl/images/my_adder.svg)
Note that the digital flip flop copies it's input from `d` to `q` on the leading edge of the clock.


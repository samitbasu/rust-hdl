# Loops and Arrays

A frequently useful feature of hardware is to be able to handle a variable number of
inputs or outputs based on some parameter.  Examples might include:
 - A processing stage with a variable number of passes
 - A mux with a variable number of inputs
 - A bank of identical state machines, where the number of banks is variable

In all of these cases, the tool to reach for is an array in RustHDL.  Including an array
of subcircuits is pretty simple.  You simply use a static sized array (via a `const generic`
parameter) or a `vec`.  Here is an example of a circuit that contains a configurable number
of subcircuits, each of which is an instance of the `Pulser` circuit (a standard RustHDL
widget)
```rust
# use rust_hdl::prelude::*;

struct PulserSet<const N: usize> {
    pub outs: Signal<Out, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pulsers: [Pulser; N]
}
```

In this case, as long as the members of the array implement `Block` (i.e., are circuits),
everything will work as expected, including simulation and synthesis.  

Frequently, though, having an array of subcircuits means you need a way to loop over them
in order to do something useful with their inputs or outputs.  Loops are were software-centric
thinking can get you into trouble very quickly.  In hardware, it's best to think of loops
in terms of unrolling.  A `for` loop in RustHDL does not actually loop over anything in
the hardware.  Rather it is a way of repeating a block of code multiple times with a varying
parameter.

So the `impl Logic` HDL kernel of the [PulserSet] example above might look something like this:
```rust
impl<const N: usize> Logic for PulserSet<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect all the clocks & enable them all
        for i in 0..N {
           self.pulsers[i].clock.next = self.clock.val();
           self.pulsers[i].enable.next = true.into();
        }
        // Connect the outputs...
        self.outs.next = 0.into();
        for i in 0..N {
           self.outs.next = self.outs.val().replace_bit(i, self.pulsers[i].pulse.val());
        }
    }
}
```

Note that we are both reading and writing from `self.outs` in the same kernel, but we write
first, which makes it OK.  Reading first would make this latching behavior, and RustHDL (or
`yosys`) would throw a fit.

You can do some basic manipulations with the index (like using `3*i+4`, for example), but
don't get carried away.  Those expressions are evaluated by the HDL kernel generator and
it has a limited vocab.
